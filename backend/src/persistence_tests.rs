use super::*;
use axum::body::{to_bytes, Body};
use axum::http::Request;
use axum::Router;
use tower::ServiceExt;

async fn postgres_state() -> Option<AppState> {
    let url = std::env::var("PROMPTARK_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://pl:pl@127.0.0.1:5432/promptark?sslmode=disable".into()
    });
    let pool = sqlx::PgPool::connect(&url).await.ok()?;
    let schema = format!("t{}", Uuid::new_v4().simple());
    AppState::from_pool(pool, &schema).await.ok()
}

#[tokio::test]
async fn session_survives_new_appstate_on_postgres() {
    let Some(state) = postgres_state().await else {
        panic!("expected local Postgres at postgres://pl:pl@127.0.0.1:5432/promptark");
    };
    state
        .db
        .as_ref()
        .unwrap()
        .upsert_account("dev@promptark.local", Some("devpass"), "user")
        .await
        .unwrap();
    let router = app(state.clone());
    let login = router
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
    assert_eq!(login.status(), StatusCode::OK);
    let body = to_bytes(login.into_body(), usize::MAX).await.unwrap();
    let session: SessionResponse = serde_json::from_slice(&body).unwrap();
    let cloned = AppState {
        db: state.db.clone(),
        ..AppState::default()
    };
    let refreshed = app(cloned)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/session/refresh")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!(
                    r#"{{"refresh_token":"{}"}}"#,
                    session.refresh_token
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(refreshed.status(), StatusCode::OK);
}

#[tokio::test]
async fn publication_favorite_and_settings_survive_postgres() {
    let Some(state) = postgres_state().await else {
        panic!("expected local Postgres at postgres://pl:pl@127.0.0.1:5432/promptark");
    };
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("dev@promptark.local", Some("devpass"), "user")
        .await
        .unwrap();
    pg.upsert_account("admin@promptark.local", Some("adminpass"), "admin")
        .await
        .unwrap();
    pg.replace_items(&[SquareItem {
        id: "sq-1".into(),
        title: "自然光群像".into(),
        kind: "prompt".into(),
        excerpt: None,
        model: None,
        member_count: None,
        content: Some("body".into()),
    }])
    .await
    .unwrap();
    let router = app(state.clone());
    let user = login_json(&router, "dev@promptark.local", "devpass").await;
    let admin = login_json(&router, "admin@promptark.local", "adminpass").await;
    let published = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/publications")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", user.access_token))
                .body(Body::from(
                    r#"{"source_id":"local-1","title":"过审标题","content":"快照"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::OK);
    let body = to_bytes(published.into_body(), usize::MAX).await.unwrap();
    let publication: Publication = serde_json::from_slice(&body).unwrap();
    let approved = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/admin/publications/{}/approve",
                    publication.id
                ))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", admin.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(approved.status(), StatusCode::OK);
    let fav = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/favorites/sq-1")
                .header(header::AUTHORIZATION, format!("Bearer {}", user.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(fav.status(), StatusCode::OK);
    let closed = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/admin/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", admin.access_token),
                )
                .body(Body::from(r#"{"square_public":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(closed.status(), StatusCode::OK);
    let fresh = AppState {
        db: state.db.clone(),
        ..AppState::default()
    };
    let list = app(fresh.clone())
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/square/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(list.into_body(), usize::MAX).await.unwrap();
    let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
    assert!(payload.items.is_empty());
    fresh.set_square_public(true).await.unwrap();
    let listed = app(fresh.clone())
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/square/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
    assert!(payload.items.iter().any(|item| item.title == "过审标题"));
    let favorites = app(fresh)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/favorites")
                .header(header::AUTHORIZATION, format!("Bearer {}", user.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(favorites.status(), StatusCode::OK);
}

async fn login_json(router: &Router, email: &str, password: &str) -> SessionResponse {
    let response = router
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
    serde_json::from_slice(&body).unwrap()
}
