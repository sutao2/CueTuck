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
    fetch_reference_image(client()?, reference_url(&url)?, index).await
}

struct CachedImage { url: String, bytes: Vec<u8>, saved: std::time::Instant }
#[derive(Default)]
struct ReferenceCache { entries: std::collections::VecDeque<CachedImage> }
impl ReferenceCache {
    fn get(&mut self, url: &str) -> Option<Vec<u8>> {
        self.entries.retain(|entry| entry.saved.elapsed() < Duration::from_secs(300));
        self.entries.iter().find(|entry|entry.url==url).map(|entry|entry.bytes.clone())
    }
    fn insert(&mut self, url: String, bytes: Vec<u8>) {
        self.entries.retain(|entry|entry.url!=url && entry.saved.elapsed()<Duration::from_secs(300));
        let mut size: usize = self.entries.iter().map(|entry|entry.bytes.len()).sum();
        while size + bytes.len() > 32*1024*1024 {
            if let Some(entry) = self.entries.pop_front() { size-=entry.bytes.len(); } else { return; }
        }
        self.entries.push_back(CachedImage { url, bytes, saved:std::time::Instant::now() });
    }
}
fn reference_cache() -> &'static std::sync::Mutex<ReferenceCache> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<ReferenceCache>> = std::sync::OnceLock::new();
    CACHE.get_or_init(||std::sync::Mutex::new(ReferenceCache::default()))
}
async fn fetch_reference_image(client: reqwest::Client, url: reqwest::Url, index: usize) -> Result<Asset, String> {
    let key=url.to_string();
    if let Some(bytes)=reference_cache().lock().map_err(|_|"图片缓存不可用")?.get(&key) { return reference_asset(&bytes,index); }
    // No session headers or redirects to other origins.
    let mut response = client.get(url).send().await.map_err(|_| "参考图连接失败")?;
    status(response.status())?;
    if response.content_length().is_some_and(|n| n > 5 * 1024 * 1024) { return Err("图片超过 5 MiB".into()); }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "参考图下载中断")? {
        if bytes.len() + chunk.len() > 5 * 1024 * 1024 { return Err("图片超过 5 MiB".into()); }
        bytes.extend_from_slice(&chunk);
    }
    let asset=reference_asset(&bytes,index)?;
    reference_cache().lock().map_err(|_|"图片缓存不可用")?.insert(key,bytes);
    Ok(asset)
}

#[derive(Clone, Serialize)]
pub struct ReferenceDownloadProgress {
    completed: usize,
    total: usize,
}

#[tauri::command]
pub async fn download_reference_images(
    urls: Vec<String>,
    on_progress: tauri::ipc::Channel<ReferenceDownloadProgress>,
) -> Result<Vec<Asset>, String> {
    if urls.len() > 6 { return Err("参考图数量无效".into()); }
    let urls = urls.iter().map(|url| reference_url(url)).collect::<Result<Vec<_>, _>>()?;
    fetch_reference_batch(client()?, urls, |completed, total| {
        let _ = on_progress.send(ReferenceDownloadProgress { completed, total });
    }).await
}

async fn fetch_reference_batch(
    client: reqwest::Client,
    urls: Vec<reqwest::Url>,
    progress: impl Fn(usize, usize),
) -> Result<Vec<Asset>, String> {
    let total = urls.len();
    let mut pending = urls.into_iter().enumerate();
    let mut tasks = tokio::task::JoinSet::new();
    let mut assets = Vec::with_capacity(total);
    progress(0, total);
    for (index, url) in pending.by_ref().take(3) {
        let client = client.clone();
        tasks.spawn(async move { (index, fetch_reference_image(client, url, index).await) });
    }
    while let Some(result) = tasks.join_next().await {
        let (index, asset) = result.map_err(|_| "参考图下载任务中断")?;
        let asset = asset.map_err(|error| format!("参考图 {} 下载失败：{}，请重试", index + 1, error))?;
        assets.push((index, asset));
        progress(assets.len(), total);
        if let Some((index, url)) = pending.next() {
            let client = client.clone();
            tasks.spawn(async move { (index, fetch_reference_image(client, url, index).await) });
        }
    }
    // Dropping JoinSet on an error aborts in-flight requests before returning.
    assets.sort_by_key(|(index, _)| *index);
    let assets: Vec<_> = assets.into_iter().map(|(_, asset)| asset).collect();
    validate(&assets)?;
    Ok(assets)
}

