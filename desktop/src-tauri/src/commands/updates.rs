use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::{json, Value};
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
use url::Url;

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
        .map_err(|_| "检查失败".to_string())
}

fn want_preview(channel: Option<&str>) -> bool {
    channel == Some("preview")
}

fn tag_is_safe(tag: &str) -> bool {
    let body = tag.strip_prefix('v').unwrap_or(tag);
    !body.is_empty()
        && body
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn pick_release(releases: &[Value], preview: bool) -> Option<&Value> {
    releases.iter().find(|row| {
        row.get("prerelease")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            == preview
    })
}

fn release_payload(latest: &Value) -> Value {
    let remote = latest
        .get("tag_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim_start_matches('v');
    let current = env!("CARGO_PKG_VERSION");
    let available = !remote.is_empty() && remote != current;
    json!({
        "available": available,
        "notes": latest.get("body").and_then(Value::as_str).unwrap_or(""),
        "version": remote,
    })
}

async fn list_releases() -> Result<Vec<Value>, String> {
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
    response.json().await.map_err(|_| "检查失败".to_string())
}

#[tauri::command]
pub async fn check_for_updates(channel: Option<String>) -> Result<Value, String> {
    let releases = list_releases().await?;
    let Some(latest) = pick_release(&releases, want_preview(channel.as_deref())) else {
        return Ok(json!({ "available": false, "notes": "" }));
    };
    Ok(release_payload(latest))
}

#[tauri::command]
pub async fn queue_update_install(
    app: AppHandle,
    channel: Option<String>,
) -> Result<Value, String> {
    let releases = list_releases().await.map_err(|_| "安装失败".to_string())?;
    let Some(latest) = pick_release(&releases, want_preview(channel.as_deref())) else {
        return Ok(json!({ "queued": false, "via": "updater" }));
    };
    let tag = latest
        .get("tag_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if tag.is_empty() || !tag_is_safe(tag) {
        return Ok(json!({ "queued": false, "via": "updater" }));
    }
    let endpoint = Url::parse(&format!(
        "https://github.com/sutao2/PromptArk/releases/download/{tag}/latest.json"
    ))
    .map_err(|_| "安装失败".to_string())?;
    let updater = app
        .updater_builder()
        .timeout(Duration::from_secs(30))
        .endpoints(vec![endpoint])
        .map_err(|_| "安装失败".to_string())?
        .build()
        .map_err(|_| "安装失败".to_string())?;
    let Some(update) = updater.check().await.map_err(|_| "安装失败".to_string())? else {
        return Ok(json!({ "queued": false, "via": "updater" }));
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|_| "安装失败".to_string())?;
    Ok(json!({ "queued": true, "via": "updater" }))
}
