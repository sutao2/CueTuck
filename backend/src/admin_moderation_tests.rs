use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::{json, Value};
async fn setup() -> (AppState, String, String) {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("owner@mod.test", Some("test-password"), "owner")
        .await
        .unwrap();
    pg.upsert_account("user@mod.test", Some("test-password"), "user")
        .await
        .unwrap();
    let owner = state
        .issue_session("owner@mod.test".into())
        .await
        .unwrap()
        .access_token;
    let user = state
        .issue_session("user@mod.test".into())
        .await
        .unwrap()
        .access_token;
    (state, owner, user)
}
fn policy(revision: i64) -> Value {
    json!({"revision":revision,"enabled":true,"daily_limit":10,"auto_approve":true,"approve_below":30,"manual_at":70,"check_duplicates":true,"check_structure":true,"check_sensitive":true,"check_images":false,"require_ai":false})
}
fn post(source: &str, content: &str) -> Value {
    json!({"source_id":source,"title":"自动审核测试","content":content})
}

#[tokio::test]
async fn moderation_sensitive_images_structure_and_global_duplicate_race() {
    let (state, owner, user) = setup().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("second@mod.test", Some("test-password"), "user")
        .await
        .unwrap();
    let second = state
        .issue_session("second@mod.test".into())
        .await
        .unwrap()
        .access_token;
    let mut config = policy(0);
    config["check_images"] = json!(true);
    assert_eq!(
        request(&state, "PUT", "/v1/admin/moderation", &owner, config)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(request(&state,"PUT","/v1/admin/safety-rules",&owner,json!({"revision":0,"rules":[{"id":"private","name":"隐私词","category":"privacy","words":["dangerous-secret"],"enabled":true,"score":95}]})).await.0,StatusCode::OK);
    for content in [
        "dangerous-secret",
        "![photo](https://example.com/test.png)",
        "bad }} before {{",
        " ",
    ] {
        assert_eq!(
            request(
                &state,
                "POST",
                "/v1/publications",
                &user,
                post(&format!("source-{}",content.len()), content)
            )
            .await
            .1["status"],
            "pending"
        );
    }
    let (a, b) = tokio::join!(
        request(
            &state,
            "POST",
            "/v1/publications",
            &user,
            post("race-a", "same-public-body")
        ),
        request(
            &state,
            "POST",
            "/v1/publications",
            &second,
            post("race-b", "same-public-body")
        )
    );
    assert_eq!(a.0, StatusCode::OK);
    assert_eq!(b.0, StatusCode::OK);
    assert_ne!(a.1["status"], b.1["status"]);
    assert_eq!(
        request(&state, "PUT", "/v1/admin/moderation", &owner, policy(0))
            .await
            .0,
        StatusCode::CONFLICT
    );
    pg.apply_schema(false).await.unwrap();
    assert_eq!(
        request(&state, "GET", "/v1/admin/moderation", &owner, json!({}))
            .await
            .1["enabled"],
        true
    );
}
#[tokio::test]
async fn moderation_disabled_local_approval_duplicate_and_dependency_fallback() {
    let (state, owner, user) = setup().await;
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/publications",
            &user,
            post("one", "initial content")
        )
        .await
        .1["status"],
        "pending"
    );
    assert_eq!(
        request(&state, "PUT", "/v1/admin/moderation", &user, policy(0))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&state, "PUT", "/v1/admin/moderation", &owner, policy(0))
            .await
            .0,
        StatusCode::OK
    );
    let (status, approved) = request(
        &state,
        "POST",
        "/v1/publications",
        &user,
        post("two", "safe unique text"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(approved["status"], "approved");
    let id = approved["id"].as_str().unwrap();
    assert_eq!(
        request(
            &state,
            "GET",
            &format!("/v1/square/items/{id}"),
            "",
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/publications",
            &user,
            post("three", "safe unique text")
        )
        .await
        .1["status"],
        "pending"
    );
    let mut config = policy(1);
    config["require_ai"] = json!(true);
    assert_eq!(
        request(&state, "PUT", "/v1/admin/moderation", &owner, config)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/publications",
            &user,
            post("four", "different unique text")
        )
        .await
        .1["status"],
        "pending"
    );
    let mine = request(&state, "GET", "/v1/publications/mine", &user, json!({}))
        .await
        .1;
    assert!(mine.to_string().contains("AI"));
    assert!(!mine.to_string().contains("owner@mod.test"));
    let approved = mine["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .unwrap();
    assert_eq!(approved["history"].as_array().unwrap().len(), 1);
}
#[tokio::test]
async fn moderation_quota_is_atomic_and_audit_failure_rolls_back() {
    let (state, owner, user) = setup().await;
    let mut config = policy(0);
    config["daily_limit"] = json!(1);
    assert_eq!(
        request(&state, "PUT", "/v1/admin/moderation", &owner, config)
            .await
            .0,
        StatusCode::OK
    );
    let (a, b) = tokio::join!(
        request(
            &state,
            "POST",
            "/v1/publications",
            &user,
            post("a", "first")
        ),
        request(
            &state,
            "POST",
            "/v1/publications",
            &user,
            post("b", "second")
        )
    );
    assert!(
        (a.0 == StatusCode::OK && b.0 == StatusCode::TOO_MANY_REQUESTS)
            || (b.0 == StatusCode::OK && a.0 == StatusCode::TOO_MANY_REQUESTS)
    );
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!(
        "UPDATE {} SET created_at=now()-interval '25 hours'",
        pg.t("publications")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT fail_auto_audit CHECK(action <> 'publication_auto_screened') NOT VALID",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/publications",
            &user,
            post("rollback", "not committed")
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {}", pg.t("publications")))
        .fetch_one(&pg.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}
