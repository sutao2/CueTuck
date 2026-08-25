use crate::session::{persist_session_tokens, KeyringRefreshStore, RefreshStore};
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

#[derive(Deserialize)]
struct TokenPair {
    email: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Serialize)]
pub struct SessionView {
    pub email: String,
    pub access_token: String,
}

fn api_base() -> String {
    std::env::var("PROMPTARK_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:8787".into())
}

fn http_client(follow_redirects: bool) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(10));
    if !follow_redirects {
        builder = builder.redirect(Policy::none());
    }
    builder.build().map_err(|error| error.to_string())
}

fn persist_pair(pair: TokenPair) -> Result<SessionView, String> {
    persist_session_tokens(&KeyringRefreshStore, &pair.access_token, &pair.refresh_token)?;
    Ok(SessionView {
        email: pair.email,
        access_token: pair.access_token,
    })
}

#[tauri::command]
pub async fn list_oauth_providers() -> Result<Value, String> {
    let response = http_client(true)?
        .get(format!("{}/v1/session/oauth/providers", api_base()))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("无法读取登录方式".to_string());
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn login_local_session(email: String, password: String) -> Result<SessionView, String> {
    let response = http_client(true)?
        .post(format!("{}/v1/session", api_base()))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("登录失败".to_string());
    }
    let pair: TokenPair = response.json().await.map_err(|error| error.to_string())?;
    persist_pair(pair)
}

fn cancelled_flows() -> &'static Mutex<HashSet<String>> {
    static CELL: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashSet::new()))
}

fn is_cancelled(flow_id: &str) -> Result<bool, String> {
    Ok(cancelled_flows()
        .lock()
        .map_err(|error| error.to_string())?
        .contains(flow_id))
}

#[tauri::command]
pub async fn start_oauth_session(provider: String) -> Result<String, String> {
    let provider = provider.to_lowercase();
    if provider != "google" && provider != "github" {
        return Err("不支持的登录方式".to_string());
    }
    let flow_id = uuid::Uuid::new_v4().to_string();
    let response = http_client(false)?
        .get(format!(
            "{}/v1/session/oauth/{provider}?response_mode=browser&flow_id={flow_id}",
            api_base()
        ))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "无法打开授权页".to_string())?
        .to_string();
    open::that_detached(&location).map_err(|error| error.to_string())?;
    Ok(flow_id)
}

async fn load_ready_pair(flow_id: &str) -> Result<Option<TokenPair>, String> {
    if flow_id.len() < 16 {
        return Err("登录未完成".to_string());
    }
    let poll = match http_client(true)?
        .get(format!(
            "{}/v1/session/oauth/session/{flow_id}",
            api_base()
        ))
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return Ok(None),
    };
    if !poll.status().is_success() {
        return Ok(None);
    }
    let payload: Value = poll.json().await.map_err(|error| error.to_string())?;
    if payload.get("status").and_then(Value::as_str) != Some("ready") {
        return Ok(None);
    }
    let session = payload
        .get("session")
        .cloned()
        .ok_or_else(|| "登录未完成".to_string())?;
    Ok(Some(
        serde_json::from_value(session).map_err(|error| error.to_string())?,
    ))
}

#[tauri::command]
pub async fn poll_oauth_session(flow_id: String) -> Result<bool, String> {
    if is_cancelled(&flow_id)? {
        return Ok(false);
    }
    Ok(load_ready_pair(&flow_id).await?.is_some())
}

#[tauri::command]
pub async fn commit_oauth_session(flow_id: String) -> Result<SessionView, String> {
    if is_cancelled(&flow_id)? {
        return Err("已取消".to_string());
    }
    let pair = load_ready_pair(&flow_id)
        .await?
        .ok_or_else(|| "登录未完成".to_string())?;
    let mut cancelled = cancelled_flows()
        .lock()
        .map_err(|error| error.to_string())?;
    if cancelled.contains(&flow_id) {
        return Err("已取消".to_string());
    }
    let view = persist_pair(pair)?;
    cancelled.insert(flow_id);
    Ok(view)
}

#[tauri::command]
pub async fn cancel_oauth_session(flow_id: String) -> Result<(), String> {
    cancelled_flows()
        .lock()
        .map_err(|error| error.to_string())?
        .insert(flow_id);
    Ok(())
}

#[tauri::command]
pub async fn logout_local_session(access_token: String) -> Result<(), String> {
    let _ = http_client(true)?
        .delete(format!("{}/v1/session", api_base()))
        .bearer_auth(&access_token)
        .send()
        .await;
    KeyringRefreshStore.clear_refresh()
}

#[tauri::command]
pub async fn refresh_local_session() -> Result<SessionView, String> {
    let refresh = KeyringRefreshStore
        .load_refresh()?
        .ok_or_else(|| "没有 refresh".to_string())?;
    let response = http_client(true)?
        .post(format!("{}/v1/session/refresh", api_base()))
        .json(&serde_json::json!({ "refresh_token": refresh }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("刷新失败".to_string());
    }
    let pair: TokenPair = response.json().await.map_err(|error| error.to_string())?;
    persist_pair(pair)
}
