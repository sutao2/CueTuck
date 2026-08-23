use axum::extract::{Path, Query, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct AppState {
    users: Arc<Mutex<HashMap<String, String>>>,
    roles: Arc<Mutex<HashMap<String, String>>>,
    access: Arc<Mutex<HashMap<String, String>>>,
    refresh: Arc<Mutex<HashMap<String, String>>>,
    items: Arc<Mutex<Vec<SquareItem>>>,
    publications: Arc<Mutex<Vec<Publication>>>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionResponse {
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SquareItem {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub excerpt: Option<String>,
    pub model: Option<String>,
    pub member_count: Option<i64>,
    #[serde(default, skip_serializing)]
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SquareContentResponse {
    pub id: String,
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct PublicationRequest {
    pub source_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Publication {
    pub id: String,
    pub source_id: String,
    pub status: String,
}

#[derive(Deserialize, Default)]
pub struct SquareListQuery {
    pub sort: Option<String>,
    pub q: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SquareListResponse {
    pub items: Vec<SquareItem>,
}

#[derive(Serialize, Deserialize)]
pub struct AdminPublicationList {
    pub items: Vec<Publication>,
}

pub fn hash_password(password: &str) -> String {
    format!("{:x}", Sha256::digest(password.as_bytes()))
}

impl AppState {
    pub fn with_user(email: &str, password: &str) -> Self {
        Self::with_users(&[(email, password, "user")])
    }

    pub fn with_users(users: &[(&str, &str, &str)]) -> Self {
        let state = Self::default();
        {
            let mut hashes = state.users.lock().expect("users");
            let mut roles = state.roles.lock().expect("roles");
            for (email, password, role) in users {
                hashes.insert((*email).to_string(), hash_password(password));
                roles.insert((*email).to_string(), (*role).to_string());
            }
        }
        state
    }

    pub fn with_square_items(items: Vec<SquareItem>) -> Self {
        let state = Self::default();
        *state.items.lock().expect("items") = items;
        state
    }

    pub fn seed_square_demo(&self) {
        *self.items.lock().expect("items") = vec![
            SquareItem {
                id: "sq-1".into(),
                title: "自然光群像".into(),
                kind: "prompt".into(),
                excerpt: Some("清透蓝天下的多元人物群像。".into()),
                model: Some("Flux".into()),
                member_count: None,
                content: Some("清透蓝天下的多元人物群像。".into()),
            },
            SquareItem {
                id: "col-portrait".into(),
                title: "人像灵感合集".into(),
                kind: "collection".into(),
                excerpt: Some("9 个真实人像参考与摄影提示词。".into()),
                model: None,
                member_count: Some(9),
                content: None,
            },
        ];
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/v1/session", post(create_session).delete(delete_session))
        .route("/v1/square/items", get(list_square_items))
        .route("/v1/square/items/:id/content", get(get_square_item_content))
        .route("/v1/publications", post(create_publication))
        .route("/v1/admin/publications", get(list_admin_publications))
        .route(
            "/v1/admin/publications/:id/approve",
            post(approve_publication),
        )
        .route(
            "/v1/admin/publications/:id/reject",
            post(reject_publication),
        )
        .layer(middleware::from_fn(allow_local_preview_cors))
        .with_state(state)
}

const PREVIEW_ORIGINS: &[&str] = &["http://localhost:1420", "http://127.0.0.1:1420"];

async fn allow_local_preview_cors(request: Request, next: Next) -> Response {
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .filter(|value| PREVIEW_ORIGINS.contains(value))
        .map(str::to_string);
    if request.method() == Method::OPTIONS {
        let mut response = Response::new(axum::body::Body::empty());
        *response.status_mut() = StatusCode::NO_CONTENT;
        apply_cors(response.headers_mut(), origin.as_deref());
        return response;
    }
    let mut response = next.run(request).await;
    apply_cors(response.headers_mut(), origin.as_deref());
    response
}

fn apply_cors(headers: &mut HeaderMap, origin: Option<&str>) {
    if let Some(origin) = origin {
        if let Ok(value) = HeaderValue::from_str(origin) {
            headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
        }
    }
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("authorization, content-type"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
}

async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let email = body.email.trim().to_string();
    if email.is_empty() || body.password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let expected = state
        .users
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .get(&email)
        .cloned();
    if expected.as_deref() != Some(hash_password(&body.password).as_str()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let access_token = format!("acc.{}", Uuid::new_v4());
    let refresh_token = format!("ref.{}", Uuid::new_v4());
    state
        .access
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .insert(access_token.clone(), email.clone());
    state
        .refresh
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .insert(refresh_token.clone(), email.clone());
    Ok(Json(SessionResponse {
        email,
        access_token,
        refresh_token,
    }))
}

async fn delete_session(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED;
    };
    if token.starts_with("ref.") {
        return StatusCode::UNAUTHORIZED;
    }
    let email = state.access.lock().ok().and_then(|mut map| map.remove(&token));
    if email.is_none() {
        return StatusCode::UNAUTHORIZED;
    }
    StatusCode::NO_CONTENT
}

async fn list_square_items(
    State(state): State<AppState>,
    Query(query): Query<SquareListQuery>,
) -> Json<SquareListResponse> {
    let sort = query.sort.unwrap_or_default();
    if sort == "favorites" || sort == "收藏" {
        return Json(SquareListResponse { items: vec![] });
    }
    let needle = query.q.unwrap_or_default().trim().to_lowercase();
    let items = state
        .items
        .lock()
        .map(|rows| {
            rows
                .iter()
                .filter(|item| needle.is_empty() || item.title.to_lowercase().contains(&needle))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    Json(SquareListResponse { items })
}

async fn create_publication(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PublicationRequest>,
) -> Result<Json<Publication>, StatusCode> {
    let Some(token) = bearer_token(&headers) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    if token.starts_with("ref.") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let email = state
        .access
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .get(&token)
        .cloned();
    if email.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let source_id = body.source_id.trim().to_string();
    if source_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let publication = Publication {
        id: format!("pub.{}", Uuid::new_v4()),
        source_id,
        status: "pending".into(),
    };
    state
        .publications
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .push(publication.clone());
    Ok(Json(publication))
}

fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<String, StatusCode> {
    let Some(token) = bearer_token(headers) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    if token.starts_with("ref.") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let email = state
        .access
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .get(&token)
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let role = state
        .roles
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .get(&email)
        .cloned()
        .unwrap_or_else(|| "user".into());
    if role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(email)
}

async fn list_admin_publications(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdminPublicationList>, StatusCode> {
    require_admin(&state, &headers)?;
    let items = state
        .publications
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .iter()
        .filter(|row| row.status == "pending")
        .cloned()
        .collect();
    Ok(Json(AdminPublicationList { items }))
}

async fn approve_publication(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Publication>, StatusCode> {
    set_publication_status(&state, &headers, &id, "approved")
}

async fn reject_publication(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Publication>, StatusCode> {
    set_publication_status(&state, &headers, &id, "rejected")
}

fn set_publication_status(
    state: &AppState,
    headers: &HeaderMap,
    id: &str,
    status: &str,
) -> Result<Json<Publication>, StatusCode> {
    require_admin(state, headers)?;
    let mut rows = state
        .publications
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let publication = rows
        .iter_mut()
        .find(|row| row.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    publication.status = status.into();
    Ok(Json(publication.clone()))
}

async fn get_square_item_content(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SquareContentResponse>, StatusCode> {
    let items = state
        .items
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let item = items
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(SquareContentResponse {
        id: item.id,
        title: item.title,
        content: item.content.unwrap_or_default(),
    }))
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    value.strip_prefix("Bearer ").map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use tower::ServiceExt;

    fn app_with_dev_user() -> Router {
        app(AppState::with_user("dev@promptark.local", "devpass"))
    }

    async fn login(app: Router) -> (Router, SessionResponse) {
        login_as(app, "dev@promptark.local", "devpass").await
    }

    async fn login_as(app: Router, email: &str, password: &str) -> (Router, SessionResponse) {
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

    fn app_with_user_and_admin() -> Router {
        app(AppState::with_users(&[
            ("dev@promptark.local", "devpass", "user"),
            ("admin@promptark.local", "adminpass", "admin"),
        ]))
    }

    async fn publish(app: &Router, access_token: &str, source_id: &str) -> String {
        let accepted = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/publications")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
                    .body(Body::from(format!(r#"{{"source_id":"{source_id}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(accepted.status(), StatusCode::OK);
        let body = to_bytes(accepted.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        payload["id"].as_str().unwrap().to_string()
    }

    #[tokio::test]
    async fn create_session_isolates_access_and_refresh() {
        let (_app, session) = login(app_with_dev_user()).await;
        assert!(session.access_token.starts_with("acc."));
        assert!(session.refresh_token.starts_with("ref."));
        assert_ne!(session.access_token, session.refresh_token);
        assert_eq!(session.email, "dev@promptark.local");
    }

    #[tokio::test]
    async fn rejects_wrong_password() {
        let response = app_with_dev_user()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/session")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"email":"dev@promptark.local","password":"nope"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn delete_session_rejects_refresh_as_bearer() {
        let (app, session) = login(app_with_dev_user()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/v1/session")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.refresh_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn lists_square_items_without_login() {
        let app = app(AppState::with_square_items(vec![SquareItem {
            id: "sq-1".into(),
            title: "自然光群像".into(),
            kind: "prompt".into(),
            excerpt: None,
            model: None,
            member_count: None,
            content: None,
        }]));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items?sort=recommended")
                    .header(header::ORIGIN, "http://localhost:1420")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
            "http://localhost:1420"
        );
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload.items[0].title, "自然光群像");
    }

    #[tokio::test]
    async fn serves_square_item_content_without_login() {
        let app = app(AppState::with_square_items(vec![SquareItem {
            id: "sq-1".into(),
            title: "自然光群像".into(),
            kind: "prompt".into(),
            excerpt: None,
            model: None,
            member_count: None,
            content: Some("清透蓝天下的多元人物群像。".into()),
        }]));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items/sq-1/content")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["id"], "sq-1");
        assert_eq!(payload["title"], "自然光群像");
        assert_eq!(payload["content"], "清透蓝天下的多元人物群像。");
    }

    #[tokio::test]
    async fn create_publication_requires_access_and_keeps_pending() {
        let (app, session) = login(app_with_dev_user()).await;
        let denied = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/publications")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"source_id":"mem-1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        let refresh_denied = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/publications")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.refresh_token))
                    .body(Body::from(r#"{"source_id":"mem-1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(refresh_denied.status(), StatusCode::UNAUTHORIZED);
        let empty = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/publications")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::from(r#"{"source_id":"   "}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(empty.status(), StatusCode::BAD_REQUEST);
        let accepted = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/publications")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::from(r#"{"source_id":"mem-1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(accepted.status(), StatusCode::OK);
        let body = to_bytes(accepted.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["status"], "pending");
        assert_eq!(payload["source_id"], "mem-1");
    }

    #[tokio::test]
    async fn regular_token_cannot_review_publication() {
        let (app, user) = login(app_with_user_and_admin()).await;
        let id = publish(&app, &user.access_token, "mem-1").await;
        let denied = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/admin/publications/{id}/approve"))
                    .header(header::AUTHORIZATION, format!("Bearer {}", user.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
        let listed = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/admin/publications")
                    .header(header::AUTHORIZATION, format!("Bearer {}", user.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(listed.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn admin_lists_pending_and_can_approve() {
        let (app, user) = login(app_with_user_and_admin()).await;
        let id = publish(&app, &user.access_token, "mem-1").await;
        let (app, admin) = login_as(app, "admin@promptark.local", "adminpass").await;
        let listed = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/admin/publications")
                    .header(header::AUTHORIZATION, format!("Bearer {}", admin.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(listed.status(), StatusCode::OK);
        let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["items"][0]["id"], id);
        assert_eq!(payload["items"][0]["source_id"], "mem-1");
        assert_eq!(payload["items"][0]["status"], "pending");
        let approved = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/admin/publications/{id}/approve"))
                    .header(header::AUTHORIZATION, format!("Bearer {}", admin.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(approved.status(), StatusCode::OK);
        let body = to_bytes(approved.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["status"], "approved");
        assert_eq!(payload["source_id"], "mem-1");
        let listed = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/admin/publications")
                    .header(header::AUTHORIZATION, format!("Bearer {}", admin.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["items"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn admin_rejects_publication() {
        let (app, user) = login(app_with_user_and_admin()).await;
        let id = publish(&app, &user.access_token, "mem-2").await;
        let (app, admin) = login_as(app, "admin@promptark.local", "adminpass").await;
        let rejected = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/admin/publications/{id}/reject"))
                    .header(header::AUTHORIZATION, format!("Bearer {}", admin.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::OK);
        let body = to_bytes(rejected.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["status"], "rejected");
        assert_eq!(payload["id"], id);
    }

    #[tokio::test]
    async fn delete_session_accepts_access() {
        let (app, session) = login(app_with_dev_user()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/v1/session")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}

#[cfg(test)]
mod container_tests {
    #[tokio::test]
    #[ignore = "needs Docker; enable when Postgres store lands"]
    async fn session_api_postgres_container() {
        panic!("Docker Testcontainers 未在本机启用");
    }
}
