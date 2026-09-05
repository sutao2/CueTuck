use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use promptark_api::{app, AppState};
use tower::ServiceExt;

#[tokio::test]
async fn sync_compares_seconds_and_milliseconds_and_rejects_invalid_batches() {
    let (app, session) = login(app(AppState::with_user("dev@promptark.local", "devpass"))).await;
    let token = format!("Bearer {}", session.access_token);
    for (stamp, title) in [("1700000000000", "旧"), ("1700000001", "新"), ("1700000000500", "较旧毫秒")] {
        let payload = serde_json::json!({"items":[{"id":"p", "kind":"prompt", "payload":{"title": title}, "updated_at": stamp}]});
        let response = app.clone().oneshot(Request::builder().method("PUT").uri("/v1/library/changes")
            .header(header::CONTENT_TYPE, "application/json").header(header::AUTHORIZATION, &token)
            .body(Body::from(payload.to_string())).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    let response = app.clone().oneshot(Request::builder().uri("/v1/library/changes?since=1700000000500")
        .header(header::AUTHORIZATION, &token).body(Body::empty()).unwrap()).await.unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["items"][0]["payload"]["title"], "新");
    let invalid = serde_json::json!({"items":[
        {"id":"q", "kind":"prompt", "payload":{"title":"不得部分写入"}, "updated_at":"1"},
        {"id":"bad", "kind":"prompt", "payload":{}, "updated_at":"not-a-timestamp"}
    ]});
    let response = app.clone().oneshot(Request::builder().method("PUT").uri("/v1/library/changes")
        .header(header::CONTENT_TYPE, "application/json").header(header::AUTHORIZATION, &token)
        .body(Body::from(invalid.to_string())).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let response = app.oneshot(Request::builder().uri("/v1/library/changes")
        .header(header::AUTHORIZATION, &token).body(Body::empty()).unwrap()).await.unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["items"].as_array().unwrap().len(), 1);
}

async fn login(app: axum::Router) -> (axum::Router, promptark_api::SessionResponse) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/session")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"dev@promptark.local","password":"devpass"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (app, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn put_then_get_library_changes_for_signed_in_account() {
    let (app, session) = login(app(AppState::with_user("dev@promptark.local", "devpass"))).await;
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/library/changes")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"items":[]}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let pushed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/library/changes")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                .body(Body::from(
                    r#"{"items":[{"id":"p-1","kind":"prompt","payload":{"title":"本地仍在","content":"正文"},"updated_at":"2"}]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(pushed.status(), StatusCode::OK);
    let listed = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/library/changes?since=1")
                .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let items = payload["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "p-1");
    assert_eq!(items[0]["payload"]["title"], "本地仍在");
    assert_eq!(items[0]["payload"]["content"], "正文");
}

#[tokio::test]
async fn newer_updated_at_wins_when_putting_library_changes() {
    let (app, session) = login(app(AppState::with_user("dev@promptark.local", "devpass"))).await;
    let token = format!("Bearer {}", session.access_token);
    let newer = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/library/changes")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &token)
                .body(Body::from(
                    r#"{"items":[{"id":"p-1","kind":"prompt","payload":{"title":"本地仍在","content":"远端正文"},"updated_at":"2"}]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(newer.status(), StatusCode::OK);
    let older = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/library/changes")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &token)
                .body(Body::from(
                    r#"{"items":[{"id":"p-1","kind":"prompt","payload":{"title":"本地仍在","content":"本机正文"},"updated_at":"1"}]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(older.status(), StatusCode::OK);
    let listed = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/library/changes?since=")
                .header(header::AUTHORIZATION, token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["items"][0]["payload"]["content"], "远端正文");
}
