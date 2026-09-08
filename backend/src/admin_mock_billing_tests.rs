use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::{json, Value};
async fn setup() -> (AppState, String, String, String) {
    let mut state = state().await;
    state.billing_mock = true;
    let pg = state.db.as_ref().unwrap();
    for (email, role) in [
        ("admin@mock.test", "owner"),
        ("a@mock.test", "user"),
        ("b@mock.test", "user"),
    ] {
        pg.upsert_account(email, Some("test-password"), role)
            .await
            .unwrap();
    }
    let mut tokens = Vec::new();
    for email in ["admin@mock.test", "a@mock.test", "b@mock.test"] {
        tokens.push(
            state
                .issue_session(email.into())
                .await
                .unwrap()
                .access_token,
        );
    }
    (
        state,
        tokens[0].clone(),
        tokens[1].clone(),
        tokens[2].clone(),
    )
}
fn batch() -> Value {
    json!({"name":"测试批次","count":2,"uses_per_code":1,"expires_at":(chrono::Utc::now()+chrono::Duration::days(1)).to_rfc3339(),"request_id":uuid::Uuid::new_v4().to_string()})
}
#[tokio::test]
async fn mock_orders_are_durable_idempotent_and_never_touch_real_pro() {
    let (state, admin, a, _) = setup().await;
    let pg = state.db.as_ref().unwrap();
    let body = json!({"email":"a@mock.test","outcome":"success","reason":"QA 模拟","request_id":uuid::Uuid::new_v4().to_string()});
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/mock-billing/actions",
            &a,
            body.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (first, second) = tokio::join!(
        request(
            &state,
            "POST",
            "/v1/admin/mock-billing/actions",
            &admin,
            body.clone()
        ),
        request(
            &state,
            "POST",
            "/v1/admin/mock-billing/actions",
            &admin,
            body.clone()
        )
    );
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(first, second);
    assert!(pg.mock_pro("a@mock.test").await.unwrap());
    assert!(!state.account_is_pro("a@mock.test").await.unwrap());
    let mut conflict = body;
    conflict["outcome"] = json!("reset");
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/mock-billing/actions",
            &admin,
            conflict
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let (_, orders) = request(
        &state,
        "GET",
        "/v1/admin/mock-billing/orders",
        &admin,
        json!({}),
    )
    .await;
    assert_eq!(orders["total"], 1);
    assert_eq!(orders["items"][0]["mock"], true);
    let reopened = Pg {
        pool: pg.pool.clone(),
        schema: pg.schema.clone(),
    };
    assert!(reopened.mock_pro("a@mock.test").await.unwrap());
    let (_, status) = request(&state, "GET", "/v1/billing/status", &a, json!({})).await;
    assert_eq!(status["mock_pro"], true);
    assert_eq!(status["pro"], false);
    let (_, detail) = request(
        &state,
        "GET",
        "/v1/admin/users/a@mock.test",
        &admin,
        json!({}),
    )
    .await;
    assert_eq!(detail["mock_pro"], true);
    assert_eq!(detail["pro"], false);
    sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT mock_audit_fail CHECK(action<>'mock_billing_changed') NOT VALID",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/billing/checkout",
            &a,
            json!({"mock_outcome":"reset"})
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert!(pg.mock_pro("a@mock.test").await.unwrap());
}
#[tokio::test]
async fn mock_codes_enforce_limits_revocation_and_real_path_isolation() {
    let (state, admin, a, b) = setup().await;
    let pg = state.db.as_ref().unwrap();
    let config = batch();
    let (status, created) = request(
        &state,
        "POST",
        "/v1/admin/mock-billing/batches",
        &admin,
        config.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/mock-billing/batches",
            &admin,
            config
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let codes = created["codes"].as_array().unwrap();
    assert_eq!(codes.len(), 2);
    let id = created["id"].as_str().unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/billing/redeem",
            &a,
            json!({"code":codes[0]})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let req = json!({"code":codes[0],"request_id":uuid::Uuid::new_v4().to_string()});
    let (ra, rb) = tokio::join!(
        request(&state, "POST", "/v1/billing/mock/redeem", &a, req.clone()),
        request(&state, "POST", "/v1/billing/mock/redeem", &b, req.clone())
    );
    assert_eq!(
        [ra.0, rb.0]
            .iter()
            .filter(|s| **s == StatusCode::OK)
            .count(),
        1
    );
    let winner = if ra.0 == StatusCode::OK { &a } else { &b };
    assert_eq!(
        request(&state, "POST", "/v1/billing/mock/redeem", winner, req)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/billing/mock/redeem",
            winner,
            json!({"code":codes[0]})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let path = format!("/v1/admin/mock-billing/batches/{id}");
    let (_, detail) = request(&state, "GET", &path, &admin, json!({})).await;
    assert!(!detail.to_string().contains(codes[0].as_str().unwrap()));
    assert!(!detail.to_string().contains("code_hash"));
    assert_eq!(detail["recent_uses"].as_array().unwrap().len(), 1);
    assert_eq!(
        request(
            &state,
            "PUT",
            &path,
            &admin,
            json!({"revision":0,"enabled":false})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/billing/mock/redeem",
            &a,
            json!({"code":codes[1]})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert!(!state.account_is_pro("a@mock.test").await.unwrap());
    assert!(!state.account_is_pro("b@mock.test").await.unwrap());
    sqlx::query(&format!(
        "UPDATE {} SET expires_at=now()-interval '1 second'",
        pg.t("mock_code_batches")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "PUT",
            &path,
            &admin,
            json!({"revision":1,"enabled":true})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let mut disabled = state.clone();
    disabled.billing_mock = false;
    assert_eq!(
        request(
            &disabled,
            "POST",
            "/v1/billing/mock/redeem",
            &a,
            json!({"code":codes[1]})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &disabled,
            "POST",
            "/v1/admin/mock-billing/batches",
            &admin,
            batch()
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &disabled,
            "GET",
            "/v1/admin/mock-billing/batches",
            &admin,
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
}
