use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::{json, Value};
use std::time::Duration;

const RELEASES_URL: &str = "https://api.github.com/repos/sutao2/PromptArk/releases";

fn http_client() -> Result<reqwest::Client, String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("promptark-desktop"),
    );
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .default_headers(headers)
        .build()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn check_for_updates() -> Result<Value, String> {
    let response = http_client()?
        .get(RELEASES_URL)
        .query(&[("per_page", "5")])
        .header("accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| "检查失败".to_string())?;
    if !response.status().is_success() {
        return Err("检查失败".to_string());
    }
    let releases: Vec<Value> = response.json().await.map_err(|_| "检查失败".to_string())?;
    let Some(latest) = releases.first() else {
        return Ok(json!({ "available": false, "notes": "" }));
    };
    let remote = latest
        .get("tag_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim_start_matches('v');
    let current = env!("CARGO_PKG_VERSION");
    let available = !remote.is_empty() && remote != current;
    Ok(json!({
        "available": available,
        "notes": latest.get("body").and_then(Value::as_str).unwrap_or(""),
        "version": remote,
    }))
}
