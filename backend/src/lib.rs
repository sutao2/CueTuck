mod media;
mod oauth;
mod password;
mod postgres;
mod state;

use axum::extract::{Path, Query, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use oauth::OAuthSettings;
use postgres::Pg;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub use password::{hash_password, verify_password};

#[derive(Clone)]
pub struct AppState {
    users: Arc<Mutex<HashMap<String, String>>>,
    roles: Arc<Mutex<HashMap<String, String>>>,
    access: Arc<Mutex<HashMap<String, String>>>,
    refresh: Arc<Mutex<HashMap<String, String>>>,
    items: Arc<Mutex<Vec<SquareItem>>>,
    publications: Arc<Mutex<Vec<Publication>>>,
    square_public: Arc<Mutex<bool>>,
    favorites: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    db: Option<Pg>,
    redis: Option<redis::aio::ConnectionManager>,
    pub oauth: OAuthSettings,
    pub media: Option<media::MediaConfig>,
    oauth_flows: Arc<Mutex<HashMap<String, String>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            roles: Arc::new(Mutex::new(HashMap::new())),
            access: Arc::new(Mutex::new(HashMap::new())),
            refresh: Arc::new(Mutex::new(HashMap::new())),
            items: Arc::new(Mutex::new(Vec::new())),
            publications: Arc::new(Mutex::new(Vec::new())),
            square_public: Arc::new(Mutex::new(true)),
            favorites: Arc::new(Mutex::new(HashMap::new())),
            db: None,
            redis: None,
            oauth: OAuthSettings::default(),
            media: None,
            oauth_flows: Arc::new(Mutex::new(HashMap::new())),
        }
    }
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
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Publication {
    pub id: String,
    pub source_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_email: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct SquareListQuery {
    pub sort: Option<String>,
    pub q: Option<String>,
    pub model: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SquareListResponse {
    pub items: Vec<SquareItem>,
}

#[derive(Serialize, Deserialize)]
pub struct AdminPublicationList {
    pub items: Vec<Publication>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub email: String,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct AdminUserList {
    pub items: Vec<AdminUser>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AdminSettings {
    pub square_public: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AdminMe {
    pub email: String,
    pub role: String,
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
        .route("/v1/health", get(health))
        .route("/v1/session", post(create_session).delete(delete_session))
        .route("/v1/session/refresh", post(refresh_session))
        .route("/v1/session/oauth/providers", get(oauth::list_providers))
        .route("/v1/session/oauth/callback", get(oauth::callback))
        .route(
            "/v1/session/oauth/session/:flow_id",
            get(oauth::poll_session),
        )
        .route("/v1/session/oauth/:provider", get(oauth::start))
        .route("/v1/media/upload", post(media::upload))
        .route("/v1/media/:id/url", get(media::signed_url))
        .route("/v1/square/items", get(list_square_items))
        .route("/v1/square/items/:id/content", get(get_square_item_content))
        .route("/v1/square/items/:id", get(get_square_item))
        .route("/v1/publications", post(create_publication))
        .route("/v1/publications/mine", get(list_my_publications))
        .route("/v1/favorites", get(list_favorites))
        .route("/v1/favorites/:id", put(put_favorite).delete(delete_favorite))
        .route("/v1/admin/me", get(get_admin_me))
        .route("/v1/admin/publications", get(list_admin_publications))
        .route(
            "/v1/admin/publications/:id/approve",
            post(approve_publication),
        )
        .route(
            "/v1/admin/publications/:id/reject",
            post(reject_publication),
        )
        .route("/v1/admin/users", get(list_admin_users))
        .route("/v1/admin/settings", get(get_admin_settings).put(put_admin_settings))
        .layer(middleware::from_fn(allow_local_preview_cors))
        .with_state(state)
}

const PREVIEW_ORIGINS: &[&str] = &[
    "http://localhost:1420",
    "http://127.0.0.1:1420",
    "http://localhost:5174",
    "http://127.0.0.1:5174",
    "http://localhost:5175",
    "http://127.0.0.1:5175",
];

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

async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let postgres = state.ping_db().await;
    let redis = state.ping_redis().await;
    let minio = match &state.media {
        Some(media) => media.ping().await,
        None => false,
    };
    Json(serde_json::json!({
        "postgres": postgres,
        "redis": redis,
        "minio": minio,
    }))
}

async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let email = body.email.trim().to_string();
    if email.is_empty() || body.password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    state.verify_login(&email, &body.password).await?;
    Ok(Json(state.issue_session(email).await?))
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

async fn refresh_session(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let presented = body.refresh_token.trim().to_string();
    if !presented.starts_with("ref.") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let email = state.rotate_refresh(&presented).await?;
    Ok(Json(state.issue_session(email).await?))
}

async fn delete_session(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED;
    };
    if token.starts_with("ref.") {
        return StatusCode::UNAUTHORIZED;
    }
    match state.revoke_access(&token).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::UNAUTHORIZED,
        Err(status) => status,
    }
}

async fn list_square_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SquareListQuery>,
) -> Json<SquareListResponse> {
    let square_public = state.square_public().await.unwrap_or(true);
    if !square_public {
        return Json(SquareListResponse { items: vec![] });
    }
    let sort = query.sort.unwrap_or_default();
    let needle = query.q.unwrap_or_default().trim().to_lowercase();
    let model = query.model.unwrap_or_default();
    let mut items = state
        .all_items()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|item| needle.is_empty() || item.title.to_lowercase().contains(&needle))
        .filter(|item| model.is_empty() || item.model.as_deref() == Some(model.as_str()))
        .collect::<Vec<_>>();
    if sort == "favorites" || sort == "收藏" {
        let Some(email) = optional_access_email(&state, &headers).await else {
            return Json(SquareListResponse { items: vec![] });
        };
        let ids = state.favorite_ids(&email).await.unwrap_or_default();
        items.retain(|item| ids.iter().any(|id| id == &item.id));
        return Json(SquareListResponse { items });
    }
    apply_preview_sort(&mut items, &sort);
    Json(SquareListResponse { items })
}

