use crate::{postgres::Pg, AppState, admin_risk::{actor_lock, audit}};
use axum::{extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Postgres, Transaction};
fn db_error(_: sqlx::Error) -> StatusCode { StatusCode::SERVICE_UNAVAILABLE }

impl Pg {
    pub async fn init_ai_jobs(&self) -> Result<(), sqlx::Error> {
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (publication_id TEXT PRIMARY KEY REFERENCES {}(id),status TEXT NOT NULL DEFAULT 'queued',attempts INT NOT NULL DEFAULT 0,claim TEXT,lease_until TIMESTAMPTZ,next_at TIMESTAMPTZ NOT NULL DEFAULT now(),updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),error TEXT)",self.t("ai_jobs"),self.t("publications"))).execute(&self.pool).await?;
        Ok(())
    }
    pub(crate) async fn claim_ai_job(&self) -> Result<Option<(String,String)>,StatusCode> {
        let mut tx=self.pool.begin().await.map_err(db_error)?;
        sqlx::query(&format!("UPDATE {} j SET status='cancelled',claim=NULL,error='投稿已由其他流程处置',updated_at=now() FROM {} p WHERE p.id=j.publication_id AND p.status<>'pending' AND j.status IN ('queued','running','failed')",self.t("ai_jobs"),self.t("publications"))).execute(&mut *tx).await.map_err(db_error)?;
        sqlx::query(&format!("UPDATE {} SET status='failed',claim=NULL,error='执行中断且已达到三次上限',updated_at=now() WHERE status='running' AND lease_until<now() AND attempts>=3",self.t("ai_jobs"))).execute(&mut *tx).await.map_err(db_error)?;
        let id:Option<String>=sqlx::query_scalar(&format!("SELECT publication_id FROM {} WHERE attempts<3 AND ((status IN ('queued','failed') AND next_at<=now()) OR (status='running' AND lease_until<now())) ORDER BY next_at FOR UPDATE SKIP LOCKED LIMIT 1",self.t("ai_jobs"))).fetch_optional(&mut *tx).await.map_err(db_error)?;
        let Some(id)=id else { tx.commit().await.map_err(db_error)?; return Ok(None) };
        let claim=uuid::Uuid::new_v4().to_string();
        sqlx::query(&format!("UPDATE {} SET status='running',claim=$2,attempts=attempts+1,lease_until=now()+interval '90 seconds',updated_at=now(),error=NULL WHERE publication_id=$1",self.t("ai_jobs"))).bind(&id).bind(&claim).execute(&mut *tx).await.map_err(db_error)?;
        tx.commit().await.map_err(db_error)?; Ok(Some((id,claim)))
    }
}
pub(crate) async fn lock_claim(pg:&Pg,tx:&mut Transaction<'_,Postgres>,id:&str,claim:&str)->Result<(),StatusCode>{
    let found:Option<String>=sqlx::query_scalar(&format!("SELECT publication_id FROM {} WHERE publication_id=$1 AND claim=$2 AND status='running' AND lease_until>now() FOR UPDATE",pg.t("ai_jobs"))).bind(id).bind(claim).fetch_optional(&mut **tx).await.map_err(db_error)?;
    if found.is_none(){return Err(StatusCode::CONFLICT)} Ok(())
}
pub(crate) async fn finish(pg:&Pg,tx:&mut Transaction<'_,Postgres>,id:&str,claim:&str,error:Option<&str>)->Result<(),StatusCode>{
    lock_claim(pg,tx,id,claim).await?;
    sqlx::query(&format!("UPDATE {} SET status=$3,error=$4,claim=NULL,lease_until=NULL,next_at=now()+attempts*interval '30 seconds',updated_at=now() WHERE publication_id=$1 AND claim=$2",pg.t("ai_jobs"))).bind(id).bind(claim).bind(if error.is_some(){"failed"}else{"completed"}).bind(error).execute(&mut **tx).await.map_err(db_error)?; Ok(())
}
pub async fn run_one(state:&AppState)->Result<bool,StatusCode>{
    let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let Some((id,claim))=pg.claim_ai_job().await? else {return Ok(false)};
    let row=sqlx::query(&format!("SELECT * FROM {} WHERE id=$1",pg.t("publications"))).bind(&id).fetch_one(&pg.pool).await.map_err(db_error)?;
    let publication=Pg::publication_from_row(&row);
    if crate::admin_ai::screen_publication(state,&publication,Some(&claim)).await.is_err(){
        let mut tx=pg.pool.begin().await.map_err(db_error)?;
        finish(pg,&mut tx,&id,&claim,Some("审核依赖不可用或执行中断，可重试；投稿仍保留")).await?;
        tx.commit().await.map_err(db_error)?;
    }
    Ok(true)
}
#[derive(Deserialize,Default)]
pub struct Page { #[serde(default)] offset:usize }
pub async fn list(State(state):State<AppState>,headers:HeaderMap,Query(page):Query<Page>)->Result<Json<Value>,StatusCode>{
    crate::require_configuration_admin(&state,&headers).await?;
    let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let items:Vec<Value>=sqlx::query_scalar(&format!("SELECT jsonb_build_object('id',publication_id,'status',status,'attempts',attempts,'error',error,'updated_at',updated_at) FROM {} ORDER BY updated_at DESC,publication_id LIMIT 25 OFFSET $1",pg.t("ai_jobs"))).bind(page.offset.min(100000) as i64).fetch_all(&pg.pool).await.map_err(db_error)?;
    let total:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {}",pg.t("ai_jobs"))).fetch_one(&pg.pool).await.map_err(db_error)?;
    crate::require_configuration_admin(&state,&headers).await?;
    Ok(Json(json!({"items":items,"total":total})))
}
pub async fn retry(State(state):State<AppState>,headers:HeaderMap,Path(id):Path<String>)->Result<Json<Value>,StatusCode>{
    let actor=crate::require_configuration_admin(&state,&headers).await?;
    let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx=pg.pool.begin().await.map_err(db_error)?;
    actor_lock(pg,&mut tx,&actor,&crate::bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,true).await?;
    let role:String=sqlx::query_scalar(&format!("SELECT role FROM {} WHERE email=$1",pg.t("accounts"))).bind(&actor).fetch_one(&mut *tx).await.map_err(db_error)?;
    if role!="owner" && pg.has_owner().await? {return Err(StatusCode::FORBIDDEN)}
    let changed=sqlx::query(&format!("UPDATE {} j SET status='queued',next_at=now(),updated_at=now() WHERE publication_id=$1 AND j.status='failed' AND attempts<3 AND EXISTS(SELECT 1 FROM {} p WHERE p.id=j.publication_id AND p.status='pending')",pg.t("ai_jobs"),pg.t("publications"))).bind(&id).execute(&mut *tx).await.map_err(db_error)?;
    if changed.rows_affected()!=1{return Err(StatusCode::CONFLICT)}
    audit(pg,&mut tx,&actor,"ai_job_retry",json!({"publication_id":id})).await?;
    tx.commit().await.map_err(db_error)?; Ok(Json(json!({"queued":true})))
}
impl AppState {
    pub fn start_ai_worker(&self){let state=self.clone();tokio::spawn(async move{loop{let _=run_one(&state).await;tokio::time::sleep(std::time::Duration::from_secs(2)).await;}});}
}
