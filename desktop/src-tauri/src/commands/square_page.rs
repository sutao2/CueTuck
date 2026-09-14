use std::{collections::HashMap, sync::{Mutex, OnceLock}, time::Duration};
use tokio::sync::oneshot;

fn pending() -> &'static Mutex<HashMap<String, oneshot::Sender<()>>> {
    static REQUESTS: OnceLock<Mutex<HashMap<String, oneshot::Sender<()>>>> = OnceLock::new();
    REQUESTS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[tauri::command]
pub fn cancel_square_page(request_id: String) {
    if let Some(sender) = pending().lock().unwrap().remove(&request_id) { let _ = sender.send(()); }
}

#[tauri::command]
pub async fn list_square_page(sort: String, query: String, model: String, content_language: Option<String>, category_id: Option<String>, offset: u32, request_id: String, access_token: Option<String>) -> Result<serde_json::Value, String> {
    if request_id.len() > 64 || offset > 100_000 { return Err("分页参数无效".into()); }
    let (sender, canceled) = oneshot::channel();
    {
        let mut requests = pending().lock().unwrap();
        if requests.len() >= 16 || requests.contains_key(&request_id) { return Err("请求过于频繁".into()); }
        requests.insert(request_id.clone(), sender);
    }
    let result = tokio::select! {
        _ = canceled => Err("请求已取消".into()),
        result = async {
            let mut request = crate::http::client()?.get(format!("{}/v1/square/browse", crate::api_config::api_base()?))
                .timeout(Duration::from_secs(10))
                .query(&[("sort", sort), ("q", query), ("model", model), ("offset", offset.to_string()), ("limit", "48".into())]);
            if let Some(language) = content_language { request = request.query(&[("content_language", language)]); }
            if let Some(category) = category_id { request = request.query(&[("category_id", category)]); }
            if let Some(token) = access_token { request = request.bearer_auth(token); }
            let mut response = request.send().await.map_err(|_| "广场连接失败".to_string())?;
            if !response.status().is_success() { return Err("广场暂时不可用".into()); }
            let mut body = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|_| "广场响应读取失败".to_string())? {
                if body.len() + chunk.len() > 1024 * 1024 { return Err("广场分页响应过大".into()); }
                body.extend_from_slice(&chunk);
            }
            serde_json::from_slice(&body).map_err(|_| "广场分页响应无效".into())
        } => result,
    };
    pending().lock().unwrap().remove(&request_id);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn cancellation_is_scoped_and_removes_pending_request() {
        let (tx, rx) = oneshot::channel();
        pending().lock().unwrap().insert("test-page".into(), tx);
        cancel_square_page("other-page".into());
        assert!(pending().lock().unwrap().contains_key("test-page"));
        cancel_square_page("test-page".into());
        assert!(rx.await.is_ok());
        assert!(!pending().lock().unwrap().contains_key("test-page"));
    }
}
