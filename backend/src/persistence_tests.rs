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
async fn admin_oauth_configuration_is_encrypted_and_survives_new_process_state() {
    let state = postgres_state().await.expect("local Postgres required");
    state.db.as_ref().unwrap().upsert_account("oauth-admin@example.com", Some("test-password"), "owner").await.unwrap();
    let session = state.issue_session("oauth-admin@example.com".into()).await.unwrap();
    let router = app(state.clone());
    let response = router.oneshot(Request::builder().method("PUT").uri("/v1/admin/oauth/google")
        .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token)).header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({ "revision":0,"current_password":"test-password", "enabled": true, "client_id": "persisted-client", "client_secret": "persisted-secret", "redirect_uri": "http://localhost:8787/v1/session/oauth/callback" }).to_string())).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let stored = state.db.as_ref().unwrap().oauth_config("google").await.unwrap().unwrap();
    assert!(!stored.contains("persisted-secret"));
    let restarted = AppState { db: state.db.clone(), oauth_config: std::sync::Arc::new(crate::oauth_admin::ConfigStore::new(state.oauth_config.key)), ..AppState::default() };
    let config = restarted.runtime_provider("google").await.unwrap().unwrap();
    assert_eq!(config.client_id, "persisted-client");
    assert_eq!(config.client_secret, "persisted-secret");
    let wrong_key = AppState { db: state.db.clone(), ..AppState::default() };
    assert!(wrong_key.runtime_provider("google").await.is_err());
}

#[tokio::test]
async fn collection_members_survive_publication_review_and_restart() {
    let state = postgres_state().await.expect("local Postgres required");
    let publication = Publication { asset_refs: vec![], id: "collection-test".into(), source_id: "local".into(), status: "pending".into(),
        title: Some("合集".into()), content: None, author_email: Some("dev@promptark.local".into()),
        category_id: Some("cat-image".into()), model: None, kind: "collection".into(),
        members: vec![PublishedPrompt { asset_ids: vec![], title: "成员".into(), content: "原始正文".into(), category_id: Some("cat-image-0".into()), model: Some("Flux".into()) }] };
    state.insert_publication(&publication).await.unwrap();
    let restarted = AppState { db: state.db.clone(), ..AppState::default() };
    let pending = restarted.pending_publications().await.unwrap();
    assert_eq!(pending[0].kind, "collection");
    assert_eq!(pending[0].members[0].content, "原始正文");
    restarted.set_publication_status(&publication.id, "approved").await.unwrap();
    let restarted = AppState { db: state.db.clone(), ..AppState::default() };
    let item = restarted.get_item(&publication.id).await.unwrap().unwrap();
    assert_eq!(item.kind, "collection");
    assert_eq!(item.member_count, Some(1));
    assert_eq!(item.members[0].model.as_deref(), Some("Flux"));
    assert_eq!(item.members[0].category_id.as_deref(), Some("cat-image-0"));
}

#[tokio::test]
async fn review_is_atomic_and_concurrent_decisions_cannot_overwrite_each_other() {
    let state = postgres_state().await.expect("local Postgres required");
    let pg = state.db.as_ref().unwrap();
    let publication = Publication { asset_refs: vec![], id: "atomic-review".into(), source_id: "local".into(), status: "pending".into(),
        title: Some("fail-listing".into()), content: Some("正文".into()), author_email: None,
        category_id: None, model: None, kind: "prompt".into(), members: vec![] };
    state.insert_publication(&publication).await.unwrap();
    sqlx::query(&format!("ALTER TABLE \"{}\".square_items ADD CONSTRAINT simulate_failure CHECK (title <> 'fail-listing')", pg.schema))
        .execute(&pg.pool).await.unwrap();
    assert!(matches!(state.set_publication_status(&publication.id, "approved").await, Err(StatusCode::INTERNAL_SERVER_ERROR)));
    assert_eq!(state.pending_publications().await.unwrap().len(), 1);
    assert!(state.all_items().await.unwrap().is_empty());
    sqlx::query(&format!("ALTER TABLE \"{}\".square_items DROP CONSTRAINT simulate_failure", pg.schema))
        .execute(&pg.pool).await.unwrap();
    let (first, second) = tokio::join!(state.set_publication_status(&publication.id, "approved"), state.set_publication_status(&publication.id, "approved"));
    assert!(first.is_ok() && second.is_ok());
    assert_eq!(state.all_items().await.unwrap().len(), 1);
    assert!(matches!(state.set_publication_status(&publication.id, "rejected").await, Err(StatusCode::CONFLICT)));
    let mut conflict = publication;
    conflict.id = "opposite".into();
    state.insert_publication(&conflict).await.unwrap();
    let (approve, reject) = tokio::join!(state.set_publication_status(&conflict.id, "approved"), state.set_publication_status(&conflict.id, "rejected"));
    assert_ne!(approve.is_ok(), reject.is_ok());
    assert!(matches!(approve, Err(StatusCode::CONFLICT)) || matches!(reject, Err(StatusCode::CONFLICT)));
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
        reference: Some(serde_json::json!({"author":"source author","license":"CC0 1.0","images":[]})),
        id: "sq-1".into(),
        title: "自然光群像".into(),
        kind: "prompt".into(),
        excerpt: None,
        model: None,
        category_id: None,
        member_count: None,
        content: Some("body".into()),
        members: vec![],
    }])
    .await
    .unwrap();
    let router = app(state.clone());
    for uri in ["/v1/square/items/sq-1", "/v1/square/items/sq-1/content"] {
        let response = router.clone().oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(payload["reference"]["author"], "source author");
    }
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

