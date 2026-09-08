use crate::{admin_security_tests::{state,request},ai_jobs};
use axum::http::StatusCode;
use serde_json::{json,Value};

#[tokio::test]
async fn schema_migration_preserves_jobs_and_explicit_reset_rebuilds_foreign_key(){
    let state=state().await;let pg=state.db.as_ref().unwrap();
    pg.upsert_account("reset@test.local",None,"user").await.unwrap();
    let token=state.issue_session("reset@test.local".into()).await.unwrap().access_token;
    sqlx::query(&format!("UPDATE {} SET data=$1 WHERE id=1",pg.t("moderation_policy"))).bind(json!({"enabled":true,"require_ai":true})).execute(&pg.pool).await.unwrap();
    assert_eq!(request(&state,"POST","/v1/publications",&token,json!({"source_id":"reset","title":"Reset test","content":"Safe"})).await.0,StatusCode::OK);
    pg.apply_schema(false).await.unwrap();
    let count:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {}",pg.t("ai_jobs"))).fetch_one(&pg.pool).await.unwrap();assert_eq!(count,1);
    pg.apply_schema(true).await.unwrap();
    let count:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {}",pg.t("ai_jobs"))).fetch_one(&pg.pool).await.unwrap();assert_eq!(count,0);
    assert!(sqlx::query(&format!("INSERT INTO {} (publication_id) VALUES ('missing')",pg.t("ai_jobs"))).execute(&pg.pool).await.is_err());
}

#[tokio::test]
async fn durable_ai_jobs_lease_recovery_retry_limit_and_human_priority(){
    let state=state().await;let pg=state.db.as_ref().unwrap();
    pg.upsert_account("worker@test.local",None,"owner").await.unwrap();
    let token=state.issue_session("worker@test.local".into()).await.unwrap().access_token;
    sqlx::query(&format!("UPDATE {} SET data=$1 WHERE id=1",pg.t("moderation_policy"))).bind(json!({"enabled":true,"require_ai":true,"check_structure":false,"check_duplicates":false,"check_sensitive":false})).execute(&pg.pool).await.unwrap();
    let (status,p)=request(&state,"POST","/v1/publications",&token,json!({"source_id":"job","title":"Queue","content":"Review me"})).await;
    assert_eq!(status,StatusCode::OK);let id=p["id"].as_str().unwrap();
    let (claimed,old)=pg.claim_ai_job().await.unwrap().unwrap();assert_eq!(claimed,id);
    assert!(pg.claim_ai_job().await.unwrap().is_none());
    sqlx::query(&format!("UPDATE {} SET lease_until=now()-interval '1 second'",pg.t("ai_jobs"))).execute(&pg.pool).await.unwrap();
    let (_,new)=pg.claim_ai_job().await.unwrap().unwrap();assert_ne!(old,new);
    let mut tx=pg.pool.begin().await.unwrap();assert_eq!(ai_jobs::finish(pg,&mut tx,id,&old,None).await.err(),Some(StatusCode::CONFLICT));tx.rollback().await.unwrap();
    let mut tx=pg.pool.begin().await.unwrap();ai_jobs::finish(pg,&mut tx,id,&new,Some("test failure")).await.unwrap();tx.commit().await.unwrap();
    assert_eq!(request(&state,"POST",&format!("/v1/admin/ai/jobs/{id}/retry"),&token,json!({})).await.0,StatusCode::OK);
    let (_,third)=pg.claim_ai_job().await.unwrap().unwrap();
    let mut tx=pg.pool.begin().await.unwrap();ai_jobs::finish(pg,&mut tx,id,&third,Some("exhausted")).await.unwrap();tx.commit().await.unwrap();
    assert!(pg.claim_ai_job().await.unwrap().is_none());
    assert_eq!(request(&state,"POST",&format!("/v1/admin/ai/jobs/{id}/retry"),&token,json!({})).await.0,StatusCode::CONFLICT);
    pg.review_publication(id,"rejected",Some("Human"),None).await.unwrap();pg.claim_ai_job().await.unwrap();
    let (_,jobs)=request(&state,"GET","/v1/admin/ai/jobs",&token,json!({})).await;
    assert_eq!(jobs["items"][0]["status"],"cancelled");
    assert_eq!(request(&state,"GET","/v1/admin/ai/jobs","",json!({})).await.0,StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn worker_records_manual_result_without_models_and_does_not_leave_running_job(){
    let state=state().await;let pg=state.db.as_ref().unwrap();pg.upsert_account("review@test.local",None,"user").await.unwrap();
    let token=state.issue_session("review@test.local".into()).await.unwrap().access_token;
    sqlx::query(&format!("UPDATE {} SET data=$1 WHERE id=1",pg.t("moderation_policy"))).bind(json!({"enabled":true,"require_ai":true,"check_duplicates":false,"check_sensitive":false})).execute(&pg.pool).await.unwrap();
    let (_,p)=request(&state,"POST","/v1/publications",&token,json!({"source_id":"manual","title":"Title","content":"Safe text"})).await;
    assert!(ai_jobs::run_one(&state).await.unwrap());
    let row:Value=sqlx::query_scalar(&format!("SELECT to_jsonb(j) FROM {} j WHERE publication_id=$1",pg.t("ai_jobs"))).bind(p["id"].as_str().unwrap()).fetch_one(&pg.pool).await.unwrap();
    assert_ne!(row["status"],"running");
    assert_eq!(request(&state,"GET","/v1/admin/ai/jobs",&token,json!({})).await.0,StatusCode::FORBIDDEN);
}
