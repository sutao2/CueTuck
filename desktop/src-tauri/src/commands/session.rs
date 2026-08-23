use crate::session::{persist_session_tokens, KeyringRefreshStore, RefreshStore};
use serde::{Deserialize, Serialize};

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

#[tauri::command]
pub async fn login_local_session(email: String, password: String) -> Result<SessionView, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/session", api_base()))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("登录失败".to_string());
    }
    let pair: TokenPair = response.json().await.map_err(|error| error.to_string())?;
    persist_session_tokens(&KeyringRefreshStore, &pair.access_token, &pair.refresh_token)?;
    Ok(SessionView {
        email: pair.email,
        access_token: pair.access_token,
    })
}

#[tauri::command]
pub async fn logout_local_session(access_token: String) -> Result<(), String> {
    let client = reqwest::Client::new();
    let _ = client
        .delete(format!("{}/v1/session", api_base()))
        .bearer_auth(&access_token)
        .send()
        .await;
    KeyringRefreshStore.clear_refresh()
}
