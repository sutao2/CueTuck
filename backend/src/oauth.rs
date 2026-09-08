use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::Json;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct OAuthSettings {
    pub state_secret: String,
    pub allowed_origins: Vec<String>,
    pub providers: HashMap<String, ProviderConfig>,
    pub mock_users: HashMap<String, OAuthUser>,
}

impl Default for OAuthSettings {
    fn default() -> Self {
        Self {
            state_secret: std::env::var("PROMPTARK_OAUTH_STATE_SECRET")
                .or_else(|_| std::env::var("PL_OAUTH_STATE_SECRET"))
                .unwrap_or_else(|_| "dev-only-oauth-state-secret-change-me-32bytes".into()),
            allowed_origins: std::env::var("PROMPTARK_OAUTH_WEB_MESSAGE_ORIGINS")
                .or_else(|_| std::env::var("PL_OAUTH_WEB_MESSAGE_ORIGINS"))
                .unwrap_or_else(|_| {
                    "http://localhost:1420,http://127.0.0.1:1420,http://localhost:5174,http://127.0.0.1:5174,http://localhost:5175,http://127.0.0.1:5175".into()
                })
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            providers: ProviderConfig::from_env(),
            mock_users: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct ProviderConfig {
    pub client_id: String,
    pub client_secret: String,
    pub authorization_uri: String,
    pub token_uri: String,
    pub user_info_uri: String,
    pub emails_uri: Option<String>,
    pub redirect_uri: String,
    pub scope: String,
}

impl ProviderConfig {
    pub(crate) fn configured(name: &str, client_id: String, client_secret: String, redirect_uri: String) -> Self {
        let github = name == "github";
        Self {
            client_id, client_secret, redirect_uri,
            authorization_uri: if github { "https://github.com/login/oauth/authorize" } else { "https://accounts.google.com/o/oauth2/v2/auth" }.into(),
            token_uri: if github { "https://github.com/login/oauth/access_token" } else { "https://oauth2.googleapis.com/token" }.into(),
            user_info_uri: if github { "https://api.github.com/user" } else { "https://openidconnect.googleapis.com/v1/userinfo" }.into(),
            emails_uri: github.then(|| "https://api.github.com/user/emails".into()),
            scope: if github { "read:user user:email" } else { "openid email profile" }.into(),
        }
    }
    fn from_env() -> HashMap<String, ProviderConfig> {
        let mut map = HashMap::new();
        for name in crate::oauth_admin::PROVIDERS {
            if let Some(config) = load_provider(name) { map.insert(name.into(), config); }
        }
        map
    }
}

fn env_pair(promptark: &str, legacy: &str) -> Option<String> {
    std::env::var(promptark)
        .or_else(|_| std::env::var(legacy))
        .ok()
        .filter(|value| !value.is_empty())
}

fn load_provider(name: &str) -> Option<ProviderConfig> {
    let prefix = name.to_uppercase();
    let client_id = env_pair(
        &format!("PROMPTARK_{prefix}_CLIENT_ID"),
        &format!("PL_{prefix}_CLIENT_ID"),
    )?;
    let client_secret = env_pair(
        &format!("PROMPTARK_{prefix}_CLIENT_SECRET"),
        &format!("PL_{prefix}_CLIENT_SECRET"),
    )?;
    Some(ProviderConfig::configured(name, client_id, client_secret, env_pair(
            &format!("PROMPTARK_{prefix}_REDIRECT_URI"),
            &format!("PL_{prefix}_REDIRECT_URI"),
        )
        .unwrap_or_else(|| "http://localhost:8787/v1/session/oauth/callback".into())))
}

#[derive(Clone)]
pub struct OAuthUser {
    pub provider: String,
    pub provider_uid: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct OAuthStartQuery {
    pub response_mode: Option<String>,
    pub web_message_origin: Option<String>,
    pub flow_id: Option<String>,
}

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone)]
struct OAuthState {
    provider: String,
    response_mode: String,
    web_message_origin: String,
    flow_id: String,
}

fn sign(secret: &str, payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac key");
    mac.update(payload.as_bytes());
    URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
}

fn create_state(settings: &OAuthSettings, state: &OAuthState) -> String {
    let issued = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let nonce = Uuid::new_v4();
    let raw = format!(
        "v3|{}|{issued}|{nonce}|{}|{}|{}",
        state.provider, state.response_mode, state.web_message_origin, state.flow_id
    );
    let payload = URL_SAFE_NO_PAD.encode(raw.as_bytes());
    format!("{payload}.{}", sign(&settings.state_secret, &payload))
}

fn verify_state(settings: &OAuthSettings, state: &str) -> Result<OAuthState, StatusCode> {
    let (payload, signature) = state.split_once('.').ok_or(StatusCode::BAD_REQUEST)?;
    if sign(&settings.state_secret, payload) != signature {
        return Err(StatusCode::BAD_REQUEST);
    }
    let raw = String::from_utf8(
        URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| StatusCode::BAD_REQUEST)?,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let fields: Vec<&str> = raw.split('|').collect();
    if fields.len() != 7 || fields[0] != "v3" {
        return Err(StatusCode::BAD_REQUEST);
    }
    let issued: u64 = fields[2].parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now.saturating_sub(issued) > 600 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(OAuthState {
        provider: fields[1].into(),
        response_mode: fields[4].into(),
        web_message_origin: fields[5].into(),
        flow_id: fields[6].into(),
    })
}

fn require_origin(settings: &OAuthSettings, origin: &str) -> Result<String, StatusCode> {
    let origin = origin.trim().trim_end_matches('/').to_string();
    if settings.allowed_origins.iter().any(|item| item == &origin) {
        Ok(origin)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn list_providers(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let mut items = Vec::new();
    for provider in crate::oauth_admin::PROVIDERS {
        if state.runtime_provider(provider).await?.is_some() { items.push(provider); }
    }
    Ok(Json(json!({ "items": items })))
}

pub async fn start(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthStartQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let provider = provider.to_lowercase();
    let config = state.runtime_provider(&provider).await?.ok_or(StatusCode::NOT_FOUND)?;
    let mode = query.response_mode.unwrap_or_default();
    if !mode.is_empty() && mode != "web_message" && mode != "browser" {
        return Err(StatusCode::BAD_REQUEST);
    }
    let origin = if mode == "web_message" {
        require_origin(
            &state.oauth,
            query.web_message_origin.as_deref().unwrap_or(""),
        )?
    } else {
        String::new()
    };
    let flow_id = if mode == "browser" {
        let id = query.flow_id.unwrap_or_default();
        if id.len() < 16 {
            return Err(StatusCode::BAD_REQUEST);
        }
        id
    } else {
        String::new()
    };
    let signed = create_state(
        &state.oauth,
        &OAuthState {
            provider: provider.clone(),
            response_mode: mode,
            web_message_origin: origin,
            flow_id,
        },
    );
    let location = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        config.authorization_uri,
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&config.redirect_uri),
        urlencoding::encode(&config.scope),
        urlencoding::encode(&signed)
    );
    Ok(Redirect::temporary(&location))
}

pub async fn callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    if query.state.as_deref().is_some_and(|s|s.starts_with("admin-test.")) {
        return crate::oauth_verification::callback(&state,&query).await;
    }
    if query.error.as_deref().is_some_and(|value| !value.is_empty()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let signed = query.state.as_deref().ok_or(StatusCode::BAD_REQUEST)?;
    let parsed = verify_state(&state.oauth, signed)?;
    let code = query.code.as_deref().filter(|value| !value.is_empty());
    let user = match code {
        Some(code) => fetch_user(&state, &parsed.provider, code).await?,
        None => return Err(StatusCode::BAD_REQUEST),
    };
    let session = state.oauth_login(&user).await?;
    if parsed.response_mode == "web_message" {
        let payload = serde_json::to_string(&session).unwrap_or_else(|_| "{}".into());
        let origin = parsed.web_message_origin;
        let html = format!(
            "<!doctype html><script>window.opener && window.opener.postMessage({{type:'prompt-launcher:oauth', result:{payload}}}, '{origin}');</script>"
        );
        return Ok(Html(html).into_response());
    }
    if parsed.response_mode == "browser" {
        state
            .put_oauth_flow(&parsed.flow_id, serde_json::to_string(&session).unwrap())
            .await;
        return Ok(StatusCode::NO_CONTENT.into_response());
    }
    Ok(Json(session).into_response())
}

pub async fn poll_session(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let Some(raw) = state.get_oauth_flow(&flow_id).await else {
        return Ok(Json(json!({ "status": "pending" })));
    };
    let value: Value = serde_json::from_str(&raw).unwrap_or(json!({}));
    Ok(Json(json!({ "status": "ready", "session": value })))
}

async fn fetch_user(
    state: &AppState,
    provider: &str,
    code: &str,
) -> Result<OAuthUser, StatusCode> {
    let config = state.runtime_provider(provider).await?.ok_or(StatusCode::NOT_FOUND)?;
    if let Some(user) = state.oauth.mock_users.get(code).cloned() {
        return Ok(user);
    }
    fetch_verified_user(&config,provider,code).await
}

async fn provider_json(response:reqwest::Response)->Result<Value,StatusCode>{
    if !response.status().is_success()||response.content_length().is_some_and(|n|n>262144){return Err(StatusCode::UNAUTHORIZED)}
    let mut response=response;let mut bytes=Vec::new();
    while let Some(chunk)=response.chunk().await.map_err(|_|StatusCode::UNAUTHORIZED)?{if bytes.len()+chunk.len()>262144{return Err(StatusCode::UNAUTHORIZED)}bytes.extend_from_slice(&chunk)}
    serde_json::from_slice(&bytes).map_err(|_|StatusCode::UNAUTHORIZED)
}
pub(crate) async fn fetch_verified_user(config:&ProviderConfig,provider:&str,code:&str)->Result<OAuthUser,StatusCode>{
    let client = reqwest::Client::builder().user_agent("PromptArk/0.1")
        .redirect(reqwest::redirect::Policy::none()).timeout(std::time::Duration::from_secs(20)).build().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token_response: Value = provider_json(client
        .post(&config.token_uri)
        .header(reqwest::header::ACCEPT, "application/json")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &config.redirect_uri),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
        ])
        .send()
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?).await?;
    let access = token_response
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let profile: Value = provider_json(client
        .get(&config.user_info_uri)
        .bearer_auth(access)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?).await?;
    let uid = if provider == "github" {
        profile
            .get("id")
            .map(|value| value.to_string().trim_matches('"').to_string())
    } else {
        profile
            .get("sub")
            .and_then(Value::as_str)
            .map(str::to_string)
    }
    .ok_or(StatusCode::UNAUTHORIZED)?;
    let mut email = profile
        .get("email")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if provider == "google" && profile.get("email_verified") != Some(&Value::Bool(true)) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if provider == "github" {
        email.clear();
        if let Some(uri) = &config.emails_uri {
            let emails: Value = provider_json(client
                .get(uri)
                .bearer_auth(access)
                .header(reqwest::header::ACCEPT, "application/json")
                .send()
                .await
                .map_err(|_| StatusCode::UNAUTHORIZED)?).await?;
            if let Some(items) = emails.as_array() {
                email = items
                    .iter()
                    .find(|item| item.get("primary") == Some(&Value::Bool(true)) && item.get("verified") == Some(&Value::Bool(true)))
                    .and_then(|item| item.get("email"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
            }
        }
    }
    if email.is_empty() || uid.trim().is_empty() || email.len()>254 || !email.contains('@') {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(OAuthUser {
        provider: provider.into(),
        provider_uid: uid,
        email,
    })
}

#[cfg(test)]
mod profile_tests {
    use super::*;
    use axum::{http::HeaderMap, routing::{get, post}, Router};

    async fn provider_server(profile: Value, emails: Value) -> (String, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let router = Router::new()
            .route("/token", post(|| async { Json(json!({ "access_token": "test-token" })) }))
            .route("/profile", get(move |headers: HeaderMap| { let profile = profile.clone(); async move {
                assert_eq!(headers["user-agent"], "PromptArk/0.1"); Json(profile)
            }}))
            .route("/emails", get(move || { let emails = emails.clone(); async move { Json(emails) } }));
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap(); });
        (base, task)
    }

    #[tokio::test]
    async fn oauth_accounts_require_verified_provider_emails() {
        for (provider, profile, emails, expected) in [
            ("google", json!({"sub":"1","email":"admin@example.com","email_verified":false}), json!([]), None),
            ("google", json!({"sub":"1","email":"admin@example.com","email_verified":true}), json!([]), Some("admin@example.com")),
            ("github", json!({"id":1,"email":"unverified@example.com"}), json!([{"primary":true,"verified":false,"email":"unverified@example.com"}]), None),
            ("github", json!({"id":1,"email":"unverified@example.com"}), json!([{"primary":true,"verified":true,"email":"verified@example.com"}]), Some("verified@example.com")),
        ] {
            let (base, task) = provider_server(profile, emails).await;
            let mut state = AppState::default();
            let mut config = ProviderConfig::configured(provider, "id".into(), "secret".into(), "http://localhost/callback".into());
            config.token_uri = format!("{base}/token"); config.user_info_uri = format!("{base}/profile"); config.emails_uri = Some(format!("{base}/emails"));
            state.oauth.providers.insert(provider.into(), config);
            let user = fetch_user(&state, provider, "code").await;
            assert_eq!(user.ok().map(|user| user.email), expected.map(str::to_string));
            task.abort();
        }
    }
}
