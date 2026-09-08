use base64::{engine::general_purpose::STANDARD, Engine};
use crate::local_database::assets::{Asset, validate};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;

#[derive(Deserialize, Serialize)]
pub struct Reference {
    pub id: String,
    pub media_id: String,
    pub name: String,
    pub mime: String,
    pub size: usize,
    pub sha256: String,
}

fn client() -> Result<reqwest::Client, String> {
    crate::http::client_builder()?.redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(45)).build().map_err(|_| "附件服务连接失败".into())
}
use crate::api_config::api_base as base;
fn status(code: reqwest::StatusCode) -> Result<(), String> {
    if code.is_success() { return Ok(()); }
    Err(match code.as_u16() {
        401 => "登录已失效，请重新登录后同步",
        409 => "附件正在上传或账号存储配额已满，请稍后重试",
        404 => "附件不存在或不属于当前账号",
        413 => "附件超过大小限制",
        _ => "附件传输失败，请重试",
    }.into())
}

#[tauri::command]
pub fn hash_private_asset(asset: Asset) -> Result<String, String> {
    let bytes = validate(std::slice::from_ref(&asset))?.remove(0);
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[tauri::command]
pub async fn upload_private_asset(access_token: String, asset: Asset) -> Result<Reference, String> {
    let bytes = validate(std::slice::from_ref(&asset))?.remove(0);
    let size = bytes.len();
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let part = reqwest::multipart::Part::bytes(bytes).file_name(asset.name.clone())
        .mime_str(&asset.mime).map_err(|_| "附件类型无效")?;
    let response = client()?.post(format!("{}/v1/media/upload", base()?)).bearer_auth(access_token)
        .multipart(reqwest::multipart::Form::new().part("file", part)).send().await.map_err(|_| "附件上传连接失败，请重试")?;
    status(response.status())?;
    let value: serde_json::Value = response.json().await.map_err(|_| "附件上传响应无效")?;
    if value["sha256"] != sha256 || value["size"] != size || value["name"] != asset.name || value["mime"] != asset.mime {
        return Err("附件上传校验失败".into());
    }
    let media_id = value["id"].as_str().filter(|id| valid_media_id(id)).ok_or("附件标识无效")?.to_owned();
    Ok(Reference { id: asset.id, media_id, name: asset.name, mime: asset.mime, size, sha256 })
}

fn valid_media_id(id: &str) -> bool {
    id.strip_prefix("media.").is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
}

#[tauri::command]
pub async fn download_private_asset(access_token: String, reference: Reference) -> Result<Asset, String> {
    let url = format!("{}/v1/media/{}/content", base()?, reference.media_id);
    download(url, Some(access_token), reference).await
}

#[tauri::command]
pub async fn download_published_asset(item_id: String, access_token: Option<String>, reference: Reference) -> Result<Asset, String> {
    let mut url = reqwest::Url::parse(&base()?).map_err(|_| "附件服务地址无效")?;
    url.path_segments_mut().map_err(|_| "附件服务地址无效")?.extend(["v1", "square", "items", &item_id, "assets", &reference.id]);
    download(url.to_string(), access_token, reference).await
}

async fn download(url: String, access_token: Option<String>, reference: Reference) -> Result<Asset, String> {
    if !valid_media_id(&reference.media_id) || reference.size > 5 * 1024 * 1024 { return Err("附件引用无效".into()); }
    let mut request = client()?.get(url);
    if let Some(token) = access_token { request = request.bearer_auth(token); }
    let mut response = request.send().await.map_err(|_| "附件下载连接失败，请重试")?;
    status(response.status())?;
    if response.content_length().is_some_and(|n| n > reference.size as u64) { return Err("附件长度校验失败".into()); }
    let mut bytes = Vec::with_capacity(reference.size);
    while let Some(chunk) = response.chunk().await.map_err(|_| "附件下载中断，请重试")? {
        if bytes.len() + chunk.len() > reference.size { return Err("附件长度校验失败".into()); }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.len() != reference.size || format!("{:x}", Sha256::digest(&bytes)) != reference.sha256 { return Err("附件内容校验失败".into()); }
    let asset = Asset { id: reference.id, name: reference.name, mime: reference.mime, data: STANDARD.encode(bytes) };
    validate(std::slice::from_ref(&asset))?;
    Ok(asset)
}
