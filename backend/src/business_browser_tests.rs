//! Explicit local integration gate; never linked into the production router.
use super::*;
use axum::{extract::State, routing::get};
use serde_json::{json, Value};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires local PostgreSQL, MinIO and installed Playwright Chrome"]
async fn real_business_browser_roundtrip() {
    let url = std::env::var("PROMPTARK_DATABASE_URL").unwrap_or_else(|_| "postgres://pl:pl@127.0.0.1:5432/promptark?sslmode=disable".into());
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    let schema = format!("business_{}", Uuid::new_v4().simple());
    let mut state = AppState::from_pool(pool.clone(), &schema).await.unwrap();
    state.media = media::MediaConfig::from_env();
    assert!(state.media.as_ref().unwrap().ping().await, "local private MinIO required");
    let pg = state.db.as_ref().unwrap();
    pg.initialize_admin(Some("owner@business.test"), Some("Business-only-password"), false).await.unwrap();
    let secret = oauth_admin::seal(&state.oauth_config.key, "smtp", "fixture-only").unwrap();
    sqlx::query(&format!("UPDATE {} SET data=$1 WHERE id=1", pg.t("mail_configuration")))
        .bind(json!({"revision":1,"enabled":true,"host":"smtp.invalid","port":465,"tls":"implicit","from":"test@business.test","username":"test","encrypted_secret":secret})).execute(&pool).await.unwrap();
    // Deliberately no mail worker: verify real outbox/challenges without sending email.
    let probe_key = Uuid::new_v4().to_string();
    let expected = probe_key.clone();
    let probe = Router::new().route("/__test/code", get(move |State(state): State<AppState>, headers: HeaderMap| {
        let expected = expected.clone();
        async move {
            if headers.get("x-business-test").and_then(|v|v.to_str().ok()) != Some(expected.as_str()) { return Err(StatusCode::NOT_FOUND); }
            let pg=state.db.as_ref().unwrap();
            let row:Value=sqlx::query_scalar(&format!("SELECT to_jsonb(m) FROM {} m WHERE recipient='reader@business.test' ORDER BY created_at DESC LIMIT 1",pg.t("mail_outbox"))).fetch_one(&pg.pool).await.map_err(|_|StatusCode::NOT_FOUND)?;
            let raw=oauth_admin::unseal(&state.oauth_config.key,&format!("mail:{}",row["id"].as_str().unwrap()),row["payload"].as_str().unwrap()).unwrap();
            let value:Value=serde_json::from_str(&raw).unwrap();
            let code=value["body"].as_str().unwrap().split("验证码：\n").nth(1).unwrap().lines().next().unwrap();
            Ok::<_,StatusCode>(Json(json!({"code":code})))
        }
    })).with_state(state.clone());
    let router=app(state.clone()).merge(probe).layer(middleware::from_fn(|request:Request,next:Next|async move {
        let origin=request.headers().get("origin").cloned();
        let mut response=next.run(request).await;
        if let Some(origin)=origin.filter(|v| matches!(v.to_str().ok(),Some("http://127.0.0.1:1433"|"http://127.0.0.1:1434"))) {
            response.headers_mut().insert("access-control-allow-origin",origin);
        }
        response
    }));
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin=format!("http://{}",listener.local_addr().unwrap());
    let server=tokio::spawn(async move {axum::serve(listener,router.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap()});
    let result=tokio::task::spawn_blocking(move || std::process::Command::new("node")
        .arg("../scripts/business-browser-smoke.mjs").env("PROMPTARK_BUSINESS_API",origin).env("PROMPTARK_BUSINESS_PROBE",probe_key).status()).await;
    server.abort();
    // Scope cleanup to the random schema and the media records created by this test.
    let keys:Vec<String>=sqlx::query_scalar(&format!("SELECT object_key FROM {}",pg.t("media_objects"))).fetch_all(&pool).await.unwrap();
    for key in keys {media::delete_reclaimed_object(state.media.as_ref().unwrap(),&key).await.unwrap();}
    assert!(schema.starts_with("business_") && schema.len()==41);
    sqlx::query(&format!("DROP SCHEMA \"{schema}\" CASCADE")).execute(&pool).await.unwrap();
    assert!(result.unwrap().unwrap().success(),"business browser failed; see output/playwright/business-*");
}
