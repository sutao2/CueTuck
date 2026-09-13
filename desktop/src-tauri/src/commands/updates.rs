use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::{json, Value};
use semver::Version;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use std::sync::Mutex;
use tauri_plugin_updater::Update;

#[derive(Default)]
pub struct UpdateState(pub Mutex<Option<(Update, Vec<u8>)>>, pub tokio::sync::Mutex<()>);
use tauri_plugin_updater::UpdaterExt;
use url::Url;

const RELEASES_URL: &str = "https://api.github.com/repos/sutao2/CueTuck/releases";

fn http_client() -> Result<reqwest::Client, String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("promptark-desktop"),
    );
    crate::http::apply_runtime_proxy(
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .default_headers(headers),
    )?
    .build()
    .map_err(|_| "检查失败".to_string())
}

fn want_preview(channel: Option<&str>) -> bool {
    channel == Some("preview")
}

fn parse_tag(tag: &str) -> Option<Version> {
    Version::parse(tag.strip_prefix('v').or_else(|| tag.strip_prefix('V')).unwrap_or(tag)).ok()
}

fn pick_release(releases: &[Value], preview: bool) -> Option<&Value> {
    releases.iter().filter_map(|row| {
        let version = parse_tag(row.get("tag_name")?.as_str()?)?;
        if row.get("draft").and_then(Value::as_bool).unwrap_or(false)
            || row.get("prerelease").and_then(Value::as_bool).unwrap_or(false) != preview
            || !version.pre.is_empty() != preview { return None; }
        Some((row, version))
    }).max_by(|(_, a), (_, b)| a.cmp_precedence(b)).map(|(row, _)| row)
}

fn newer_than_current(latest: &Value) -> bool {
    let Some(remote) = latest.get("tag_name").and_then(Value::as_str).and_then(parse_tag) else { return false; };
    let Ok(current) = Version::parse(env!("CARGO_PKG_VERSION")) else { return false; };
    remote.cmp_precedence(&current).is_gt()
}

fn release_payload(latest: &Value) -> Value {
    let remote = latest
        .get("tag_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim_start_matches(['v', 'V']);
    let available = newer_than_current(latest);
    json!({
        "available": available,
        "notes": latest.get("body").and_then(Value::as_str).unwrap_or(""),
        "version": remote,
    })
}

async fn list_releases() -> Result<Vec<Value>, String> {
    let response = http_client()?
        .get(RELEASES_URL)
        .query(&[("per_page", "100")])
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
pub async fn queue_update_install(app: AppHandle, channel: Option<String>, version: String, state: State<'_, UpdateState>) -> Result<Value, String> {
    let _guard = state.1.try_lock().map_err(|_| "更新正在进行".to_string())?;
    let releases = list_releases().await?;
    let latest = pick_release(&releases, want_preview(channel.as_deref())).ok_or("没有可用更新")?;
    if !newer_than_current(latest) || release_payload(latest)["version"] != version { return Err("版本已变化，请重新检查更新".into()); }
    let tag = latest["tag_name"].as_str().ok_or("版本无效")?;
    let endpoint = Url::parse(&format!("https://github.com/sutao2/CueTuck/releases/download/{tag}/latest.json")).map_err(|_| "更新地址无效")?;
    let mut builder = app.updater_builder().timeout(Duration::from_secs(180)).endpoints(vec![endpoint]).map_err(|_| "更新地址无效")?;
    if let Some(proxy) = crate::http::runtime_proxy_url()? { builder = builder.proxy(proxy); }
    let updater = builder.build().map_err(|_| "更新配置无效")?;
    let update = updater.check().await.map_err(|_| "读取更新包失败，请重试")?.ok_or("当前平台没有更新包")?;
    if update.version != version || update.download_url.scheme() != "https" || update.download_url.host_str() != Some("github.com") || !update.download_url.path().starts_with(&format!("/sutao2/CueTuck/releases/download/{tag}/")) {
        return Err("更新包与所选版本不一致".into());
    }
    *state.0.lock().map_err(|_| "更新状态不可用")? = None;
    let mut downloaded = 0u64;
    let mut last = std::time::Instant::now();
    let bytes = update.download(|chunk, total| {
        downloaded += chunk as u64;
        if last.elapsed() >= Duration::from_millis(100) || total == Some(downloaded) {
            let _ = app.emit("update-progress", json!({"phase":"downloading", "downloaded":downloaded,"total":total}));
            last = std::time::Instant::now();
        }
    }, || { let _ = app.emit("update-progress", json!({"phase":"verifying"})); }).await.map_err(|_| "下载或签名验证失败，请重试")?;
    let size = bytes.len();
    *state.0.lock().map_err(|_| "更新状态不可用")? = Some((update, bytes));
    Ok(json!({"ready":true,"version":version,"size":size}))
}

#[tauri::command]
pub async fn install_downloaded_update(app: AppHandle, version: String, state: State<'_, UpdateState>) -> Result<(), String> {
    let _guard = state.1.try_lock().map_err(|_| "更新正在进行".to_string())?;
    let staged = state.0.lock().map_err(|_| "更新状态不可用")?.clone().ok_or("请先下载更新")?;
    if staged.0.version != version { return Err("已下载版本不匹配，请重新下载".into()); }
    tauri::async_runtime::spawn_blocking(move || staged.0.install(staged.1)).await.map_err(|_| "安装任务失败")?.map_err(|_| "安装失败，请关闭其他实例后重试")?;
    app.restart();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_older_equivalent_and_invalid_versions_before_install() {
        for tag in ["v0.0.9".to_string(), format!("v{}+rebuild", env!("CARGO_PKG_VERSION")), "nightly".into(), "../latest".into()] {
            let release = json!({ "tag_name": tag });
            assert!(!newer_than_current(&release));
            assert_eq!(release_payload(&release)["available"], false);
        }
    }

    #[test]
    fn selects_highest_valid_non_draft_version_in_channel() {
        let releases = vec![
            json!({"tag_name": "v99.0.0", "draft": true}),
            json!({"tag_name": "nightly"}),
            json!({"tag_name": "v0.2.0"}),
            json!({"tag_name": "V0.10.0", "body": "newest"}),
            json!({"tag_name": "v1.0.0-beta.2", "prerelease": true}),
            json!({"tag_name": "v1.0.0-beta.10", "prerelease": true}),
            json!({"tag_name": "v2.0.0-beta.1", "prerelease": false}),
        ];
        let stable = release_payload(pick_release(&releases, false).unwrap());
        assert_eq!(stable, json!({"available": true, "version": "0.10.0", "notes": "newest"}));
        assert_eq!(pick_release(&releases, true).unwrap()["tag_name"], "v1.0.0-beta.10");
        assert!(pick_release(&[json!({"tag_name": "v2.0.0-beta.1"})], false).is_none());
        assert!(pick_release(&[], true).is_none());
    }
}
