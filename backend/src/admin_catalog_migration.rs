use crate::{
    admin_risk::{actor_lock, audit},
    bearer_token,
    postgres::Pg,
    require_admin, AppState, SquareItem,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Postgres, Transaction};
fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Migration {
    target: String,
    revision: i64,
    target_revision: i64,
    reason: String,
}
impl Pg {
    pub async fn init_catalog_redirects(&self) -> Result<(), sqlx::Error> {
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (kind TEXT NOT NULL,source TEXT NOT NULL,target TEXT NOT NULL,CHECK(source<>target),PRIMARY KEY(kind,source))",self.t("catalog_redirects"))).execute(&self.pool).await?;
        Ok(())
    }
    pub(crate) async fn resolve_catalog_item(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item: &mut SquareItem,
    ) -> Result<bool, StatusCode> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(&format!(
            "SELECT kind,source,target FROM {}",
            self.t("catalog_redirects")
        ))
        .fetch_all(&mut **tx)
        .await
        .map_err(db_error)?;
        let mut changed = false;
        for (kind, source, target) in rows {
            let field = if kind == "categories" {
                &mut item.category_id
            } else {
                &mut item.model
            };
            if field.as_deref() == Some(&source) {
                *field = Some(target.clone());
                changed = true
            }
            for member in &mut item.members {
                let field = if kind == "categories" {
                    &mut member.category_id
                } else {
                    &mut member.model
                };
                if field.as_deref() == Some(&source) {
                    *field = Some(target.clone());
                    changed = true
                }
            }
        }
        Ok(changed)
    }
}
pub async fn migrate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
    Json(input): Json<Migration>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    if !matches!(kind.as_str(), "categories" | "models") {
        return Err(StatusCode::NOT_FOUND);
    }
    if input.target == id
        || input.target.is_empty()
        || input.target.chars().count() > 100
        || input.reason.trim().is_empty()
        || input.reason.chars().count() > 1000
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    actor_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
        true,
    )
    .await?;
    pg.catalog_lock(&mut tx).await?;
    let source: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} WHERE kind=$1 AND id=$2 AND NOT deleted",
        pg.t("catalog")
    ))
    .bind(&kind)
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_error)?
    .ok_or(StatusCode::NOT_FOUND)?;
    let target: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} WHERE kind=$1 AND id=$2 AND NOT deleted",
        pg.t("catalog")
    ))
    .bind(&kind)
    .bind(&input.target)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_error)?
    .ok_or(StatusCode::NOT_FOUND)?;
    if source["revision"] != input.revision || target["revision"] != input.target_revision {
        return Err(StatusCode::CONFLICT);
    }
    if target["enabled"] != true {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut children = 0;
    if kind == "categories" {
        if source["parent_id"].is_null() && !target["parent_id"].is_null() {
            return Err(StatusCode::BAD_REQUEST);
        }
        if let Some(parent) = target["parent_id"].as_str() {
            let valid:bool=sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE kind='categories' AND id=$1 AND NOT deleted AND (data->>'enabled')::boolean)",pg.t("catalog"))).bind(parent).fetch_one(&mut *tx).await.map_err(db_error)?;
            if parent == id || !valid {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        let skills:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM (SELECT data FROM {} ORDER BY revision DESC LIMIT 1) c,jsonb_array_elements(c.data->'skills') s WHERE s->'categories' ? $1",pg.t("ai_configuration"))).bind(&id).fetch_one(&mut *tx).await.map_err(db_error)?;
        if skills > 0 {
            return Ok(Json(
                json!({"migrated":false,"blocked":"skills","message":"审核 Skill 仍引用此分类，请所有者先在审核 Skills 中更换适用分类。"}),
            ));
        }
        let conflicts:bool=sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {0} a JOIN {0} b ON lower(a.data->>'name')=lower(b.data->>'name') WHERE a.kind='categories' AND b.kind='categories' AND NOT a.deleted AND NOT b.deleted AND a.data->>'parent_id'=$1 AND b.data->>'parent_id'=$2)",pg.t("catalog"))).bind(&id).bind(&input.target).fetch_one(&mut *tx).await.map_err(db_error)?;
        if conflicts {
            return Ok(Json(
                json!({"migrated":false,"blocked":"children","message":"目标下有同名子分类，请先改名或分别迁移这些子分类。"}),
            ));
        }
        children=sqlx::query(&format!("UPDATE {} SET data=jsonb_set(jsonb_set(data,'{{parent_id}}',to_jsonb($2::text)),'{{revision}}',to_jsonb((data->>'revision')::bigint+1)) WHERE kind='categories' AND NOT deleted AND data->>'parent_id'=$1",pg.t("catalog"))).bind(&id).bind(&input.target).execute(&mut *tx).await.map_err(db_error)?.rows_affected();
    }
    let field = if kind == "categories" {
        "category_id"
    } else {
        "model"
    };
    let content=sqlx::query(&format!("UPDATE {} s SET {field}=CASE WHEN {field}=$1 THEN $2 ELSE {field} END,members=COALESCE((SELECT jsonb_agg(CASE WHEN m->>'{field}'=$1 THEN jsonb_set(m,'{{{field}}}',to_jsonb($2::text)) ELSE m END ORDER BY ordinal) FROM jsonb_array_elements(s.members) WITH ORDINALITY AS e(m,ordinal)),'[]'::jsonb),revision=revision+1 WHERE {field}=$1 OR EXISTS(SELECT 1 FROM jsonb_array_elements(members) m WHERE m->>'{field}'=$1)",pg.t("square_items"))).bind(&id).bind(&input.target).execute(&mut *tx).await.map_err(db_error)?.rows_affected();
    sqlx::query(&format!(
        "UPDATE {} SET target=$3 WHERE kind=$1 AND target=$2",
        pg.t("catalog_redirects")
    ))
    .bind(&kind)
    .bind(&id)
    .bind(&input.target)
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    sqlx::query(&format!(
        "INSERT INTO {} (kind,source,target) VALUES ($1,$2,$3)",
        pg.t("catalog_redirects")
    ))
    .bind(&kind)
    .bind(&id)
    .bind(&input.target)
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    sqlx::query(&format!("UPDATE {} SET deleted=TRUE,data=jsonb_set(data,'{{revision}}',to_jsonb((data->>'revision')::bigint+1)) WHERE kind=$1 AND id=$2",pg.t("catalog"))).bind(&kind).bind(&id).execute(&mut *tx).await.map_err(db_error)?;
    audit(pg,&mut tx,&actor,"catalog_migrated",json!({"kind":kind,"source":id,"target":input.target,"reason":input.reason.trim(),"count":content,"children":children})).await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(
        json!({"migrated":true,"content":content,"children":children,"target":input.target}),
    ))
}
