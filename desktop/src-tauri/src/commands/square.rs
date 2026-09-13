use serde::Deserialize;

#[derive(Deserialize)]
struct SquareListResponse {
    items: Vec<serde_json::Value>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct SquareContentResponse {
    #[serde(default)]
    reference: Option<serde_json::Value>,
    id: String,
    title: String,
    content: String,
    #[serde(default)]
    author: Option<String>,
    category_id: Option<String>,
    model: Option<String>,
    #[serde(default = "prompt_kind")]
    kind: String,
    #[serde(default)]
    members: Vec<serde_json::Value>,
    #[serde(default)]
    asset_refs: Vec<super::media::Reference>,
}

fn prompt_kind() -> String { "prompt".into() }

use crate::api_config::api_base;

#[tauri::command]
pub async fn square_reports(access_token:String,config:Option<serde_json::Value>,offset:Option<u32>)->Result<serde_json::Value,String> {
    let client=crate::http::client()?;
    let request=if let Some(config)=config {client.post(format!("{}/v1/reports",api_base()?)).json(&config)} else {client.get(format!("{}/v1/reports",api_base()?)).query(&[("offset",offset.unwrap_or(0))])};
    let response=request.bearer_auth(access_token).send().await.map_err(|_|"举报服务连接失败".to_string())?;
    if !response.status().is_success(){return Err(match response.status().as_u16(){401=>"登录已失效，请重新登录",404=>"该内容已不可举报",429=>"每天最多提交 20 件举报，请稍后重试",_=>"举报请求失败，请重试"}.into());}
    response.json().await.map_err(|_|"举报响应无效".into())
}

#[tauri::command]
pub async fn get_square_catalog() -> Result<serde_json::Value, String> {
    let response = crate::http::client()?.get(format!("{}/v1/square/catalog",api_base()?)).send().await.map_err(|error|error.to_string())?;
    if !response.status().is_success() { return Err("广场字典暂时不可用".into()); }
    response.json().await.map_err(|error|error.to_string())
}

#[tauri::command]
pub async fn list_square_items(
    sort: Option<String>,
    query: Option<String>,
    model: Option<String>,
    category_id: Option<String>,
    access_token: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = crate::http::client()?;
    let sort = sort.unwrap_or_else(|| "推荐".into());
    let query = query.unwrap_or_default();
    let model = model.unwrap_or_default();
    let mut request = client.get(format!("{}/v1/square/items", api_base()?));
    if let Some(token) = access_token { request = request.bearer_auth(token); }
    if let Some(category) = category_id {
        request = request.query(&[("category_id", category)]);
    }
    request = if model.trim().is_empty() {
        request.query(&[("sort", sort.as_str()), ("q", query.as_str())])
    } else {
        request.query(&[
            ("sort", sort.as_str()),
            ("q", query.as_str()),
            ("model", model.trim()),
        ])
    };
    let response = request
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("广场暂时不可用".to_string());
    }
    let payload: SquareListResponse = response.json().await.map_err(|error| error.to_string())?;
    Ok(payload.items)
}

#[tauri::command]
pub async fn get_square_content(id: String, access_token: Option<String>) -> Result<SquareContentResponse, String> {
    let client = crate::http::client()?;
    let mut request = client.get(format!("{}/v1/square/items/{}/content", api_base()?, id));
    if let Some(token) = access_token { request = request.bearer_auth(token); }
    let response = request
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("广场暂时不可用".to_string());
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn record_square_download(id: String) -> Result<serde_json::Value, String> {
    let client = crate::http::client()?;
    let response = client
        .post(format!("{}/v1/square/items/{}/downloads", api_base()?, id))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("download stats failed".into());
    }
    // Older servers acknowledged the write without a body.
    if response.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(serde_json::Value::Null);
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_publication(
    source_id: String,
    access_token: String,
    title: Option<String>,
    content: Option<String>,
    category_id: Option<String>,
    model: Option<String>,
    kind: Option<String>,
    members: Option<Vec<serde_json::Value>>,
    asset_refs: Option<Vec<super::media::Reference>>,
) -> Result<serde_json::Value, String> {
    if source_id.trim().is_empty() {
        return Err("未选择本地内容".to_string());
    }
    let client = crate::http::client()?;
    let response = client
        .post(format!("{}/v1/publications", api_base()?))
        .bearer_auth(&access_token)
        .json(&serde_json::json!({
            "source_id": source_id,
            "title": title,
            "content": content,
            "category_id": category_id,
            "model": model,
            "kind": kind.unwrap_or_else(prompt_kind),
            "members": members.unwrap_or_default(),
            "asset_refs": asset_refs.unwrap_or_default(),
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("发布失败".to_string());
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_my_publications(access_token: String) -> Result<serde_json::Value, String> {
    let client = crate::http::client()?;
    let response = client
        .get(format!("{}/v1/publications/mine", api_base()?))
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("发布列表暂时不可用".to_string());
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn put_favorite(id: String, access_token: String) -> Result<serde_json::Value, String> {
    favorite_request("PUT", Some(&id), &access_token).await
}

#[tauri::command]
pub async fn delete_favorite(id: String, access_token: String) -> Result<serde_json::Value, String> {
    favorite_request("DELETE", Some(&id), &access_token).await
}

#[tauri::command]
pub async fn list_favorites(access_token: String) -> Result<serde_json::Value, String> {
    favorite_request("GET", None, &access_token).await
}

async fn favorite_request(method: &str, id: Option<&str>, access_token: &str) -> Result<serde_json::Value, String> {
    let client = crate::http::client()?;
    let url = match id {
        Some(id) => format!("{}/v1/favorites/{id}", api_base()?),
        None => format!("{}/v1/favorites", api_base()?),
    };
    let builder = match method {
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => client.get(url),
    };
    let response = builder
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("收藏失败".to_string());
    }
    if method == "DELETE" {
        return Ok(serde_json::json!({ "ok": true }));
    }
    response.json().await.map_err(|error| error.to_string())
}
