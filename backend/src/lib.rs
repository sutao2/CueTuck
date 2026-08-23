use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
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
    access: Arc<Mutex<HashMap<String, String>>>,
    refresh: Arc<Mutex<HashMap<String, String>>>,
    items: Arc<Mutex<Vec<SquareItem>>>,
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

pub fn hash_password(password: &str) -> String {
    format!("{:x}", Sha256::digest(password.as_bytes()))
}

impl AppState {
    pub fn with_user(email: &str, password: &str) -> Self {
        let state = Self::default();
        state
            .users
            .lock()
            .expect("users")
            .insert(email.to_string(), hash_password(password));
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
            },
            SquareItem {
                id: "col-portrait".into(),
                title: "人像灵感合集".into(),
                kind: "collection".into(),
                excerpt: Some("9 个真实人像参考与摄影提示词。".into()),
                model: None,
                member_count: Some(9),
            },
        ];
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/v1/session", post(create_session).delete(delete_session))
        .route("/v1/square/items", get(list_square_items))
        .with_state(state)
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
        }]));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/square/items?sort=recommended")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: SquareListResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload.items[0].title, "自然光群像");
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
