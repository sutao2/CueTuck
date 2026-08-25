use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use promptark_api::{app, AppState};
use tower::ServiceExt;

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
async fn saves_display_name_for_signed_in_user_and_rejects_anonymous() {
    let (app, session) = login(app(AppState::with_user("dev@promptark.local", "devpass"))).await;
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/me")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"display_name":"林晚","bio":"人像"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let saved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/me")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                .body(Body::from(r#"{"display_name":"林晚","bio":"人像"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(saved.status(), StatusCode::OK);
    let listed = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/me")
                .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["email"], "dev@promptark.local");
    assert_eq!(payload["display_name"], "林晚");
    assert_eq!(payload["bio"], "人像");
}