fn apply_preview_sort(items: &mut [SquareItem], sort: &str) {
    match sort {
        "latest" | "最新" => items.reverse(),
        "hot" | "热门" => items.sort_by(|left, right| left.title.cmp(&right.title)),
        _ => {}
    }
}

async fn optional_access_email(state: &AppState, headers: &HeaderMap) -> Option<String> {
    let token = bearer_token(headers)?;
    if token.starts_with("ref.") {
        return None;
    }
    state.email_for_access(&token).await.ok().flatten()
}

async fn create_publication(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PublicationRequest>,
) -> Result<Json<Publication>, StatusCode> {
    let author_email = require_user(&state, &headers).await?;
    let source_id = body.source_id.trim().to_string();
    if source_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let publication = Publication {
        id: format!("pub.{}", Uuid::new_v4()),
        source_id,
        status: "pending".into(),
        title: body.title.filter(|value| !value.trim().is_empty()),
        content: body.content,
        author_email: Some(author_email),
    };
    state.insert_publication(&publication).await?;
    Ok(Json(publication))
}

async fn list_my_publications(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdminPublicationList>, StatusCode> {
    let email = require_user(&state, &headers).await?;
    Ok(Json(AdminPublicationList {
        items: state.publications_for(&email).await?,
    }))
}

pub(crate) async fn require_user(state: &AppState, headers: &HeaderMap) -> Result<String, StatusCode> {
    let Some(token) = bearer_token(headers) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    if token.starts_with("ref.") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    state
        .email_for_access(&token)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)
}

#[derive(Serialize)]
struct FavoriteList {
    items: Vec<SquareItem>,
}