#[cfg(test)]
mod reference_tests {
    use super::*;
    #[test]
    fn cache_expires_evicts_and_returns_fresh_ids() {
        let mut cache=ReferenceCache::default();
        cache.insert("first".into(), b"\x89PNG\r\n\x1a\n".to_vec());
        let bytes=cache.get("first").unwrap();
        assert_ne!(reference_asset(&bytes,0).unwrap().id,reference_asset(&bytes,1).unwrap().id);
        cache.entries[0].saved=std::time::Instant::now()-Duration::from_secs(301);
        assert!(cache.get("first").is_none());
        for i in 0..10 { cache.insert(i.to_string(),vec![0;5*1024*1024]); }
        assert!(cache.get("0").is_none()); assert!(cache.get("9").is_some());
        assert!(cache.entries.iter().map(|e|e.bytes.len()).sum::<usize>()<=32*1024*1024);
    }
    #[tokio::test]
    async fn cached_image_requires_no_connection() {
        let url=reqwest::Url::parse("http://127.0.0.1:1/cached-test").unwrap();
        reference_cache().lock().unwrap().insert(url.to_string(),b"\x89PNG\r\n\x1a\n".to_vec());
        let asset=fetch_reference_image(reqwest::Client::new(),url,2).await.unwrap();
        assert_eq!(asset.name,"参考图-3.png");
    }
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
    async fn batch_fixture(broken: bool) -> (Result<Vec<Asset>, String>, usize, usize, Vec<usize>) {
        use std::{io::{BufRead, BufReader, Write}, net::TcpListener, sync::{Arc, Barrier, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}}};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap(); listener.set_nonblocking(true).unwrap();
        let done = Arc::new(AtomicBool::new(false));
        let connections = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(3));
        let (stop, accepted, reads) = (done.clone(), connections.clone(), requests.clone());
        let server = std::thread::spawn(move || {
            let mut workers = Vec::new();
            while !stop.load(Ordering::SeqCst) {
                let Ok((stream, _)) = listener.accept() else { std::thread::sleep(Duration::from_millis(1)); continue; };
                accepted.fetch_add(1, Ordering::SeqCst);
                let (reads, barrier) = (reads.clone(), barrier.clone());
                workers.push(std::thread::spawn(move || {
                    stream.set_nonblocking(false).unwrap();
                    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                    let mut reader = BufReader::new(stream);
                    loop {
                        let mut line = String::new();
                        if reader.read_line(&mut line).unwrap_or(0) == 0 { break; }
                        let index: usize = line.split_whitespace().nth(1).unwrap().trim_start_matches('/').parse().unwrap();
                        loop { let mut header = String::new(); if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" { break; } }
                        let number = reads.fetch_add(1, Ordering::SeqCst);
                        if number < 3 { barrier.wait(); }
                        std::thread::sleep(Duration::from_millis(if broken && index == 0 { 1 } else { 30 + (6 - index) as u64 * 10 }));
                        let body = if broken && index == 0 { b"invalid".to_vec() } else { [b"\x89PNG\r\n\x1a\n".as_slice(), &[index as u8]].concat() };
                        let header = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: image/png\r\n\r\n", body.len());
                        if reader.get_mut().write_all(header.as_bytes()).and_then(|_| reader.get_mut().write_all(&body)).is_err() { break; }
                    }
                }));
            }
            for worker in workers { worker.join().unwrap(); }
        });
        let progress = Mutex::new(Vec::new());
        let urls = (0..6).map(|i| reqwest::Url::parse(&format!("http://{address}/{i}")).unwrap()).collect();
        let result = fetch_reference_batch(reqwest::Client::builder().no_proxy().timeout(Duration::from_secs(3)).build().unwrap(), urls,
            |completed, _| progress.lock().unwrap().push(completed)).await;
        done.store(true, Ordering::SeqCst); server.join().unwrap();
        (result, connections.load(Ordering::SeqCst), requests.load(Ordering::SeqCst), progress.into_inner().unwrap())
    }

    #[tokio::test]
    async fn batch_reuses_three_connections_and_preserves_image_order() {
        let (result, connections, requests, progress) = batch_fixture(false).await;
        let assets = result.unwrap();
        assert_eq!(connections, 3); assert_eq!(requests, 6);
        assert_eq!(progress, (0..=6).collect::<Vec<_>>());
        for (i, asset) in assets.iter().enumerate() {
            assert_eq!(asset.name, format!("参考图-{}.png", i + 1));
            assert_eq!(STANDARD.decode(&asset.data).unwrap()[8], i as u8);
        }
    }

    #[tokio::test]
    async fn failed_batch_cancels_requests_without_starting_queued_images() {
        let (result, connections, requests, progress) = batch_fixture(true).await;
        assert!(result.unwrap_err().contains("参考图 1 下载失败"));
        assert_eq!(connections, 3); assert_eq!(requests, 3); assert_eq!(progress, vec![0]);
    }

    #[tokio::test]
    #[ignore = "requires explicitly selected public reference URLs"]
    async fn downloads_real_reference_batch_to_isolated_library() {
        let urls: Vec<String> = serde_json::from_str(&std::env::var("PROMPTARK_REFERENCE_SMOKE_URLS").expect("explicit URLs required")).unwrap();
        let started = std::time::Instant::now();
        let parsed = urls.iter().map(|url| reference_url(url).unwrap()).collect();
        let assets = fetch_reference_batch(client().unwrap(), parsed, |_, _| {}).await.unwrap();
        let dir = tempfile::tempdir().unwrap(); crate::local_database::initialize_in_dir(dir.path()).unwrap();
        let row = crate::local_database::assets::save_prompt(dir.path(), None, "batch smoke", "body", None, None, &assets).unwrap();
        let local = crate::local_database::assets::list(dir.path(), &row.id).unwrap();
        assert_eq!(local.len(), urls.len());
        for (saved, downloaded) in local.iter().zip(&assets) { assert_eq!(saved.data, downloaded.data); }
        println!("{} images downloaded and verified in {:?}", local.len(), started.elapsed());
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
