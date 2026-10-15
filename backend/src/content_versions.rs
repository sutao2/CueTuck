//! Internal version storage. Public authorization and moderation stay at the callers.
use crate::postgres::Pg;
use axum::http::StatusCode;
use serde_json::Value;
use sqlx::{Postgres, Row, Transaction};

fn db(_: sqlx::Error) -> StatusCode { StatusCode::INTERNAL_SERVER_ERROR }

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RevisionRequest {
    pub content_id: String,
    pub base_version: i64,
    pub change_note: String,
    pub request_id: String,
}

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
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (
            kind TEXT NOT NULL, submission_id TEXT NOT NULL, content_id TEXT NOT NULL,
            base_version BIGINT NOT NULL CHECK(base_version>0), change_note TEXT NOT NULL,
            request JSONB NOT NULL, PRIMARY KEY(kind,submission_id))",self.t("content_candidates")))
            .execute(&mut *tx).await?;
        // Legacy rows keep their IDs and bytes. Never refresh a v1 snapshot on restart.
        sqlx::query(&format!("WITH legacy AS (
            SELECT s.kind,s.id content_id,p.author_email,
              to_jsonb(s) || jsonb_build_object('asset_refs',COALESCE(p.asset_refs,'[]'::jsonb),'cover',p.cover) snapshot
            FROM {} s LEFT JOIN {} p ON p.id=s.id AND p.status='approved'
            WHERE s.kind IN ('prompt','collection')
            UNION ALL
            SELECT 'skill',p.id,p.author_email,jsonb_build_object('id',p.id,'title',p.title,
              'description',p.description,'category',p.category,'license',p.license,'bundle',p.bundle,'digest',p.digest)
            FROM {} p WHERE p.status='approved' AND NOT EXISTS(SELECT 1 FROM {5} c WHERE c.kind='skill' AND c.submission_id=p.id)
          ), added AS (
            INSERT INTO {} (kind,content_id,author_email,current_version)
            SELECT kind,content_id,author_email,1 FROM legacy ON CONFLICT DO NOTHING RETURNING kind,content_id
          ) INSERT INTO {} (kind,content_id,version,submission_id,base_version,change_note,snapshot)
            SELECT l.kind,l.content_id,1,'legacy:'||l.content_id,0,'',l.snapshot
            FROM legacy l JOIN added a USING(kind,content_id)",
            self.t("square_items"),self.t("publications"),self.t("skill_publications"),
            self.t("content_heads"),self.t("content_versions"),self.t("content_candidates"))).execute(&mut *tx).await?;
        tx.commit().await
    }

    pub(crate) async fn register_content_candidate(&self, tx: &mut Transaction<'_, Postgres>, kind: &str,
        submission: &str, author: &str, revision: &RevisionRequest, request: &Value,
    ) -> Result<(), StatusCode> {
        if revision.base_version < 1 || revision.content_id.is_empty() || revision.change_note.trim().is_empty()
            || revision.change_note.chars().count()>2000 || uuid::Uuid::parse_str(&revision.request_id).is_err() {
            return Err(StatusCode::BAD_REQUEST);
        }
        let head=sqlx::query(&format!("SELECT author_email,current_version FROM {} WHERE kind=$1 AND content_id=$2 FOR UPDATE",self.t("content_heads")))
            .bind(kind).bind(&revision.content_id).fetch_optional(&mut **tx).await.map_err(db)?.ok_or(StatusCode::NOT_FOUND)?;
        if head.get::<Option<String>,_>("author_email").as_deref()!=Some(author) {return Err(StatusCode::FORBIDDEN)}
        if let Some(prior)=sqlx::query_scalar::<_,Value>(&format!("SELECT request FROM {} WHERE kind=$1 AND submission_id=$2",self.t("content_candidates")))
            .bind(kind).bind(submission).fetch_optional(&mut **tx).await.map_err(db)? {
            return if prior==*request {Ok(())}else{Err(StatusCode::CONFLICT)};
        }
        if head.get::<i64,_>("current_version")!=revision.base_version {return Err(StatusCode::CONFLICT)}
        sqlx::query(&format!("INSERT INTO {}(kind,submission_id,content_id,base_version,change_note,request) VALUES($1,$2,$3,$4,$5,$6)",self.t("content_candidates")))
            .bind(kind).bind(submission).bind(&revision.content_id).bind(revision.base_version).bind(&revision.change_note).bind(request)
            .execute(&mut **tx).await.map_err(db)?;
        Ok(())
    }

    async fn approved_identity(&self, tx: &mut Transaction<'_, Postgres>, kind: &str, submission: &str) -> Result<(String,i64,String),StatusCode> {
        let candidate: Option<(String,i64,String)>=sqlx::query_as(&format!("SELECT content_id,base_version,change_note FROM {} WHERE kind=$1 AND submission_id=$2",self.t("content_candidates")))
            .bind(kind).bind(submission).fetch_optional(&mut **tx).await.map_err(db)?;
        Ok(candidate.unwrap_or_else(||(submission.into(),0,String::new())))
    }

    pub(crate) async fn publish_prompt_revision(&self, tx: &mut Transaction<'_, Postgres>, publication: &crate::Publication) -> Result<(),StatusCode> {
        let Some(mut item)=publication.square_item() else {return Ok(())};
        self.resolve_catalog_item(tx,&mut item).await?;
        let (id,base,note)=self.approved_identity(tx,&publication.kind,&publication.id).await?;
        // Legacy repeated approvals already have a v1 snapshot; do not invent a second revision.
        let legacy: bool=sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE kind=$1 AND content_id=$2 AND submission_id=$3)",self.t("content_versions")))
            .bind(&publication.kind).bind(&id).bind(format!("legacy:{}",publication.id)).fetch_one(&mut **tx).await.map_err(db)?;
        if legacy && base==0 {return Ok(())}
        item.id=id.clone();
        let mut snapshot=serde_json::to_value(&item).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        snapshot["content"]=serde_json::json!(item.content);
        snapshot["members"]=serde_json::json!(item.members);
        snapshot["asset_refs"]=serde_json::json!(publication.asset_refs);
        snapshot["cover"]=serde_json::json!(publication.cover);
        self.append_content_version(tx,&publication.kind,&id,publication.author_email.as_deref().unwrap_or("system:unowned"),&publication.id,base,&note,&snapshot).await?;
        if publication.author_email.is_none() {
            sqlx::query(&format!("UPDATE {} SET author_email=NULL WHERE kind=$1 AND content_id=$2",self.t("content_heads")))
                .bind(&publication.kind).bind(&id).execute(&mut **tx).await.map_err(db)?;
        }
        sqlx::query(&format!("INSERT INTO {}(id,title,kind,excerpt,model,member_count,content,category_id,members,sort_index)
          VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,COALESCE((SELECT max(sort_index)+1 FROM {0}),0))
          ON CONFLICT(id) DO UPDATE SET title=EXCLUDED.title,excerpt=EXCLUDED.excerpt,model=EXCLUDED.model,
            member_count=EXCLUDED.member_count,content=EXCLUDED.content,category_id=EXCLUDED.category_id,members=EXCLUDED.members",self.t("square_items")))
            .bind(&id).bind(&item.title).bind(&item.kind).bind(&item.excerpt).bind(&item.model).bind(item.member_count)
            .bind(&item.content).bind(&item.category_id).bind(serde_json::json!(item.members)).execute(&mut **tx).await.map_err(db)?;
        if base>0 {
            sqlx::query(&format!("DELETE FROM {} WHERE item_id=$1",self.t("prompt_translations"))).bind(&id).execute(&mut **tx).await.map_err(db)?;
        }
        Ok(())
    }

    pub(crate) async fn publish_skill_revision(&self, tx: &mut Transaction<'_, Postgres>, submission: &str) -> Result<(),StatusCode> {
        let row=sqlx::query(&format!("SELECT author_email,jsonb_build_object('title',title,'description',description,'category',category,'license',license,'bundle',bundle,'digest',digest) snapshot FROM {} WHERE id=$1",self.t("skill_publications")))
            .bind(submission).fetch_one(&mut **tx).await.map_err(db)?;
        let (id,base,note)=self.approved_identity(tx,"skill",submission).await?;
        let mut snapshot:Value=row.get("snapshot");snapshot["id"]=serde_json::json!(id);
        self.append_content_version(tx,"skill",&id,&row.get::<String,_>("author_email"),submission,base,&note,&snapshot).await?;
        if base>0 {
            sqlx::query(&format!("UPDATE {} p SET title=s.title,description=s.description,category=s.category,license=s.license,bundle=s.bundle,digest=s.digest FROM {0} s WHERE p.id=$1 AND s.id=$2",self.t("skill_publications")))
                .bind(&id).bind(submission).execute(&mut **tx).await.map_err(db)?;
        }
        Ok(())
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
    async fn author_updates_stay_pending_and_approval_keeps_identity_and_history() {
        use crate::admin_security_tests::request;
        let state=crate::admin_security_tests::state().await;
        let pg=state.db.as_ref().unwrap();
        pg.upsert_account("author@test.local",Some("password"),"owner").await.unwrap();
        pg.put_profile("author@test.local",Some("Author"),None).await.unwrap();
        let token=state.issue_session("author@test.local".into()).await.unwrap().access_token;
        let (status,first)=request(&state,"POST","/v1/publications",&token,json!({"source_id":"local","title":"Original","content":"v1"})).await;
        assert_eq!(status,StatusCode::OK);
        let id=first["id"].as_str().unwrap();
        pg.review_publication(id,"approved",None,None).await.unwrap();
        pg.put_favorite("author@test.local",id).await.unwrap();
        let mut update=json!({"source_id":"local","title":"Updated","content":"v2","revision":{"content_id":id,"base_version":1,"change_note":"Improve","request_id":uuid::Uuid::new_v4().to_string()}});
        let (status,candidate)=request(&state,"POST","/v1/publications",&token,update.clone()).await;
        assert_eq!(status,StatusCode::OK);
        assert_eq!(state.get_item(id).await.unwrap().unwrap().content.as_deref(),Some("v1"));
        assert_eq!(request(&state,"POST","/v1/publications",&token,update.clone()).await.1["id"],candidate["id"]);
        update["content"]=json!("different retry");
        assert_eq!(request(&state,"POST","/v1/publications",&token,update.clone()).await.0,StatusCode::CONFLICT);
        pg.review_publication(candidate["id"].as_str().unwrap(),"approved",None,None).await.unwrap();
        assert_eq!(state.get_item(id).await.unwrap().unwrap().content.as_deref(),Some("v2"));
        assert!(state.get_item(candidate["id"].as_str().unwrap()).await.unwrap().is_none());
        assert_eq!(pg.favorite_ids("author@test.local").await.unwrap(),vec![id]);
        let path=format!("/v1/content/prompt/{id}/versions/1");
        let old=request(&state,"GET",&path,"",json!(null)).await;
        assert_eq!(old.0,StatusCode::OK);assert_eq!(old.1["snapshot"]["content"],"v1");
        update["revision"]["request_id"]=json!(uuid::Uuid::new_v4().to_string());
        assert_eq!(request(&state,"POST","/v1/publications",&token,update).await.0,StatusCode::CONFLICT);
        sqlx::query(&format!("UPDATE {} SET visibility='offline' WHERE id=$1",pg.t("square_items"))).bind(id).execute(&pg.pool).await.unwrap();
        assert_eq!(request(&state,"GET",&path,"",json!(null)).await.0,StatusCode::NOT_FOUND);
    }

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

async fn visible(state: &crate::AppState, headers: &axum::http::HeaderMap, kind: &str, id: &str) -> Result<(),StatusCode> {
    if !matches!(kind,"prompt"|"collection"|"skill") {return Err(StatusCode::NOT_FOUND)}
    let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    if !state.square_public().await? {crate::require_user(state,headers).await?;}
    let public:bool=if kind=="skill" {
        sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} p JOIN {} a ON p.author_email=a.email WHERE p.id=$1 AND p.status='approved' AND NOT a.disabled AND NOT EXISTS(SELECT 1 FROM {} c WHERE c.kind='skill' AND c.submission_id=p.id))",pg.t("skill_publications"),pg.t("accounts"),pg.t("content_candidates")))
            .bind(id).fetch_one(&pg.pool).await.map_err(db)?
    }else {state.get_item(id).await?.is_some_and(|item|item.kind==kind)};
    if !public {return Err(StatusCode::NOT_FOUND)}
    Ok(())
}

