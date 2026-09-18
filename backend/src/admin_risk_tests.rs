use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::json;

#[tokio::test]
async fn report_limits_exports_and_rule_validation_are_enforced() {
    let (state, admin, user, reviewer) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    let bad = json!({"revision":0,"rules":[{"id":"bad","name":"bad","category":"privacy","words":[" "],"score":30,"enabled":true}]});
    assert_eq!(
        request(&state, "PUT", "/v1/admin/safety-rules", &admin, bad)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    for i in 0..20 {
        sqlx::query(&format!("INSERT INTO {} (id,target_id,reporter,category,reason,status) VALUES ($1,'old','reporter@test.dev','other','private reason','dismissed')",pg.t("reports"))).bind(format!("limit-{i}")).execute(&pg.pool).await.unwrap();
    }
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/reports",
            &user,
            json!({"target_id":"risk-item","category":"other","reason":"new"})
        )
        .await
        .0,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        request(&state, "GET", "/v1/reports", &reviewer, json!({}))
            .await
            .1["total"],
        0
    );
    let export = request(&state, "GET", "/v1/admin/reports/export", &admin, json!({})).await;
    assert_eq!(export.0, StatusCode::OK);
    assert_eq!(export.1["total"], 20);
    assert!(!export.1.to_string().contains("private reason"));
    assert!(!export.1.to_string().contains("reporter@test.dev"));
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/admin/reports?status=unknown",
            &admin,
            json!({})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    sqlx::query(&format!(
        "UPDATE {} SET created_at=now()-interval '25 hours'",
        pg.t("reports")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/reports",
            &user,
            json!({"target_id":"risk-item","category":"other","reason":"new"})
        )
        .await
        .0,
        StatusCode::OK
    );
    pg.apply_schema(false).await.unwrap();
    assert_eq!(
        request(&state, "GET", "/v1/reports", &user, json!({}))
            .await
            .1["total"],
        21
    );
    pg.upsert_account("admin@test.dev", None, "user")
        .await
        .unwrap();
    assert_eq!(
        request(&state, "GET", "/v1/admin/reports", &admin, json!({}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
}

async fn fixture() -> (AppState, String, String, String) {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    for (email, role) in [
        ("admin@test.dev", "admin"),
        ("reporter@test.dev", "user"),
        ("reviewer@test.dev", "reviewer"),
    ] {
        pg.upsert_account(email, Some("test-password"), role)
            .await
            .unwrap();
    }
    let admin = state
        .issue_session("admin@test.dev".into())
        .await
        .unwrap()
        .access_token;
    let user = state
        .issue_session("reporter@test.dev".into())
        .await
        .unwrap()
        .access_token;
    let reviewer = state
        .issue_session("reviewer@test.dev".into())
        .await
        .unwrap()
        .access_token;
    state
        .insert_publication(&Publication { cover: None, asset_refs: vec![],
            id: "risk-item".into(),
            source_id: "local".into(),
            status: "pending".into(),
            title: Some("Test public content".into()),
            content: Some("test-secret advertised".into()),
            author_email: Some("reporter@test.dev".into()),
            category_id: None,
            model: None,
            kind: "prompt".into(),
            members: vec![],
        })
        .await
        .unwrap();
    pg.review_publication(
        "risk-item",
        "approved",
        None,
        Some(("admin@test.dev", &admin)),
    )
    .await
    .unwrap();
    (state, admin, user, reviewer)
}

#[tokio::test]
async fn reports_deduplicate_restrict_access_and_close_atomically() {
    let (state, admin, user, reviewer) = fixture().await;
    let body = json!({"target_id":"risk-item","category":"privacy","reason":"Contains a secret"});
    let (a, b) = tokio::join!(
        request(&state, "POST", "/v1/reports", &user, body.clone()),
        request(&state, "POST", "/v1/reports", &user, body)
    );
    assert_eq!(a.0, StatusCode::OK);
    assert_eq!(a.1["id"], b.1["id"]);
    for token in [&user, &reviewer] {
        assert_eq!(
            request(&state, "GET", "/v1/admin/reports", token, json!({}))
                .await
                .0,
            StatusCode::FORBIDDEN
        );
    }
    let id = a.1["id"].as_str().unwrap();
    let path = format!("/v1/admin/reports/{id}");
    assert_eq!(request(&state,"PUT",&path,&admin,json!({"revision":0,"action":"assign","reason":"Handle this","assignee":"reviewer@test.dev","priority":"high"})).await.0,StatusCode::BAD_REQUEST);
    let (status,done)=request(&state,"PUT",&path,&admin,json!({"revision":0,"action":"offline","reason":"Verified secret exposure","priority":"high"})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(done["status"], "closed");
    assert_eq!(
        request(&state, "GET", "/v1/square/items/risk-item", "", json!({}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            &path,
            &admin,
            json!({"revision":0,"action":"dismiss","reason":"old page","priority":"normal"})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let mine = request(&state, "GET", "/v1/reports", &user, json!({}))
        .await
        .1;
    assert_eq!(mine["items"][0]["resolution"], "Verified secret exposure");
    assert!(!mine.to_string().contains("admin@test.dev"));
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/reports",
            &user,
            json!({"target_id":"missing","category":"privacy","reason":"test"})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn rule_versions_matching_and_audit_rollback() {
    let (state, admin, user, _) = fixture().await;
    let config = json!({"revision":0,"rules":[{"id":"secrets","name":"Secret exposure","category":"privacy","words":["TEST-SECRET"],"score":90,"enabled":true}]});
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/safety-rules",
            &user,
            config.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/safety-rules",
            &admin,
            config.clone()
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(&state, "PUT", "/v1/admin/safety-rules", &admin, config)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let result = request(
        &state,
        "POST",
        "/v1/admin/safety-rules/test",
        &admin,
        json!({"text":"a test-secret","category":null}),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["score"], 90);
    assert_eq!(result.1["source"], "local_rules");
    let report = request(
        &state,
        "POST",
        "/v1/reports",
        &user,
        json!({"target_id":"risk-item","category":"privacy","reason":"reason"}),
    )
    .await
    .1;
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!(
        "ALTER TABLE {} ADD CONSTRAINT reject_risk_audit CHECK (action <> 'report_offline')",
        pg.t("security_audit")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "PUT",
            &format!("/v1/admin/reports/{}", report["id"].as_str().unwrap()),
            &admin,
            json!({"revision":0,"action":"offline","reason":"reason","priority":"high"})
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        request(&state, "GET", "/v1/square/items/risk-item", "", json!({}))
            .await
            .0,
        StatusCode::OK
    );
    let reports = request(&state, "GET", "/v1/admin/reports", &admin, json!({}))
        .await
        .1;
    assert_eq!(reports["items"][0]["revision"], 0);
}

#[tokio::test]
async fn admin_usage_exemption_preserves_report_dedup_and_current_role() {
    let (state, _, _, _) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    for role in ["user", "reviewer", "support", "admin", "owner"] {
        let email = format!("quota-{role}@risk.test");
        pg.upsert_account(&email, Some("test-password"), role).await.unwrap();
        let token = state.issue_session(email.clone()).await.unwrap().access_token;
        sqlx::query(&format!("INSERT INTO {}(id,target_id,reporter,category,reason,status) SELECT $1||n,'old',$2,'other','previous','dismissed' FROM generate_series(1,20) n", pg.t("reports")))
            .bind(role).bind(&email).execute(&pg.pool).await.unwrap();
        let input = json!({"target_id":"risk-item","category":"other","reason":"new report"});
        let result = request(&state, "POST", "/v1/reports", &token, input.clone()).await;
        let allowed = matches!(role, "admin" | "owner");
        assert_eq!(result.0, if allowed { StatusCode::OK } else { StatusCode::TOO_MANY_REQUESTS }, "{role}");
        if allowed {
            assert_eq!(request(&state, "POST", "/v1/reports", &token, input.clone()).await.1["id"], result.1["id"]);
            assert_eq!(request(&state, "POST", "/v1/reports", &token,
                json!({"target_id":"missing","category":"other","reason":"new"})).await.0, StatusCode::NOT_FOUND);
            sqlx::query(&format!("UPDATE {} SET role='user' WHERE email=$1", pg.t("accounts")))
                .bind(&email).execute(&pg.pool).await.unwrap();
            sqlx::query(&format!("UPDATE {} SET status='dismissed' WHERE reporter=$1", pg.t("reports")))
                .bind(&email).execute(&pg.pool).await.unwrap();
            assert_eq!(request(&state, "POST", "/v1/reports", &token, input).await.0, StatusCode::TOO_MANY_REQUESTS);
        }
        let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {} WHERE reporter=$1", pg.t("reports")))
            .bind(&email).fetch_one(&pg.pool).await.unwrap();
        assert_eq!(count, if allowed {21} else {20});
    }
    sqlx::query(&format!("DROP SCHEMA {} CASCADE", pg.schema)).execute(&pg.pool).await.unwrap();
}
