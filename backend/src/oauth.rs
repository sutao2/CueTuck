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
    fn from_env() -> HashMap<String, ProviderConfig> {
        let mut map = HashMap::new();
        if let Some(google) = load_provider(
            "google",
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            "https://openidconnect.googleapis.com/v1/userinfo",
            None,
            "http://localhost:8787/v1/session/oauth/callback",
            "openid email profile",
        ) {
            map.insert("google".into(), google);
        }
        if let Some(github) = load_provider(
            "github",
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
            "https://api.github.com/user",
            Some("https://api.github.com/user/emails"),
            "http://localhost:8787/v1/session/oauth/callback",
            "read:user user:email",
        ) {
            map.insert("github".into(), github);
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

fn load_provider(
    name: &str,
    authorization_uri: &str,
    token_uri: &str,
    user_info_uri: &str,
    emails_uri: Option<&str>,
    default_redirect: &str,
    scope: &str,
) -> Option<ProviderConfig> {
    let prefix = name.to_uppercase();
    let client_id = env_pair(
        &format!("PROMPTARK_{prefix}_CLIENT_ID"),
        &format!("PL_{prefix}_CLIENT_ID"),
    )?;
    let client_secret = env_pair(
        &format!("PROMPTARK_{prefix}_CLIENT_SECRET"),
        &format!("PL_{prefix}_CLIENT_SECRET"),
    )?;
    Some(ProviderConfig {
        client_id,
        client_secret,
        authorization_uri: authorization_uri.into(),
        token_uri: token_uri.into(),
        user_info_uri: user_info_uri.into(),
        emails_uri: emails_uri.map(str::to_string),
        redirect_uri: env_pair(
            &format!("PROMPTARK_{prefix}_REDIRECT_URI"),
            &format!("PL_{prefix}_REDIRECT_URI"),
        )
        .unwrap_or_else(|| default_redirect.into()),
        scope: scope.into(),
    })
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

pub fn enabled_providers(settings: &OAuthSettings) -> Vec<String> {
    let mut names: Vec<_> = settings.providers.keys().cloned().collect();
    names.sort();
    names
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

pub async fn list_providers(State(state): State<AppState>) -> Json<Value> {
    Json(json!({ "items": enabled_providers(&state.oauth) }))
}

pub async fn start(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthStartQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let provider = provider.to_lowercase();
    let config = state
        .oauth
        .providers
        .get(&provider)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
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
    if let Some(user) = state.oauth.mock_users.get(code).cloned() {
        return Ok(user);
    }
    let config = state
        .oauth
        .providers
        .get(provider)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    let client = reqwest::Client::new();
    let token_response: Value = client
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
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .json()
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let access = token_response
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let profile: Value = client
        .get(&config.user_info_uri)
        .bearer_auth(access)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .json()
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
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
    if email.is_empty() && provider == "github" {
        if let Some(uri) = &config.emails_uri {
            let emails: Value = client
                .get(uri)
                .bearer_auth(access)
                .header(reqwest::header::ACCEPT, "application/json")
                .send()
                .await
                .map_err(|_| StatusCode::UNAUTHORIZED)?
                .json()
                .await
                .map_err(|_| StatusCode::UNAUTHORIZED)?;
            if let Some(items) = emails.as_array() {
                email = items
                    .iter()
                    .find(|item| item.get("primary") == Some(&Value::Bool(true)))
                    .and_then(|item| item.get("email"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
            }
        }
    }
    if email.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(OAuthUser {
        provider: provider.into(),
        provider_uid: uid,
        email,
    })
}