#[tokio::test]
async fn redeem_survives_new_appstate_on_postgres() {
    let Some(state) = postgres_state().await else {
        panic!("expected local Postgres at postgres://pl:pl@127.0.0.1:5432/promptark");
    };
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("dev@promptark.local", Some("devpass"), "user")
        .await
        .unwrap();
    pg.seed_unused_redeem_code("PREVIEW-PRO").await.unwrap();
    let router = app(state.clone());
    let session = login_json(&router, "dev@promptark.local", "devpass").await;
    let redeemed = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/redeem")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", session.access_token),
                )
                .body(Body::from(r#"{"code":"PREVIEW-PRO"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(redeemed.status(), StatusCode::OK);
    let fresh = AppState {
        db: state.db.clone(),
        ..AppState::default()
    };
    let listed = app(fresh)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/billing/status")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["pro"], true);
}

fn stripe_signature(secret: &str, payload: &str, timestamp: i64) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(format!("{timestamp}.{payload}").as_bytes());
    let digest = mac.finalize().into_bytes();
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("t={timestamp},v1={hex}")
}

#[tokio::test]
async fn webhook_survives_new_appstate_on_postgres() {
    let Some(state) = postgres_state().await else {
        panic!("expected local Postgres at postgres://pl:pl@127.0.0.1:5432/promptark");
    };
    let state = state.with_webhook_secret("whsec_preview");
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("dev@promptark.local", Some("devpass"), "user")
        .await
        .unwrap();
    let payload = serde_json::json!({
        "type": "checkout.session.completed",
        "data": {
            "object": {
                "client_reference_id": "dev@promptark.local",
                "payment_status": "paid"
            }
        }
    })
    .to_string();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let router = app(state.clone());
    let session = login_json(&router, "dev@promptark.local", "devpass").await;
    let accepted = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/webhook")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    "Stripe-Signature",
                    stripe_signature("whsec_preview", &payload, timestamp),
                )
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(accepted.status(), StatusCode::OK);
    let fresh = AppState {
        db: state.db.clone(),
        ..AppState::default()
    };
    let listed = app(fresh)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/billing/status")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["pro"], true);
}

#[tokio::test]
async fn anonymous_download_count_survives_new_appstate_on_postgres() {
    let Some(state) = postgres_state().await else {
        panic!("expected local Postgres at postgres://pl:pl@127.0.0.1:5432/promptark");
    };
    let pg = state.db.as_ref().unwrap();
    pg.insert_item(&SquareItem {
        reference: None,
        id: "sq-hot".into(),
        title: "Zed".into(),
        kind: "prompt".into(),
        excerpt: None,
        model: None,
        category_id: None,
        member_count: None,
        content: Some("body".into()),
        members: vec![],
    })
    .await
    .unwrap();
    pg.insert_item(&SquareItem {
        reference: None,
        id: "sq-cold".into(),
        title: "Alpha".into(),
        kind: "prompt".into(),
        excerpt: None,
        model: None,
        category_id: None,
        member_count: None,
        content: Some("body".into()),
        members: vec![],
    })
    .await
    .unwrap();
    let counted = app(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/square/items/sq-hot/downloads")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(counted.status(), StatusCode::OK);
    let fresh = AppState {
        db: state.db.clone(),
        ..AppState::default()
    };
    let listed = app(fresh)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/square/items?sort=hot")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
    let titles: Vec<String> = payload.items.into_iter().map(|item| item.title).collect();
    assert_eq!(titles, vec!["Zed", "Alpha"]);
}
