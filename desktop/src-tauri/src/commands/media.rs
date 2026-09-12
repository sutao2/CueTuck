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

fn reference_url(value: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(value).map_err(|_| "参考图地址无效")?;
    if url.scheme() != "https" || url.host_str() != Some("cms-assets.youmind.com") || !url.username().is_empty() || url.password().is_some() || url.port_or_known_default() != Some(443) {
        return Err("参考图地址不在允许范围".into());
    }
    Ok(url)
}
fn reference_asset(bytes: &[u8], index: usize) -> Result<Asset, String> {
    let (ext, mime) = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") { ("png", "image/png") }
        else if bytes.starts_with(&[255,216,255]) { ("jpg", "image/jpeg") }
        else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") { ("gif", "image/gif") }
        else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") { ("webp", "image/webp") }
        else { return Err("不支持的图片内容".into()); };
    let asset = Asset { id: uuid::Uuid::new_v4().to_string(), name: format!("参考图-{}.{}",index+1,ext), mime: mime.into(), data: STANDARD.encode(bytes) };
    validate(std::slice::from_ref(&asset))?; Ok(asset)
}
#[tauri::command]
pub async fn download_reference_image(url: String, index: usize) -> Result<Asset, String> {
    if index >= 6 { return Err("参考图数量无效".into()); }
    // No session headers or redirects to other origins.
    let mut response = client()?.get(reference_url(&url)?).send().await.map_err(|_| "参考图连接失败")?;
    status(response.status())?;
    if response.content_length().is_some_and(|n| n > 5 * 1024 * 1024) { return Err("图片超过 5 MiB".into()); }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "参考图下载中断")? {
        if bytes.len() + chunk.len() > 5 * 1024 * 1024 { return Err("图片超过 5 MiB".into()); }
        bytes.extend_from_slice(&chunk);
    }
    reference_asset(&bytes,index)
}

#[cfg(test)]
mod reference_tests {
    use super::*;
    #[tokio::test]
    #[ignore = "requires an explicitly selected public reference URL"]
    async fn downloads_real_reference_to_isolated_offline_library() {
        let url=std::env::var("PROMPTARK_REFERENCE_SMOKE_URL").expect("explicit reference URL required");
        let asset=download_reference_image(url,0).await.unwrap();
        let dir=tempfile::tempdir().unwrap();crate::local_database::initialize_in_dir(dir.path()).unwrap();
        let row=crate::local_database::assets::save_prompt(dir.path(),None,"image smoke","body",None,None,std::slice::from_ref(&asset)).unwrap();
        assert_eq!(row.image_count,1);
        let local=crate::local_database::assets::list(dir.path(),&row.id).unwrap();
        assert_eq!(local[0].data,asset.data);assert!(!validate(&local).unwrap()[0].is_empty());
    }
    #[test]
    fn restricts_reference_origin_and_image_content() {
        assert!(reference_url("https://cms-assets.youmind.com/a.png").is_ok());
        for value in ["http://cms-assets.youmind.com/a", "https://cms-assets.youmind.com.evil.test/a", "https://user:pass@cms-assets.youmind.com/a", "https://cms-assets.youmind.com:8443/a", "https://127.0.0.1/a"] { assert!(reference_url(value).is_err()); }
        assert!(reference_asset(b"<svg/>",0).is_err());
        assert_eq!(reference_asset(b"\x89PNG\r\n\x1a\n",0).unwrap().mime,"image/png");
        assert!(reference_asset(&[&b"\x89PNG\r\n\x1a\n"[..], &vec![0;5*1024*1024]].concat(),0).is_err());
    }
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