pub async fn history(axum::extract::State(state): axum::extract::State<crate::AppState>, headers: axum::http::HeaderMap,
    axum::extract::Path((kind,id)): axum::extract::Path<(String,String)>) -> Result<axum::Json<Value>,StatusCode> {
    visible(&state,&headers,&kind,&id).await?;
    let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let value:Option<Value>=sqlx::query_scalar(&format!("SELECT jsonb_build_object('id',h.content_id,'kind',h.kind,'current_version',h.current_version,'versions',
        COALESCE((SELECT jsonb_agg(jsonb_build_object('version',v.version,'change_note',v.change_note,'published_at',v.published_at) ORDER BY v.version DESC) FROM (SELECT * FROM {} WHERE kind=h.kind AND content_id=h.content_id ORDER BY version DESC LIMIT 100) v),'[]'::jsonb))
        FROM {} h WHERE h.kind=$1 AND h.content_id=$2",pg.t("content_versions"),pg.t("content_heads")))
        .bind(&kind).bind(&id).fetch_optional(&pg.pool).await.map_err(db)?;
    Ok(axum::Json(value.ok_or(StatusCode::NOT_FOUND)?))
}

pub async fn snapshot(axum::extract::State(state): axum::extract::State<crate::AppState>, headers: axum::http::HeaderMap,
    axum::extract::Path((kind,id,version)): axum::extract::Path<(String,String,i64)>) -> Result<axum::Json<Value>,StatusCode> {
    visible(&state,&headers,&kind,&id).await?;
    let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let value:Option<Value>=sqlx::query_scalar(&format!("SELECT jsonb_build_object('id',content_id,'kind',kind,'version',version,'change_note',change_note,'published_at',published_at,'snapshot',snapshot) FROM {} WHERE kind=$1 AND content_id=$2 AND version=$3",pg.t("content_versions")))
        .bind(&kind).bind(&id).bind(version).fetch_optional(&pg.pool).await.map_err(db)?;
    Ok(axum::Json(value.ok_or(StatusCode::NOT_FOUND)?))
}
