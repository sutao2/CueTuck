use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::{json, Value};

async fn fixture() -> (AppState, String, String) {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("reviewer@example.com", Some("test-password"), "reviewer")
        .await
        .unwrap();
    pg.upsert_account("author@example.com", Some("test-password"), "user")
        .await
        .unwrap();
    let reviewer = state
        .issue_session("reviewer@example.com".into())
        .await
        .unwrap()
        .access_token;
    let author = state
        .issue_session("author@example.com".into())
        .await
        .unwrap()
        .access_token;
    (state, reviewer, author)
}
async fn publication(state: &AppState, id: &str) {
    state
        .insert_publication(&Publication { asset_refs: vec![],
            id: id.into(),
            source_id: format!("local-{id}"),
            status: "pending".into(),
            title: Some(format!("Title {id}")),
            content: Some("作者原始正文 {{变量}}".into()),
            author_email: Some("author@example.com".into()),
            category_id: None,
            model: None,
            kind: "prompt".into(),
            members: vec![],
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn reviews_filter_pagination_dates_and_history_survive_restart() {
    let (state, reviewer, _) = fixture().await;
    for id in ["a", "b", "c"] {
        publication(&state, id).await;
    }
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!("UPDATE {} SET created_at=CASE WHEN id='c' THEN NULL ELSE '2026-09-07 12:00:00+00'::timestamptz END", pg.t("publications"))).execute(&pg.pool).await.unwrap();
    let (status, list) = request(
        &state,
        "GET",
        "/v1/admin/reviews?from=2026-09-07&to=2026-09-07&author=AUTHOR&limit=1&offset=1",
        &reviewer,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["total"], 2);
    assert_eq!(list["items"][0]["id"], "b");
    for query in [
        "from=2026-02-30",
        "from=2026-09-08&to=2026-09-07",
        "limit=101",
        "offset=-1",
        "status=other",
    ] {
        assert_eq!(
            request(
                &state,
                "GET",
                &format!("/v1/admin/reviews?{query}"),
                &reviewer,
                json!({})
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    let restarted = AppState {
        db: state.db.clone(),
        ..AppState::default()
    };
    let list = request(
        &restarted,
        "GET",
        "/v1/admin/reviews?q=local-c",
        &reviewer,
        json!({}),
    )
    .await
    .1;
    assert_eq!(list["total"], 1);
    assert!(list["items"][0]["created_at"].is_null());
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/admin/reviews?q=%25",
            &reviewer,
            json!({})
        )
        .await
        .1["total"],
        0
    );
}

#[tokio::test]
async fn review_reason_is_first_write_only_and_author_history_is_private() {
    let (state, reviewer, author) = fixture().await;
    publication(&state, "a").await;
    for reason in ["缺少使用说明", "不应覆盖首次原因"] {
        assert_eq!(
            request(
                &state,
                "POST",
                "/v1/admin/publications/a/reject",
                &reviewer,
                json!({"reason":reason})
            )
            .await
            .0,
            StatusCode::OK
        );
    }
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/publications/a/approve",
            &reviewer,
            json!({})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let list = request(
        &state,
        "GET",
        "/v1/admin/reviews?status=rejected",
        &reviewer,
        json!({}),
    )
    .await
    .1;
    let history = &list["items"][0]["history"];
    assert_eq!(history.as_array().unwrap().len(), 1);
    assert_eq!(history[0]["actor_email"], "reviewer@example.com");
    assert_eq!(history[0]["reason"], "缺少使用说明");
    let mine = request(&state, "GET", "/v1/publications/mine", &author, json!({}))
        .await
        .1;
    assert_eq!(mine["items"][0]["history"][0]["reason"], "缺少使用说明");
    assert!(!mine.to_string().contains("reviewer@example.com"));
    assert_eq!(mine["items"][0]["content"], "作者原始正文 {{变量}}");
    assert_eq!(
        request(&state, "GET", "/v1/publications/mine", &reviewer, json!({}))
            .await
            .1["items"],
        json!([])
    );
}

#[tokio::test]
async fn concurrent_review_records_once_and_audit_failure_rolls_back_listing() {
    let (state, reviewer, _) = fixture().await;
    publication(&state, "a").await;
    publication(&state, "b").await;
    let pg = state.db.as_ref().unwrap();
    let (a, b) = tokio::join!(
        pg.review_publication(
            "a",
            "approved",
            None,
            Some(("reviewer@example.com", &reviewer))
        ),
        pg.review_publication(
            "a",
            "approved",
            None,
            Some(("reviewer@example.com", &reviewer))
        )
    );
    assert!(a.is_ok() && b.is_ok());
    let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {}", pg.t("review_events")))
        .fetch_one(&pg.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT fail_review CHECK (details->>'publication_id' IS DISTINCT FROM 'b')", pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/publications/b/approve",
            &reviewer,
            json!({})
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert!(state.get_item("b").await.unwrap().is_none());
    assert_eq!(state.pending_publications().await.unwrap()[0].id, "b");
    let count: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {} WHERE publication_id='b'",
        pg.t("review_events")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn batch_reports_each_failure_and_validates_before_any_write() {
    let (state, reviewer, author) = fixture().await;
    publication(&state, "a").await;
    publication(&state, "b").await;
    for body in [
        json!({"ids":["a"],"status":"rejected","reason":" "}),
        json!({"ids":["a","a"],"status":"approved"}),
        json!({"ids":[],"status":"approved"}),
        json!({"ids":(0..51).map(|i| i.to_string()).collect::<Vec<_>>(),"status":"approved"}),
    ] {
        assert_eq!(
            request(&state, "POST", "/v1/admin/reviews/batch", &reviewer, body)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/reviews/batch",
            &author,
            json!({"ids":["a"],"status":"approved"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    request(
        &state,
        "POST",
        "/v1/admin/publications/a/reject",
        &reviewer,
        json!({"reason":"说明不足"}),
    )
    .await;
    let (status, result) = request(
        &state,
        "POST",
        "/v1/admin/reviews/batch",
        &reviewer,
        json!({"ids":["a","missing","b"],"status":"approved"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        result["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["status"].clone())
            .collect::<Vec<Value>>(),
        vec![json!(409), json!(404), json!(200)]
    );
    assert!(state.get_item("b").await.unwrap().is_some());
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!(
        "UPDATE {} SET role='user' WHERE email='reviewer@example.com'",
        pg.t("accounts")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert!(matches!(
        pg.review_publication(
            "b",
            "approved",
            None,
            Some(("reviewer@example.com", &reviewer))
        )
        .await,
        Err(StatusCode::FORBIDDEN)
    ));
    sqlx::query(&format!(
        "UPDATE {} SET disabled=true WHERE email='reviewer@example.com'",
        pg.t("accounts")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert!(matches!(
        pg.review_publication(
            "b",
            "approved",
            None,
            Some(("reviewer@example.com", &reviewer))
        )
        .await,
        Err(StatusCode::UNAUTHORIZED)
    ));
}
