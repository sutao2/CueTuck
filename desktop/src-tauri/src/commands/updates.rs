use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::{json, Value};
use semver::Version;
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
    if !newer_than_current(latest) {
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
