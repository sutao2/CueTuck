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

use crate::api_config::api_base;

fn http_client(follow_redirects: bool) -> Result<reqwest::Client, String> {
    let mut builder = crate::http::client_builder()?.timeout(Duration::from_secs(10));
    if !follow_redirects {
        builder = builder.redirect(Policy::none());
    }
    builder.build().map_err(|error| error.to_string())
}

fn persist_pair(pair: TokenPair) -> Result<SessionView, String> {
    persist_session_tokens(
        &KeyringRefreshStore,
        &pair.access_token,
        &pair.refresh_token,
    )?;
    Ok(SessionView {
        email: pair.email,
        access_token: pair.access_token,
    })
}

#[tauri::command]
pub async fn list_oauth_providers() -> Result<Value, String> {
    let response = http_client(true)?
        .get(format!("{}/v1/session/oauth/providers", api_base()?))
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
        .post(format!("{}/v1/session", api_base()?))
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

#[tauri::command]
pub async fn identity_request(action: String, config: Value) -> Result<Value, String> {
    if !["options", "request", "confirm"].contains(&action.as_str()) {
        return Err("不支持的验证动作".into());
    }
    let url = format!("{}/v1/session/identity/{action}", api_base()?);
    let client = http_client(false)?;
    let response = if action == "options" {
        client.get(url)
    } else {
        client.post(url).json(&config)
    }
    .send()
    .await
    .map_err(|_| "邮箱验证服务连接失败".to_owned())?;
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            429 => "操作频繁，请一分钟后重试",
            503 => "站点邮件服务暂不可用",
            403 => "站点暂未开放新账号注册",
            400 => "验证码无效或已过期，或密码不符合 12–128 字符要求",
            _ => "邮箱验证失败，请稍后重试",
        }
        .into());
    }
    response.json().await.map_err(|_| "邮箱验证响应无效".into())
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
            api_base()?
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
        .get(format!("{}/v1/session/oauth/session/{flow_id}", api_base()?))
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
pub async fn logout_local_session(access_token: Option<String>) -> Result<(), String> {
    KeyringRefreshStore.clear_refresh()?;
    if let Some(token) = access_token {
        let _ = http_client(true)?
            .delete(format!("{}/v1/session", api_base()?))
            .bearer_auth(token)
            .send()
            .await;
    }
    Ok(())
}

#[tauri::command]
pub async fn refresh_local_session() -> Result<Option<SessionView>, String> {
    refresh_saved_session(&KeyringRefreshStore, &http_client(false)?, &api_base()?).await
}

async fn refresh_saved_session(
    store: &(impl RefreshStore + Sync),
    client: &reqwest::Client,
    base: &str,
) -> Result<Option<SessionView>, String> {
    let Some(refresh) = store.load_refresh()? else { return Ok(None) };
    let response = client.post(format!("{base}/v1/session/refresh"))
        .json(&serde_json::json!({ "refresh_token": refresh }))
        .send().await.map_err(|_| "暂时无法恢复登录，请检查网络后重试".to_string())?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        store.clear_refresh()?;
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err("暂时无法恢复登录，请稍后重试".to_string());
    }
    let pair: TokenPair = response.json().await.map_err(|_| "登录响应无效，请重试".to_string())?;
    persist_session_tokens(store, &pair.access_token, &pair.refresh_token)?;
    Ok(Some(SessionView { email: pair.email, access_token: pair.access_token }))
}

