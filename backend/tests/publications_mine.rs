use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use promptark_api::{app, AdminPublicationList, AppState};
use tower::ServiceExt;

async fn login_as(
    app: axum::Router,
    email: &str,
    password: &str,
) -> (axum::Router, promptark_api::SessionResponse) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/session")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!(
                    r#"{{"email":"{email}","password":"{password}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (app, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn lists_own_publications_and_hides_other_accounts() {
    let app = app(AppState::with_users(&[
        ("dev@promptark.local", "devpass", "user"),
        ("other@promptark.local", "otherpass", "user"),
    ]));
    let (app, owner) = login_as(app, "dev@promptark.local", "devpass").await;
    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/publications")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", owner.access_token))
                .body(Body::from(
                    r#"{"source_id":"mem-1","title":"新稿","content":"快照"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::OK);
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/publications/mine")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let mine = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/publications/mine")
                .header(header::AUTHORIZATION, format!("Bearer {}", owner.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(mine.status(), StatusCode::OK);
    let body = to_bytes(mine.into_body(), usize::MAX).await.unwrap();
    let payload: AdminPublicationList = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload.items.len(), 1);
    assert_eq!(payload.items[0].title.as_deref(), Some("新稿"));
    assert_eq!(payload.items[0].status, "pending");
    assert_eq!(
        payload.items[0].author_email.as_deref(),
        Some("dev@promptark.local")
    );
    let (app, other) = login_as(app, "other@promptark.local", "otherpass").await;
    let hidden = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/publications/mine")
                .header(header::AUTHORIZATION, format!("Bearer {}", other.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(hidden.status(), StatusCode::OK);
    let body = to_bytes(hidden.into_body(), usize::MAX).await.unwrap();
    let payload: AdminPublicationList = serde_json::from_slice(&body).unwrap();
    assert!(payload.items.is_empty());
}
