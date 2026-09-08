use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use promptark_api::{app, AppState, SquareItem, SquareListResponse};
use tower::ServiceExt;

fn demo_sort_items() -> Vec<SquareItem> {
    vec![
        SquareItem {
            reference: None,
            id: "sq-b".into(),
            title: "Beta".into(),
            kind: "prompt".into(),
            excerpt: None,
            model: Some("Flux".into()),
            category_id: None,
            member_count: None,
            content: None,
            members: vec![],
        },
        SquareItem {
            reference: None,
            id: "sq-a".into(),
            title: "Alpha".into(),
            kind: "prompt".into(),
            excerpt: None,
            model: None,
            category_id: None,
            member_count: None,
            content: None,
            members: vec![],
        },
        SquareItem {
            reference: None,
            id: "sq-g".into(),
            title: "Gamma".into(),
            kind: "prompt".into(),
            excerpt: None,
            model: Some("Flux".into()),
            category_id: None,
            member_count: None,
            content: None,
            members: vec![],
        },
    ]
}

async fn square_titles(app: &axum::Router, uri: &str) -> Vec<String> {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
    payload.items.into_iter().map(|item| item.title).collect()
}

#[tokio::test]
async fn record_anonymous_download_increments_count_without_auth() {
    let app = app(AppState::with_square_items(demo_sort_items()));
    let counted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/square/items/sq-g/downloads")
                .header(header::AUTHORIZATION, "Bearer garbage")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(counted.status(), StatusCode::NO_CONTENT);
    let counted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/square/items/sq-g/downloads")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(counted.status(), StatusCode::NO_CONTENT);
    let counted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/square/items/sq-b/downloads")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(counted.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        square_titles(&app, "/v1/square/items?sort=hot").await,
        vec!["Gamma", "Beta", "Alpha"]
    );
}

#[tokio::test]
async fn get_content_does_not_count_as_anonymous_stats() {
    let app = app(AppState::with_square_items(demo_sort_items()));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/square/items/sq-g/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        square_titles(&app, "/v1/square/items?sort=hot").await,
        vec!["Alpha", "Beta", "Gamma"]
    );
}

#[tokio::test]
async fn missing_item_download_stat_is_404() {
    let app = app(AppState::with_square_items(demo_sort_items()));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/square/items/no-such/downloads")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
