use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::{json, Value};
async fn fixture() -> (AppState, String, String) {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("owner@oauth.test", Some("test-password"), "owner")
        .await
        .unwrap();
    pg.upsert_account("admin@oauth.test", Some("test-password"), "admin")
        .await
        .unwrap();
    let owner = state
        .issue_session("owner@oauth.test".into())
        .await
        .unwrap()
        .access_token;
    let admin = state
        .issue_session("admin@oauth.test".into())
        .await
        .unwrap()
        .access_token;
    (state, owner, admin)
}
fn config() -> Value {
    json!({"revision":0,"current_password":"test-password","enabled":true,"client_id":"synthetic-client","client_secret":"synthetic-secret","redirect_uri":"http://localhost:8787/v1/session/oauth/callback"})
}
fn nonce(value: &Value) -> String {
    url::Url::parse(value["authorization_url"].as_str().unwrap())
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned()
}
#[tokio::test]
async fn oauth_saves_require_owner_password_version_and_atomic_audit() {
    let (state, owner, admin) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    assert_eq!(
        request(&state, "PUT", "/v1/admin/oauth/google", &admin, config())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let mut bad = config();
    bad["current_password"] = json!("wrong");
    assert_eq!(
        request(&state, "PUT", "/v1/admin/oauth/google", &owner, bad)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let (status, saved) = request(&state, "PUT", "/v1/admin/oauth/google", &owner, config()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["revision"], 1);
    assert_eq!(saved["verification"]["status"], "unverified");
    assert!(!saved.to_string().contains("synthetic-secret"));
    assert_eq!(
        request(&state, "PUT", "/v1/admin/oauth/google", &owner, config())
            .await
            .0,
        StatusCode::CONFLICT
    );
    let started = request(
        &state,
        "POST",
        "/v1/admin/oauth/google/verify",
        &owner,
        json!({}),
    )
    .await
    .1;
    let state_token = nonce(&started);
    let mut next = config();
    next["revision"] = json!(1);
    next["client_secret"] = json!("");
    next["client_id"] = json!("changed-client");
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/oauth/google",
            &owner,
            next.clone()
        )
        .await
        .1["revision"],
        2
    );
    assert_eq!(
        request(
            &state,
            "GET",
            &format!("/v1/session/oauth/callback?state={state_token}&code=old"),
            "",
            json!({})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT oauth_audit_fail CHECK(action<>'oauth_configuration_saved') NOT VALID",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    next["revision"] = json!(2);
    next["client_id"] = json!("must-not-save");
    assert_eq!(
        request(&state, "PUT", "/v1/admin/oauth/google", &owner, next)
            .await
            .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(state.oauth_revision("google").await.unwrap(), 2);
    assert_eq!(
        state
            .runtime_provider("google")
            .await
            .unwrap()
            .unwrap()
            .client_id,
        "changed-client"
    );
}
#[tokio::test]
async fn oauth_verification_rejects_expiry_replay_mock_and_revoked_owner() {
    let (mut state, owner, admin) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    request(&state, "PUT", "/v1/admin/oauth/google", &owner, config()).await;
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/oauth/google/verify",
            &admin,
            json!({})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    state.oauth.mock_users.insert(
        "mock-code".into(),
        oauth::OAuthUser {
            provider: "google".into(),
            provider_uid: "fake".into(),
            email: "fake@test.test".into(),
        },
    );
    let started = request(
        &state,
        "POST",
        "/v1/admin/oauth/google/verify",
        &owner,
        json!({}),
    )
    .await
    .1;
    let code = nonce(&started);
    let path = format!("/v1/session/oauth/callback?state={code}&code=mock-code");
    assert_eq!(
        request(&state, "GET", &path, "", json!({})).await.0,
        StatusCode::OK
    );
    assert_eq!(
        state.oauth_verification_view("google").await.unwrap()["status"],
        "failed"
    );
    assert_eq!(
        request(&state, "GET", &path, "", json!({})).await.0,
        StatusCode::BAD_REQUEST
    );
    let started = request(
        &state,
        "POST",
        "/v1/admin/oauth/google/verify",
        &owner,
        json!({}),
    )
    .await
    .1;
    let code = nonce(&started);
    sqlx::query(&format!(
        "UPDATE {} SET expires_at=now()-interval '1 second'",
        pg.t("oauth_verifications")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        state.oauth_verification_view("google").await.unwrap()["status"],
        "expired"
    );
    assert_eq!(
        request(
            &state,
            "GET",
            &format!("/v1/session/oauth/callback?state={code}&code=mock-code"),
            "",
            json!({})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let started = request(
        &state,
        "POST",
        "/v1/admin/oauth/google/verify",
        &owner,
        json!({}),
    )
    .await
    .1;
    let code = nonce(&started);
    sqlx::query(&format!(
        "UPDATE {} SET role='admin' WHERE email='owner@oauth.test'",
        pg.t("accounts")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "GET",
            &format!("/v1/session/oauth/callback?state={code}&error=denied"),
            "",
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    let result = state.oauth_verification_view("google").await.unwrap();
    assert_eq!(result["status"], "failed");
    assert!(result["message"].as_str().unwrap().contains("权限"));
}
#[tokio::test]
async fn oauth_real_exchange_records_success_without_creating_or_linking_accounts() {
    use axum::{
        routing::{get, post},
        Router,
    };
    let (mut state, owner, _) = fixture().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener,Router::new().route("/token",post(||async{Json(json!({"access_token":"private-provider-token"}))})).route("/profile",get(||async{Json(json!({"sub":"external-id","email":"external@example.com","email_verified":true}))}))).await.unwrap()
    });
    let mut provider = oauth::ProviderConfig::configured(
        "google",
        "test-client".into(),
        "test-secret".into(),
        "http://localhost:8787/v1/session/oauth/callback".into(),
    );
    provider.token_uri = format!("{base}/token");
    provider.user_info_uri = format!("{base}/profile");
    state.oauth.providers.insert("google".into(), provider);
    let started = request(
        &state,
        "POST",
        "/v1/admin/oauth/google/verify",
        &owner,
        json!({}),
    )
    .await
    .1;
    let code = nonce(&started);
    let response = oauth_verification::callback(
        &state,
        &oauth::OAuthCallbackQuery {
            state: Some(code),
            code: Some("valid-code".into()),
            error: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(response.headers()["cache-control"], "no-store");
    let body = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("验证成功"));
    assert!(!text.contains("external@example.com"));
    assert!(!text.contains("private-provider-token"));
    assert_eq!(
        state.oauth_verification_view("google").await.unwrap()["status"],
        "succeeded"
    );
    let pg = state.db.as_ref().unwrap();
    let accounts: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {}", pg.t("accounts")))
        .fetch_one(&pg.pool)
        .await
        .unwrap();
    let links: i64 =
        sqlx::query_scalar(&format!("SELECT count(*) FROM {}", pg.t("oauth_accounts")))
            .fetch_one(&pg.pool)
            .await
            .unwrap();
    assert_eq!(accounts, 2);
    assert_eq!(links, 0);
    state
        .oauth
        .providers
        .get_mut("google")
        .unwrap()
        .client_secret = "rotated".into();
    assert_eq!(
        state.oauth_verification_view("google").await.unwrap()["status"],
        "configuration_changed"
    );
    server.abort();
}

#[tokio::test]
async fn provider_failures_and_oversized_responses_are_not_success() {
    use axum::{
        response::IntoResponse,
        routing::{get, post},
        Router,
    };
    for mode in ["denied", "oversized", "unverified"] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener,Router::new().route("/token",post(move||async move{if mode=="denied"{(StatusCode::FORBIDDEN,Json(json!({"access_token":"must-not-use"}))).into_response()}else if mode=="oversized"{Json(json!({"access_token":"x".repeat(270000)})).into_response()}else{Json(json!({"access_token":"token"})).into_response()}})).route("/profile",get(||async{Json(json!({"sub":"uid","email":"user@example.com","email_verified":false}))}))).await.unwrap()
        });
        let mut config = oauth::ProviderConfig::configured(
            "google",
            "id".into(),
            "secret".into(),
            "http://localhost:8787/v1/session/oauth/callback".into(),
        );
        config.token_uri = format!("{base}/token");
        config.user_info_uri = format!("{base}/profile");
        assert!(oauth::fetch_verified_user(&config, "google", "code")
            .await
            .is_err());
        server.abort();
    }
}