#[tauri::command]
pub async fn get_me(access_token: String) -> Result<Value, String> {
    let response = http_client(true)?
        .get(format!("{}/v1/me", api_base()?))
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("查看资料失败".to_string());
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn put_me(
    access_token: String,
    display_name: Option<String>,
    bio: Option<String>,
) -> Result<Value, String> {
    let response = http_client(true)?
        .put(format!("{}/v1/me", api_base()?))
        .bearer_auth(&access_token)
        .json(&serde_json::json!({ "display_name": display_name, "bio": bio }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("保存资料失败".to_string());
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn put_library_changes(access_token: String, items: Vec<Value>) -> Result<Value, String> {
    let response = http_client(true)?
        .put(format!("{}/v1/library/changes", api_base()?))
        .bearer_auth(&access_token)
        .json(&serde_json::json!({ "items": items }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("同步失败".to_string());
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_library_changes(
    access_token: String,
    since: Option<String>,
) -> Result<Value, String> {
    let response = http_client(true)?
        .get(format!("{}/v1/library/changes", api_base()?))
        .bearer_auth(&access_token)
        .query(&[("since", since.unwrap_or_default())])
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("同步失败".to_string());
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_billing_status(access_token: String) -> Result<Value, String> {
    let response = http_client(true)?
        .get(format!("{}/v1/billing/status", api_base()?))
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("账单暂时不可用".to_string());
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn start_billing_checkout(
    access_token: String,
    mock_outcome: Option<String>,
    request_id: Option<String>,
) -> Result<Value, String> {
    let response = http_client(true)?
        .post(format!("{}/v1/billing/checkout", api_base()?))
        .bearer_auth(&access_token)
        .json(&serde_json::json!({ "mock_outcome": mock_outcome, "request_id": request_id }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err("账单需要登录".to_string());
    }
    let status = response.status();
    let payload: Value = response
        .json()
        .await
        .map_err(|_| "支付请求失败，请重试".to_string())?;
    if !status.is_success()
        && !([403, 409].contains(&status.as_u16()) && payload["note"].is_string())
    {
        return Err("支付请求失败，请重试".into());
    }
    Ok(payload)
}

#[tauri::command]
pub async fn redeem_billing_code(
    access_token: String,
    code: String,
    mock_mode: Option<bool>,
    request_id: Option<String>,
) -> Result<Value, String> {
    let mock_mode = mock_mode.unwrap_or(false);
    let path = if mock_mode { "mock/redeem" } else { "redeem" };
    let body = if mock_mode {
        serde_json::json!({"code":code,"request_id":request_id})
    } else {
        serde_json::json!({"code":code})
    };
    let response = http_client(true)?
        .post(format!("{}/v1/billing/{path}", api_base()?))
        .bearer_auth(&access_token)
        .json(&body)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("兑换失败".to_string());
    }
    response.json().await.map_err(|error| error.to_string())
}

#[cfg(test)]
mod restore_tests {
    use super::*;
    use crate::session::MemoryRefreshStore;
    use std::io::{Read, Write};

    fn client() -> reqwest::Client {
        reqwest::Client::builder().no_proxy().timeout(Duration::from_secs(5)).build().unwrap()
    }

    fn server(status: u16, body: &'static str) -> (String, std::thread::JoinHandle<()>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            let mut bytes = [0; 4096];
            let size = stream.read(&mut bytes).unwrap();
            assert!(String::from_utf8_lossy(&bytes[..size]).starts_with("POST /v1/session/refresh "));
            write!(stream, "HTTP/1.1 {status} Response\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        });
        (url, handle)
    }

    #[tokio::test]
    async fn restore_rotates_credentials_and_returns_only_access() {
        let store = MemoryRefreshStore::default(); store.save_refresh("ref.old").unwrap();
        let (url, handle) = server(200, r#"{"email":"test@example.test","access_token":"acc.new","refresh_token":"ref.new"}"#);
        let session = refresh_saved_session(&store, &client(), &url).await.unwrap().unwrap();
        handle.join().unwrap();
        assert_eq!(store.load_refresh().unwrap().as_deref(), Some("ref.new"));
        assert_eq!(session.email, "test@example.test");
        assert_eq!(session.access_token, "acc.new");
        assert!(!serde_json::to_string(&session).unwrap().contains("ref.new"));
    }

    #[tokio::test]
    async fn restore_preserves_credentials_on_transient_or_malformed_responses() {
        for (status, body) in [(503, "{}"), (429, "{}"), (200, "{}"), (200, r#"{"email":"x","access_token":"bad","refresh_token":"ref.new"}"#)] {
            let store = MemoryRefreshStore::default(); store.save_refresh("ref.old").unwrap();
            let (url, handle) = server(status, body);
            assert!(refresh_saved_session(&store, &client(), &url).await.is_err());
            handle.join().unwrap();
            assert_eq!(store.load_refresh().unwrap().as_deref(), Some("ref.old"));
        }
    }

    #[tokio::test]
    async fn restore_clears_only_rejected_credentials_and_skips_empty_store() {
        let store = MemoryRefreshStore::default();
        assert!(refresh_saved_session(&store, &client(), "invalid-url").await.unwrap().is_none());
        store.save_refresh("ref.old").unwrap();
        assert!(refresh_saved_session(&store, &client(), "invalid-url").await.is_err());
        assert_eq!(store.load_refresh().unwrap().as_deref(), Some("ref.old"));
        let (url, handle) = server(401, "{}");
        assert!(refresh_saved_session(&store, &client(), &url).await.unwrap().is_none());
        handle.join().unwrap(); assert_eq!(store.load_refresh().unwrap(), None);
    }
}