async fn list_favorites(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<FavoriteList>, StatusCode> {
    let email = require_user(&state, &headers).await?;
    let ids = state.favorite_ids(&email).await?;
    let items = state
        .all_items()
        .await?
        .into_iter()
        .filter(|item| ids.iter().any(|id| id == &item.id))
        .collect();
    Ok(Json(FavoriteList { items }))
}

async fn put_favorite(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<SquareItem>, StatusCode> {
    let email = require_user(&state, &headers).await?;
    let item = state.get_item(&id).await?.ok_or(StatusCode::NOT_FOUND)?;
    state.put_favorite(&email, &id).await?;
    Ok(Json(item))
}

async fn delete_favorite(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> StatusCode {
    let Ok(email) = require_user(&state, &headers).await else {
        return StatusCode::UNAUTHORIZED;
    };
    match state.delete_favorite(&email, &id).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(status) => status,
    }
}

async fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<String, StatusCode> {
    let email = require_user(state, headers).await?;
    if state.role_of(&email).await? != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(email)
}

async fn get_admin_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdminMe>, StatusCode> {
    let email = require_admin(&state, &headers).await?;
    let role = state.role_of(&email).await?;
    Ok(Json(AdminMe { email, role }))
}

async fn list_admin_publications(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdminPublicationList>, StatusCode> {
    require_admin(&state, &headers).await?;
    Ok(Json(AdminPublicationList {
        items: state.pending_publications().await?,
    }))
}

async fn approve_publication(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Publication>, StatusCode> {
    set_publication_status(&state, &headers, &id, "approved").await
}

async fn reject_publication(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Publication>, StatusCode> {
    set_publication_status(&state, &headers, &id, "rejected").await
}

async fn get_admin_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdminSettings>, StatusCode> {
    require_admin(&state, &headers).await?;
    Ok(Json(AdminSettings {
        square_public: state.square_public().await?,
    }))
}

async fn put_admin_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminSettings>,
) -> Result<Json<AdminSettings>, StatusCode> {
    require_admin(&state, &headers).await?;
    state.set_square_public(body.square_public).await?;
    Ok(Json(body))
}

async fn list_admin_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdminUserList>, StatusCode> {
    require_admin(&state, &headers).await?;
    Ok(Json(AdminUserList {
        items: state.list_users().await?,
    }))
}

async fn set_publication_status(
    state: &AppState,
    headers: &HeaderMap,
    id: &str,
    status: &str,
) -> Result<Json<Publication>, StatusCode> {
    require_admin(state, headers).await?;
    Ok(Json(state.set_publication_status(id, status).await?))
}

async fn get_square_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SquareItem>, StatusCode> {
    state
        .get_item(&id)
        .await?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn get_square_item_content(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SquareContentResponse>, StatusCode> {
    let item = state.get_item(&id).await?.ok_or(StatusCode::NOT_FOUND)?;
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
    async fn allows_admin_web_preview_cors() {
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
                    .uri("/v1/square/items")
                    .header(header::ORIGIN, "http://localhost:5174")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
            "http://localhost:5174"
        );
    }

    #[tokio::test]
    async fn allows_web_preview_cors() {
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
                    .uri("/v1/square/items")
                    .header(header::ORIGIN, "http://localhost:5175")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
            "http://localhost:5175"
        );
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
    async fn serves_square_item_without_login() {
        let app = app(AppState::with_square_items(vec![SquareItem {
            id: "sq-1".into(),
            title: "自然光群像".into(),
            kind: "prompt".into(),
            excerpt: Some("摘要".into()),
            model: Some("Flux".into()),
            member_count: None,
            content: Some("不该出现在详情里".into()),
        }]));
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items/sq-1")
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
        assert_eq!(payload["kind"], "prompt");
        assert_eq!(payload["excerpt"], "摘要");
        assert!(payload.get("content").is_none() || payload["content"].is_null());
        let missing = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items/no-such")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn sorts_recommended_latest_and_hot_apart() {
        let app = app(AppState::with_square_items(vec![
            SquareItem {
                id: "a".into(),
                title: "Beta".into(),
                kind: "prompt".into(),
                excerpt: None,
                model: Some("Flux".into()),
                member_count: None,
                content: None,
            },
            SquareItem {
                id: "b".into(),
                title: "Alpha".into(),
                kind: "prompt".into(),
                excerpt: None,
                model: Some("Midjourney".into()),
                member_count: None,
                content: None,
            },
            SquareItem {
                id: "c".into(),
                title: "Gamma".into(),
                kind: "prompt".into(),
                excerpt: None,
                model: Some("Flux".into()),
                member_count: None,
                content: None,
            },
        ]));
        async fn titles(app: &Router, uri: &str) -> Vec<String> {
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
        let recommended = titles(&app, "/v1/square/items?sort=recommended").await;
        let latest = titles(&app, "/v1/square/items?sort=latest").await;
        let hot = titles(&app, "/v1/square/items?sort=hot").await;
        assert_eq!(recommended, vec!["Beta", "Alpha", "Gamma"]);
        assert_eq!(latest, vec!["Gamma", "Alpha", "Beta"]);
        assert_eq!(hot, vec!["Alpha", "Beta", "Gamma"]);
        assert_ne!(recommended, latest);
        assert_ne!(recommended, hot);
        assert_ne!(latest, hot);
        let flux = titles(&app, "/v1/square/items?model=Flux").await;
        assert_eq!(flux, vec!["Beta", "Gamma"]);
        assert_eq!(
            titles(&app, "/v1/square/items?sort=%E6%8E%A8%E8%8D%90").await,
            recommended
        );
        assert_eq!(
            titles(&app, "/v1/square/items?sort=%E6%9C%80%E6%96%B0").await,
            latest
        );
        assert_eq!(
            titles(&app, "/v1/square/items?sort=%E7%83%AD%E9%97%A8").await,
            hot
        );
    }

    #[tokio::test]
    async fn favorites_sort_requires_login() {
        let (app, session) = login(app_with_square_user()).await;
        let anonymous = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items?sort=favorites")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(anonymous.status(), StatusCode::OK);
        let body = to_bytes(anonymous.into_body(), usize::MAX).await.unwrap();
        let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
        assert!(payload.items.is_empty());
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/v1/favorites/sq-1")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let logged_in = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items?sort=favorites")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(logged_in.status(), StatusCode::OK);
        let body = to_bytes(logged_in.into_body(), usize::MAX).await.unwrap();
        let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload.items.len(), 1);
        assert_eq!(payload.items[0].id, "sq-1");
        let chinese = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items?sort=%E6%94%B6%E8%97%8F")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(chinese.status(), StatusCode::OK);
        let body = to_bytes(chinese.into_body(), usize::MAX).await.unwrap();
        let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload.items[0].id, "sq-1");
    }

    #[tokio::test]
    async fn approve_without_snapshot_does_not_list_on_square() {
        let (app, user) = login(app_with_user_and_admin()).await;
        let id = publish(&app, &user.access_token, "mem-1").await;
        let (app, admin) = login_as(app, "admin@promptark.local", "adminpass").await;
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
        let listed = app
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
        assert!(payload.items.is_empty());
    }

    #[tokio::test]
    async fn approve_with_snapshot_lists_on_square() {
        let (app, user) = login(app_with_user_and_admin()).await;
        let accepted = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/publications")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, format!("Bearer {}", user.access_token))
                    .body(Body::from(
                        r#"{"source_id":"mem-1","title":"新稿","content":"快照正文"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(accepted.status(), StatusCode::OK);
        let body = to_bytes(accepted.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = payload["id"].as_str().unwrap().to_string();
        let (app, admin) = login_as(app, "admin@promptark.local", "adminpass").await;
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
        let listed = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(listed.status(), StatusCode::OK);
        let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
        let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
        assert!(payload.items.iter().any(|item| item.title == "新稿"));
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
        assert_eq!(payload["author_email"], "dev@promptark.local");
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
    async fn regular_token_cannot_list_users() {
        let (app, user) = login(app_with_user_and_admin()).await;
        let denied = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/admin/users")
                    .header(header::AUTHORIZATION, format!("Bearer {}", user.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn admin_lists_user_emails_and_roles() {
        let (app, admin) = login_as(app_with_user_and_admin(), "admin@promptark.local", "adminpass").await;
        let listed = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/admin/users")
                    .header(header::AUTHORIZATION, format!("Bearer {}", admin.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(listed.status(), StatusCode::OK);
        let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let items = payload["items"].as_array().unwrap();
        let emails: Vec<&str> = items.iter().map(|row| row["email"].as_str().unwrap()).collect();
        assert!(emails.contains(&"dev@promptark.local"));
        assert!(emails.contains(&"admin@promptark.local"));
        let admin_row = items
            .iter()
            .find(|row| row["email"] == "admin@promptark.local")
            .unwrap();
        let user_row = items
            .iter()
            .find(|row| row["email"] == "dev@promptark.local")
            .unwrap();
        assert_eq!(admin_row["role"], "admin");
        assert_eq!(user_row["role"], "user");
        assert!(admin_row.get("password").is_none());
        assert!(admin_row.get("password_hash").is_none());
    }

    #[tokio::test]
    async fn regular_token_cannot_change_settings() {
        let (app, user) = login(app_with_user_and_admin()).await;
        let denied = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/v1/admin/settings")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, format!("Bearer {}", user.access_token))
                    .body(Body::from(r#"{"square_public":false}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn admin_can_close_public_square() {
        let state = AppState::with_users(&[
            ("dev@promptark.local", "devpass", "user"),
            ("admin@promptark.local", "adminpass", "admin"),
        ]);
        state.seed_square_demo();
        let (app, admin) = login_as(app(state), "admin@promptark.local", "adminpass").await;
        let open = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(open.status(), StatusCode::OK);
        let body = to_bytes(open.into_body(), usize::MAX).await.unwrap();
        let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
        assert!(!payload.items.is_empty());
        let saved = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/v1/admin/settings")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, format!("Bearer {}", admin.access_token))
                    .body(Body::from(r#"{"square_public":false}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(saved.status(), StatusCode::OK);
        let closed = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(closed.into_body(), usize::MAX).await.unwrap();
        let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
        assert!(payload.items.is_empty());
        let settings = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/admin/settings")
                    .header(header::AUTHORIZATION, format!("Bearer {}", admin.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(settings.status(), StatusCode::OK);
        let body = to_bytes(settings.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["square_public"], false);
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

    #[tokio::test]
    async fn admin_me_returns_email_and_role() {
        let (app, admin) = login_as(app_with_user_and_admin(), "admin@promptark.local", "adminpass").await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/admin/me")
                    .header(header::AUTHORIZATION, format!("Bearer {}", admin.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["email"], "admin@promptark.local");
        assert_eq!(payload["role"], "admin");
    }

    #[tokio::test]
    async fn admin_me_rejects_regular_access() {
        let (app, session) = login(app_with_user_and_admin()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/admin/me")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    fn app_with_square_user() -> Router {
        let state = AppState::with_user("dev@promptark.local", "devpass");
        state.seed_square_demo();
        app(state)
    }

    #[tokio::test]
    async fn put_favorite_lists_for_account() {
        let (app, session) = login(app_with_square_user()).await;
        let put = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/v1/favorites/sq-1")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(put.status(), StatusCode::OK);
        let listed = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/favorites")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(listed.status(), StatusCode::OK);
        let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["items"][0]["id"], "sq-1");
    }

    #[tokio::test]
    async fn delete_favorite_removes_account_relation() {
        let (app, session) = login(app_with_square_user()).await;
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/v1/favorites/sq-1")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let deleted = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/v1/favorites/sq-1")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
        let listed = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/favorites")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
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
    async fn favorites_require_access_token() {
        let app = app_with_square_user();
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/v1/favorites/sq-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn refresh_rotates_and_invalidates_old_pair() {
        let (app, session) = login(app_with_dev_user()).await;
        let rotated = app
            .clone()
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
        assert_eq!(rotated.status(), StatusCode::OK);
        let body = to_bytes(rotated.into_body(), usize::MAX).await.unwrap();
        let next: SessionResponse = serde_json::from_slice(&body).unwrap();
        assert_ne!(next.access_token, session.access_token);
        assert_ne!(next.refresh_token, session.refresh_token);
        assert!(next.access_token.starts_with("acc."));
        assert!(next.refresh_token.starts_with("ref."));
        let old_access = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/favorites")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(old_access.status(), StatusCode::UNAUTHORIZED);
        let old_refresh = app
            .clone()
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
        assert_eq!(old_refresh.status(), StatusCode::UNAUTHORIZED);
        let new_access = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/favorites")
                    .header(header::AUTHORIZATION, format!("Bearer {}", next.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(new_access.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn password_uses_argon2id_not_sha256() {
        let encoded = hash_password("devpass");
        assert!(encoded.starts_with("$argon2"));
        assert_ne!(encoded.len(), 64);
        assert!(verify_password("devpass", &encoded));
        assert!(!verify_password("nope", &encoded));
        let sha256_hex = {
            use sha2::{Digest, Sha256};
            format!("{:x}", Sha256::digest(b"devpass"))
        };
        assert!(!verify_password("devpass", &sha256_hex));
    }

    #[tokio::test]
    async fn oauth_google_redirects_when_configured() {
        let mut state = AppState::default();
        state.oauth.providers.insert(
            "google".into(),
            crate::oauth::ProviderConfig {
                client_id: "google-client".into(),
                client_secret: "google-secret".into(),
                authorization_uri: "https://accounts.example/oauth/authorize".into(),
                token_uri: "https://accounts.example/oauth/token".into(),
                user_info_uri: "https://accounts.example/oauth/userinfo".into(),
                emails_uri: None,
                redirect_uri: "http://localhost:8787/v1/session/oauth/callback".into(),
                scope: "openid email".into(),
            },
        );
        let response = app(state)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/session/oauth/google")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        let location = response
            .headers()
            .get(header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.starts_with("https://accounts.example/oauth/authorize?"));
        assert!(location.contains("client_id=google-client"));
    }

    #[tokio::test]
    async fn oauth_callback_with_mock_code_issues_session() {
        let mut state = AppState::default();
        state.oauth.providers.insert(
            "google".into(),
            crate::oauth::ProviderConfig {
                client_id: "google-client".into(),
                client_secret: "google-secret".into(),
                authorization_uri: "https://accounts.example/oauth/authorize".into(),
                token_uri: "https://accounts.example/oauth/token".into(),
                user_info_uri: "https://accounts.example/oauth/userinfo".into(),
                emails_uri: None,
                redirect_uri: "http://localhost:8787/v1/session/oauth/callback".into(),
                scope: "openid email".into(),
            },
        );
        state.oauth.mock_users.insert(
            "code-123".into(),
            crate::oauth::OAuthUser {
                provider: "google".into(),
                provider_uid: "g-1".into(),
                email: "oauth@promptark.local".into(),
            },
        );
        let start = app(state.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/session/oauth/google")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let location = start
            .headers()
            .get(header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let state_param = location
            .split("state=")
            .nth(1)
            .unwrap()
            .split('&')
            .next()
            .unwrap();
        let response = app(state)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/v1/session/oauth/callback?code=code-123&state={state_param}"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let session: SessionResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(session.email, "oauth@promptark.local");
        assert!(session.access_token.starts_with("acc."));
    }
}

#[cfg(test)]
mod persistence_tests;
