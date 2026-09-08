use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::{json, Value};

#[tokio::test]
async fn email_notifications_reuse_encrypted_mail_and_show_actual_delivery_state() {
    let (state, owner, _) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    let mut settings = config();
    settings["webhook_enabled"] = json!(false);
    settings["email_enabled"] = json!(true);
    settings["recipient"] = json!("operations@example.com");
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/notifications/config",
            &owner,
            settings.clone()
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    let mail = json!({"revision":0,"enabled":true,"host":"smtp.example.com","port":465,"tls":"implicit","from":"noreply@example.com","username":"smtp-user","password":"synthetic-mail-password","current_password":"test-password"});
    assert_eq!(
        request(&state, "PUT", "/v1/admin/mail/config", &owner, mail)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/notifications/config",
            &owner,
            settings
        )
        .await
        .0,
        StatusCode::OK
    );
    let (status, queued) = request(
        &state,
        "POST",
        "/v1/admin/notifications/test",
        &owner,
        json!({"revision":1,"channel":"email"}),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(queued["status"], "mail_queued");
    let job = pg.claim_mail(true).await.unwrap().unwrap();
    assert!(pg.complete_mail(&job.id, &job.claim, Ok(())).await.unwrap());
    let (_, rows) = request(
        &state,
        "GET",
        "/v1/admin/notifications/deliveries",
        &owner,
        json!({}),
    )
    .await;
    assert_eq!(rows["items"][0]["status"], "accepted");
    assert_eq!(rows["items"][0]["attempts"], 1);
    assert!(!rows.to_string().contains("operations@example.com"));
    assert!(!rows.to_string().contains("payload"));
    let (_, detail) = request(
        &state,
        "GET",
        &format!(
            "/v1/admin/notifications/deliveries/{}",
            queued["id"].as_str().unwrap()
        ),
        &owner,
        json!({}),
    )
    .await;
    assert_eq!(detail["history"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn webhook_attempts_expiry_and_retention_are_bounded() {
    let (state, owner, _) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    request(
        &state,
        "PUT",
        "/v1/admin/notifications/config",
        &owner,
        config(),
    )
    .await;
    let (_, queued) = request(
        &state,
        "POST",
        "/v1/admin/notifications/test",
        &owner,
        json!({"revision":1,"channel":"webhook"}),
    )
    .await;
    let id = queued["id"].as_str().unwrap();
    for attempt in 1..=3 {
        let job = pg.claim_notification().await.unwrap().unwrap();
        assert!(pg
            .complete_notification(id, job["claim"].as_str().unwrap(), Err("模拟超时".into()))
            .await
            .unwrap());
        sqlx::query(&format!(
            "UPDATE {} SET next_at=now()-interval '1 second' WHERE id=$1",
            pg.t("notification_deliveries")
        ))
        .bind(id)
        .execute(&pg.pool)
        .await
        .unwrap();
        let (_, rows) = request(
            &state,
            "GET",
            "/v1/admin/notifications/deliveries",
            &owner,
            json!({}),
        )
        .await;
        assert_eq!(rows["items"][0]["attempts"], attempt);
    }
    assert!(pg.claim_notification().await.unwrap().is_none());
    let (_, rows) = request(
        &state,
        "GET",
        "/v1/admin/notifications/deliveries",
        &owner,
        json!({}),
    )
    .await;
    assert_eq!(
        request(
            &state,
            "POST",
            &format!("/v1/admin/notifications/deliveries/{id}/retry"),
            &owner,
            json!({"revision":rows["items"][0]["revision"]})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    sqlx::query(&format!(
        "UPDATE {} SET expires_at=now()-interval '1 second' WHERE id=$1",
        pg.t("notification_deliveries")
    ))
    .bind(id)
    .execute(&pg.pool)
    .await
    .unwrap();
    assert!(pg.claim_notification().await.unwrap().is_none());
    let (_, rows) = request(
        &state,
        "GET",
        "/v1/admin/notifications/deliveries?status=expired",
        &owner,
        json!({}),
    )
    .await;
    assert_eq!(rows["total"], 1);
}
async fn fixture() -> (AppState, String, String) {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("owner@notify.test", Some("test-password"), "owner")
        .await
        .unwrap();
    pg.upsert_account("admin@notify.test", Some("test-password"), "admin")
        .await
        .unwrap();
    let owner = state
        .issue_session("owner@notify.test".into())
        .await
        .unwrap()
        .access_token;
    let admin = state
        .issue_session("admin@notify.test".into())
        .await
        .unwrap()
        .access_token;
    (state, owner, admin)
}
fn config() -> Value {
    json!({"revision":0,"current_password":"test-password","email_enabled":false,"recipient":"","webhook_enabled":true,"endpoint":"https://notify.example.com/private-path-secret","secret":"synthetic-signing-secret","threshold":70,"daily_limit":50,"retention_enabled":false,"retention_days":90})
}
#[tokio::test]
async fn notification_settings_seal_endpoint_and_enforce_owner_version_and_audit() {
    let (state, owner, admin) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/notifications/config",
            &admin,
            config()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut bad = config();
    bad["endpoint"] = json!("https://127.0.0.1/metadata");
    assert_eq!(
        request(&state, "PUT", "/v1/admin/notifications/config", &owner, bad)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let (status, saved) = request(
        &state,
        "PUT",
        "/v1/admin/notifications/config",
        &owner,
        config(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["revision"], 1);
    assert_eq!(saved["webhook_host"], "notify.example.com");
    assert!(pg.has_notification_secrets().await.unwrap());
    let raw: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {}",
        pg.t("notification_configuration")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    for private in ["private-path-secret", "synthetic-signing-secret"] {
        assert!(!saved.to_string().contains(private));
        assert!(!raw.to_string().contains(private));
    }
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/notifications/config",
            &owner,
            config()
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let mut next = config();
    next["revision"] = json!(1);
    next["endpoint"] = json!("");
    next["secret"] = json!("");
    sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT no_notification_save CHECK(action<>'notification_configuration_saved') NOT VALID",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/notifications/config",
            &owner,
            next
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(pg.notification_config().await.unwrap().revision, 1);
}
#[tokio::test]
async fn notification_collection_is_opt_in_deduplicated_quota_limited_and_body_free() {
    let (state, owner, _) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    assert_eq!(admin_notifications::collect(&state).await.unwrap(), 0);
    let mut settings = config();
    settings["daily_limit"] = json!(1);
    request(
        &state,
        "PUT",
        "/v1/admin/notifications/config",
        &owner,
        settings,
    )
    .await;
    for (id, score, age) in [
        ("old", 99, true),
        ("low", 2, false),
        ("high", 90, false),
        ("limit", 95, false),
    ] {
        sqlx::query(&format!("INSERT INTO {} (id,actor_email,action,details,created_at) VALUES ($1,'system:local-rules','publication_auto_screened',$2,CASE WHEN $3 THEN now()-interval '1 day' ELSE now() END)",pg.t("security_audit"))).bind(id).bind(json!({"publication_id":id,"result":{"score":score,"body":"never-send-this","email":"private@example.com"}})).bind(age).execute(&pg.pool).await.unwrap();
    }
    let (a, b) = tokio::join!(
        admin_notifications::collect(&state),
        admin_notifications::collect(&state)
    );
    assert_eq!(a.unwrap() + b.unwrap(), 2);
    let (status, list) = request(
        &state,
        "GET",
        "/v1/admin/notifications/deliveries",
        &owner,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["total"], 2);
    let rows = list["items"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .filter(|r| r["status"] == "quota_limited")
            .count(),
        1
    );
    assert!(!list.to_string().contains("never-send-this"));
    assert!(!list.to_string().contains("private@example.com"));
    let first = pg.claim_notification().await.unwrap().unwrap();
    assert!(pg.claim_notification().await.unwrap().is_none());
    let id = first["id"].as_str().unwrap();
    let claim = first["claim"].as_str().unwrap();
    assert!(!pg.complete_notification(id, "wrong", Ok(())).await.unwrap());
    assert!(pg
        .complete_notification(id, claim, Err("模拟超时".into()))
        .await
        .unwrap());
    let (_, list) = request(
        &state,
        "GET",
        "/v1/admin/notifications/deliveries?status=failed",
        &owner,
        json!({}),
    )
    .await;
    let revision = list["items"][0]["revision"].clone();
    assert_eq!(
        request(
            &state,
            "POST",
            &format!("/v1/admin/notifications/deliveries/{id}/retry"),
            &owner,
            json!({"revision":revision})
        )
        .await
        .0,
        StatusCode::OK
    );
    let second = pg.claim_notification().await.unwrap().unwrap();
    assert!(pg
        .complete_notification(id, second["claim"].as_str().unwrap(), Ok(()))
        .await
        .unwrap());
    assert_eq!(
        request(
            &state,
            "POST",
            &format!("/v1/admin/notifications/deliveries/{id}/retry"),
            &owner,
            json!({"revision":revision})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
}
#[tokio::test]
async fn notification_configuration_change_invalidates_jobs_and_retention_preserves_security() {
    let (state, owner, _) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    request(
        &state,
        "PUT",
        "/v1/admin/notifications/config",
        &owner,
        config(),
    )
    .await;
    let (_, queued) = request(
        &state,
        "POST",
        "/v1/admin/notifications/test",
        &owner,
        json!({"revision":1,"channel":"webhook"}),
    )
    .await;
    let id = queued["id"].as_str().unwrap();
    let claim = pg.claim_notification().await.unwrap().unwrap();
    let mut next = config();
    next["revision"] = json!(1);
    next["secret"] = json!("");
    next["endpoint"] = json!("");
    request(
        &state,
        "PUT",
        "/v1/admin/notifications/config",
        &owner,
        next.clone(),
    )
    .await;
    assert!(!pg
        .complete_notification(id, claim["claim"].as_str().unwrap(), Ok(()))
        .await
        .unwrap());
    sqlx::query(&format!(
        "UPDATE {} SET created_at=now()-interval '100 days'",
        pg.t("notification_deliveries")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        admin_notifications::cleanup(&state).await.unwrap()["enabled"],
        false
    );
    let before: i64 =
        sqlx::query_scalar(&format!("SELECT count(*) FROM {}", pg.t("security_audit")))
            .fetch_one(&pg.pool)
            .await
            .unwrap();
    next["revision"] = json!(2);
    next["retention_enabled"] = json!(true);
    request(
        &state,
        "PUT",
        "/v1/admin/notifications/config",
        &owner,
        next,
    )
    .await;
    let active = uuid::Uuid::new_v4().to_string();
    sqlx::query(&format!("INSERT INTO {} (id,event_id,channel,metadata,config_revision,status,created_at) VALUES ($1,$1,'webhook','{{}}',3,'queued',now()-interval '100 days')",pg.t("notification_deliveries"))).bind(&active).execute(&pg.pool).await.unwrap();
    let cleanup = admin_notifications::cleanup(&state).await.unwrap();
    assert_eq!(cleanup["counts"]["notification_deliveries"], 1);
    let remaining: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {}",
        pg.t("notification_deliveries")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(remaining, 1);
    let after: i64 =
        sqlx::query_scalar(&format!("SELECT count(*) FROM {}", pg.t("security_audit")))
            .fetch_one(&pg.pool)
            .await
            .unwrap();
    assert!(after >= before);
}
#[tokio::test]
async fn webhook_signs_exact_payload_and_rejects_non_success_without_logging_body() {
    use axum::{http::HeaderMap, routing::post, Router};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/events", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().route(
                "/events",
                post(|headers: HeaderMap, body: String| async move {
                    assert_eq!(headers["x-promptark-delivery"], "stable-id");
                    assert_eq!(
                        headers["x-promptark-signature"],
                        admin_notifications::signature("synthetic-secret", &body)
                    );
                    (
                        StatusCode::BAD_GATEWAY,
                        "private provider error must not be logged",
                    )
                }),
            ),
        )
        .await
        .unwrap()
    });
    let result = admin_notifications::send_webhook(
        &reqwest::Client::new(),
        &endpoint,
        "synthetic-secret",
        "stable-id",
        "{\"test\":true}",
    )
    .await;
    assert_eq!(result.unwrap_err(), "Webhook 返回 HTTP 502");
    server.abort();
    assert_ne!(
        admin_notifications::signature("a", "x"),
        admin_notifications::signature("a", "y")
    );
    assert!(crate::outbound::validate_url("https://127.0.0.1/private").is_err());
}
