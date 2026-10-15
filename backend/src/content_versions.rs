//! Internal version storage. Public authorization and moderation stay at the callers.
use crate::postgres::Pg;
use axum::http::StatusCode;
use serde_json::Value;
use sqlx::{Postgres, Row, Transaction};

fn db(_: sqlx::Error) -> StatusCode { StatusCode::INTERNAL_SERVER_ERROR }

impl Pg {
    pub(crate) async fn init_content_versions(&self) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(self.t("content_versions")).execute(&mut *tx).await?;
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (
            kind TEXT NOT NULL CHECK(kind IN ('prompt','collection','skill')),
            content_id TEXT NOT NULL, author_email TEXT, current_version BIGINT NOT NULL CHECK(current_version>0),
            PRIMARY KEY(kind,content_id))", self.t("content_heads"))).execute(&mut *tx).await?;
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (
            kind TEXT NOT NULL, content_id TEXT NOT NULL, version BIGINT NOT NULL CHECK(version>0),
            submission_id TEXT NOT NULL, base_version BIGINT NOT NULL CHECK(base_version>=0),
            change_note TEXT NOT NULL, snapshot JSONB NOT NULL CHECK(jsonb_typeof(snapshot)='object'),
            published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            PRIMARY KEY(kind,content_id,version), UNIQUE(kind,submission_id),
            FOREIGN KEY(kind,content_id) REFERENCES {}(kind,content_id))",
            self.t("content_versions"),self.t("content_heads"))).execute(&mut *tx).await?;
        // Legacy rows keep their IDs and bytes. Never refresh a v1 snapshot on restart.
        sqlx::query(&format!("WITH legacy AS (
            SELECT s.kind,s.id content_id,p.author_email,
              to_jsonb(s) || jsonb_build_object('asset_refs',COALESCE(p.asset_refs,'[]'::jsonb),'cover',p.cover) snapshot
            FROM {} s LEFT JOIN {} p ON p.id=s.id AND p.status='approved'
            WHERE s.kind IN ('prompt','collection')
            UNION ALL
            SELECT 'skill',p.id,p.author_email,jsonb_build_object('id',p.id,'title',p.title,
              'description',p.description,'category',p.category,'license',p.license,'bundle',p.bundle,'digest',p.digest)
            FROM {} p WHERE p.status='approved'
          ), added AS (
            INSERT INTO {} (kind,content_id,author_email,current_version)
            SELECT kind,content_id,author_email,1 FROM legacy ON CONFLICT DO NOTHING RETURNING kind,content_id
          ) INSERT INTO {} (kind,content_id,version,submission_id,base_version,change_note,snapshot)
            SELECT l.kind,l.content_id,1,'legacy:'||l.content_id,0,'',l.snapshot
            FROM legacy l JOIN added a USING(kind,content_id)",
            self.t("square_items"),self.t("publications"),self.t("skill_publications"),
            self.t("content_heads"),self.t("content_versions"))).execute(&mut *tx).await?;
        tx.commit().await
    }

    /// Called inside the same transaction as publication approval and current-content replacement.
    /// This does not authorize an HTTP request and must never be exposed as a direct write endpoint.
    pub(crate) async fn append_content_version(
        &self, tx: &mut Transaction<'_, Postgres>, kind: &str, content_id: &str,
        author: &str, submission_id: &str, base_version: i64, note: &str, snapshot: &Value,
    ) -> Result<i64, StatusCode> {
        if !matches!(kind,"prompt"|"collection"|"skill") || content_id.is_empty() || author.is_empty()
            || submission_id.is_empty() || base_version < 0 || note.chars().count()>2000 || !snapshot.is_object() {
            return Err(StatusCode::BAD_REQUEST);
        }
        if base_version == 0 {
            sqlx::query(&format!("INSERT INTO {} (kind,content_id,author_email,current_version) VALUES($1,$2,$3,1) ON CONFLICT DO NOTHING",self.t("content_heads")))
                .bind(kind).bind(content_id).bind(author).execute(&mut **tx).await.map_err(db)?;
        }
        let head=sqlx::query(&format!("SELECT author_email,current_version FROM {} WHERE kind=$1 AND content_id=$2 FOR UPDATE",self.t("content_heads")))
            .bind(kind).bind(content_id).fetch_optional(&mut **tx).await.map_err(db)?.ok_or(StatusCode::NOT_FOUND)?;
        if head.get::<Option<String>,_>("author_email").as_deref()!=Some(author) { return Err(StatusCode::FORBIDDEN); }
        if let Some(prior)=sqlx::query(&format!("SELECT content_id,version,base_version,change_note,snapshot FROM {} WHERE kind=$1 AND submission_id=$2",self.t("content_versions")))
            .bind(kind).bind(submission_id).fetch_optional(&mut **tx).await.map_err(db)? {
            if prior.get::<String,_>("content_id")!=content_id || prior.get::<i64,_>("base_version")!=base_version
                || prior.get::<String,_>("change_note")!=note || prior.get::<Value,_>("snapshot")!=*snapshot { return Err(StatusCode::CONFLICT); }
            return Ok(prior.get("version"));
        }
        let current:i64=head.get("current_version");
        let empty:bool=sqlx::query_scalar(&format!("SELECT NOT EXISTS(SELECT 1 FROM {} WHERE kind=$1 AND content_id=$2)",self.t("content_versions")))
            .bind(kind).bind(content_id).fetch_one(&mut **tx).await.map_err(db)?;
        if !(base_version==0 && empty || base_version==current && !empty) { return Err(StatusCode::CONFLICT); }
        let next=base_version.checked_add(1).ok_or(StatusCode::CONFLICT)?;
        sqlx::query(&format!("INSERT INTO {} (kind,content_id,version,submission_id,base_version,change_note,snapshot) VALUES($1,$2,$3,$4,$5,$6,$7)",self.t("content_versions")))
            .bind(kind).bind(content_id).bind(next).bind(submission_id).bind(base_version).bind(note).bind(snapshot)
            .execute(&mut **tx).await.map_err(db)?;
        sqlx::query(&format!("UPDATE {} SET current_version=$3 WHERE kind=$1 AND content_id=$2",self.t("content_heads")))
            .bind(kind).bind(content_id).bind(next).execute(&mut **tx).await.map_err(db)?;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn migration_is_additive_and_never_refreshes_history() {
        let state = crate::admin_security_tests::state().await;
        let pg = state.db.as_ref().unwrap();
        sqlx::query(&format!("INSERT INTO {}(id,title,kind,content,download_count) VALUES('version-legacy','Original','prompt','Original bytes',7)",pg.t("square_items")))
            .execute(&pg.pool).await.unwrap();
        pg.init_content_versions().await.unwrap();
        let read = format!("SELECT snapshot FROM {} WHERE content_id='version-legacy'",pg.t("content_versions"));
        let original: Value = sqlx::query_scalar(&read).fetch_one(&pg.pool).await.unwrap();
        assert_eq!(original["content"], "Original bytes");
        sqlx::query(&format!("UPDATE {} SET content='Changed independently' WHERE id='version-legacy'",pg.t("square_items")))
            .execute(&pg.pool).await.unwrap();
        pg.init_content_versions().await.unwrap();
        let after: Value = sqlx::query_scalar(&read).fetch_one(&pg.pool).await.unwrap();
        assert_eq!(original, after);
        let row: (String,i64) = sqlx::query_as(&format!("SELECT content,download_count FROM {} WHERE id='version-legacy'",pg.t("square_items")))
            .fetch_one(&pg.pool).await.unwrap();
        assert_eq!(row,("Changed independently".into(),7));
        let mut tx=pg.pool.begin().await.unwrap();
        assert_eq!(pg.append_content_version(&mut tx,"prompt","version-legacy","someone","claim",1,"",&json!({})).await,Err(StatusCode::FORBIDDEN));
    }

    #[tokio::test]
    async fn revisions_require_owner_and_current_base_and_rollback_together() {
        let state = crate::admin_security_tests::state().await;
        let pg = state.db.as_ref().unwrap();
        for kind in ["prompt","collection","skill"] {
            let first=json!({"body":"first"});
            let second=json!({"body":"second"});
            let mut tx=pg.pool.begin().await.unwrap();
            assert_eq!(pg.append_content_version(&mut tx,kind,"item","author","s1",0,"initial",&first).await,Ok(1));
            tx.commit().await.unwrap();
            let mut tx=pg.pool.begin().await.unwrap();
            assert_eq!(pg.append_content_version(&mut tx,kind,"item","intruder","s2",1,"update",&second).await,Err(StatusCode::FORBIDDEN));
            tx.rollback().await.unwrap();
            let mut tx=pg.pool.begin().await.unwrap();
            assert_eq!(pg.append_content_version(&mut tx,kind,"item","author","s2",1,"update",&second).await,Ok(2));
            tx.rollback().await.unwrap();
            let head: i64=sqlx::query_scalar(&format!("SELECT current_version FROM {} WHERE kind=$1 AND content_id='item'",pg.t("content_heads"))).bind(kind).fetch_one(&pg.pool).await.unwrap();
            assert_eq!(head,1);
            let mut tx=pg.pool.begin().await.unwrap();
            assert_eq!(pg.append_content_version(&mut tx,kind,"item","author","s2",1,"update",&second).await,Ok(2));
            tx.commit().await.unwrap();
            let mut tx=pg.pool.begin().await.unwrap();
            assert_eq!(pg.append_content_version(&mut tx,kind,"item","author","s1",0,"initial",&first).await,Ok(1));
            assert_eq!(pg.append_content_version(&mut tx,kind,"item","author","s2",1,"update",&first).await,Err(StatusCode::CONFLICT));
            assert_eq!(pg.append_content_version(&mut tx,kind,"item","author","s3",1,"stale",&first).await,Err(StatusCode::CONFLICT));
            tx.rollback().await.unwrap();
        }
    }
}
