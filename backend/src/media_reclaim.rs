use crate::{admin_risk::{actor_lock, audit}, postgres::Pg, AppState};
use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Postgres, Row, Transaction};

fn db_error(_: sqlx::Error) -> StatusCode { StatusCode::SERVICE_UNAVAILABLE }
pub(crate) async fn reference_lock(pg: &Pg, tx: &mut Transaction<'_, Postgres>) -> Result<(), StatusCode> {
    sqlx::query("SET LOCAL lock_timeout='5s'").execute(&mut **tx).await.map_err(db_error)?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{}:media-references",pg.schema)).execute(&mut **tx).await.map_err(db_error)?;
    Ok(())
}
async fn configuration_lock(pg: &Pg, tx: &mut Transaction<'_, Postgres>, actor: &str, token: &str) -> Result<(), StatusCode> {
    sqlx::query("SET LOCAL lock_timeout='5s'").execute(&mut **tx).await.map_err(db_error)?;
    actor_lock(pg,tx,actor,token,true).await?;
    let role: String=sqlx::query_scalar(&format!("SELECT role FROM {} WHERE email=$1",pg.t("accounts"))).bind(actor).fetch_one(&mut **tx).await.map_err(db_error)?;
    if role != "owner" && pg.has_owner().await? { return Err(StatusCode::FORBIDDEN); }
    Ok(())
}
fn eligible(pg: &Pg) -> String {
    format!(r#"m.ready AND m.file_name IS NOT NULL AND m.content_type IS NOT NULL AND m.size IS NOT NULL AND m.sha256 IS NOT NULL
        AND m.id ~ '^media\.[0-9a-f]{{8}}(-[0-9a-f]{{4}}){{3}}-[0-9a-f]{{12}}$' AND m.object_key='promptark/'||m.id
        AND (m.deleting OR m.last_used_at < now()-interval '7 days')
        AND NOT EXISTS (SELECT 1 FROM {} l, jsonb_array_elements(COALESCE(l.payload::jsonb->'asset_refs','[]'::jsonb)) r WHERE r->>'media_id'=m.id)
        AND NOT EXISTS (SELECT 1 FROM {} p, jsonb_array_elements(p.asset_refs) r WHERE r->>'media_id'=m.id)"#,pg.t("library_changes"),pg.t("publications"))
}
pub async fn list(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    crate::require_configuration_admin(&state,&headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut items: Vec<Value> = tokio::time::timeout(std::time::Duration::from_secs(5),sqlx::query_scalar(&format!(
        "SELECT jsonb_build_object('id',m.id,'name',m.file_name,'size',m.size,'deleting',m.deleting,'last_used_at',m.last_used_at) FROM {} m WHERE {} ORDER BY m.deleting DESC,m.last_used_at,m.id LIMIT 26",pg.t("media_objects"),eligible(pg))).fetch_all(&pg.pool))
        .await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?.map_err(db_error)?;
    let more=items.len()>25;items.truncate(25);
    crate::require_configuration_admin(&state,&headers).await?;
    Ok(Json(json!({"items":items,"more":more})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Confirm { confirm: bool }

pub async fn purge(State(state): State<AppState>, Path(id): Path<String>, headers: HeaderMap, Json(input): Json<Confirm>) -> Result<Json<Value>, StatusCode> {
    let actor=crate::require_configuration_admin(&state,&headers).await?;
    if !input.confirm { return Err(StatusCode::BAD_REQUEST); }
    let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let config=state.media.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let token=crate::bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let mut tx=pg.pool.begin().await.map_err(db_error)?;
    configuration_lock(pg,&mut tx,&actor,&token).await?;
    reference_lock(pg,&mut tx).await?;
    let row=sqlx::query(&format!("SELECT m.id,m.deleting FROM {} m WHERE m.id=$1 AND {} FOR UPDATE",pg.t("media_objects"),eligible(pg)))
        .bind(&id).fetch_optional(&mut *tx).await.map_err(db_error)?.ok_or(StatusCode::CONFLICT)?;
    if !row.get::<bool,_>("deleting") {
        sqlx::query(&format!("UPDATE {} SET deleting=TRUE WHERE id=$1",pg.t("media_objects"))).bind(&id).execute(&mut *tx).await.map_err(db_error)?;
        audit(pg,&mut tx,&actor,"media_reclaim_started",json!({"id":id})).await?;
    }
    tx.commit().await.map_err(db_error)?;
    // Keep the row locked during storage I/O, without blocking all reference writers.
    let mut tx=pg.pool.begin().await.map_err(db_error)?;
    configuration_lock(pg,&mut tx,&actor,&token).await?;
    let key: Option<String>=sqlx::query_scalar(&format!("SELECT object_key FROM {} WHERE id=$1 AND deleting FOR UPDATE NOWAIT",pg.t("media_objects")))
        .bind(&id).fetch_optional(&mut *tx).await.map_err(|_| StatusCode::CONFLICT)?;
    let Some(key)=key else { return Ok(Json(json!({"removed":true}))) };
    if key != format!("promptark/{id}") { return Err(StatusCode::CONFLICT); }
    crate::media::delete_reclaimed_object(config,&key).await?;
    sqlx::query(&format!("DELETE FROM {} WHERE id=$1 AND deleting",pg.t("media_objects"))).bind(&id).execute(&mut *tx).await.map_err(db_error)?;
    audit(pg,&mut tx,&actor,"media_reclaim_completed",json!({"id":id})).await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(json!({"removed":true})))
}
