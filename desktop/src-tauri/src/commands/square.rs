use serde::Deserialize;

#[derive(Deserialize)]
struct SquareListResponse {
    items: Vec<serde_json::Value>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct SquareContentResponse {
    #[serde(default)]
    publisher: Option<PublicPublisher>,
    #[serde(default)]
    translations: serde_json::Value,
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
    cover: Option<serde_json::Value>,
    #[serde(default)]
    asset_refs: Vec<super::media::Reference>,
}

#[derive(Deserialize, serde::Serialize)]
struct PublicPublisher {
    display_name: String,
    bio: Option<String>,
}

fn prompt_kind() -> String { "prompt".into() }

use crate::api_config::api_base;

#[tauri::command]
pub async fn square_reports(access_token:String,config:Option<serde_json::Value>,offset:Option<u32>)->Result<serde_json::Value,String> {
    let client=crate::http::client()?;
    let request=if let Some(config)=config {client.post(format!("{}/v1/reports",api_base()?)).json(&config)} else {client.get(format!("{}/v1/reports",api_base()?)).query(&[("offset",offset.unwrap_or(0))])};
    let response=request.bearer_auth(access_token).send().await.map_err(|_|"举报服务连接失败".to_string())?;
    super::ensure_authorized(response.status())?;
    if !response.status().is_success(){return Err(match response.status().as_u16(){401=>"登录已失效，请重新登录",404=>"该内容已不可举报",429=>"每天最多提交 20 件举报，请稍后重试",_=>"举报请求失败，请重试"}.into());}
    response.json().await.map_err(|_|"举报响应无效".into())
}

#[tauri::command]
pub async fn get_square_catalog() -> Result<serde_json::Value, String> {
    let response = crate::http::client()?.get(format!("{}/v1/square/catalog",api_base()?)).send().await.map_err(|error|error.to_string())?;
    super::ensure_authorized(response.status())?;
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
    super::ensure_authorized(response.status())?;
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
    super::ensure_authorized(response.status())?;
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
    super::ensure_authorized(response.status())?;
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
    cover: Option<serde_json::Value>,
    asset_refs: Option<Vec<super::media::Reference>>,
    revision: Option<serde_json::Value>,
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
            "cover": cover,
            "asset_refs": asset_refs.unwrap_or_default(),
            "revision": revision,
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    super::ensure_authorized(response.status())?;
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
    super::ensure_authorized(response.status())?;
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
    super::ensure_authorized(response.status())?;
    if !response.status().is_success() {
        return Err("收藏失败".to_string());
    }
    if method == "DELETE" {
        return Ok(serde_json::json!({ "ok": true }));
    }
    response.json().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn square_translations(id:String,target:Option<String>,access_token:Option<String>)->Result<serde_json::Value,String>{
 let mut url=url::Url::parse(&api_base()?).map_err(|_|"服务地址无效")?;
 {let mut path=url.path_segments_mut().map_err(|_|"服务地址无效")?;path.extend(["v1","square","items",&id,"translations"]);if let Some(t)=&target{if t!="zh"&&t!="en"{return Err("语言无效".into())}path.push(t);}}
 let client=crate::http::client()?;let mut req=if target.is_some(){client.post(url)}else{client.get(url)};
 if let Some(token)=access_token{req=req.bearer_auth(token)}
 let response=req.send().await.map_err(|_|"翻译服务连接失败")?;
 super::ensure_authorized(response.status())?;
    if !response.status().is_success(){return Err(match response.status().as_u16(){401=>"请先登录再生成广场译文",429=>"今日翻译次数已用完，请稍后重试",503=>"广场翻译暂时暂停，请稍后重试",404=>"该提示词已不可用",_=>"读取翻译失败，请重试"}.into())}
 response.json().await.map_err(|_|"翻译响应无效".into())
}

#[cfg(test)]
mod publisher_tests {
    use super::*;
    #[test]
    fn collection_content_preserves_cover_manifest() {
        let payload=serde_json::json!({"id":"collection","title":"Collection","content":"","category_id":null,"model":null,"kind":"collection","members":[{"title":"Member","content":"Body"}],"cover":{"layout":"grid","asset_ids":["cover-id"]}});
        let response:SquareContentResponse=serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(serde_json::to_value(response).unwrap()["cover"],payload["cover"]);
    }

    #[test]
    fn detail_preserves_public_publisher_and_accepts_older_servers() {
        let mut payload=serde_json::json!({"id":"item","title":"Title","content":"body","category_id":null,"model":null});
        let legacy:SquareContentResponse=serde_json::from_value(payload.clone()).unwrap();
        assert!(legacy.publisher.is_none());
        payload["publisher"]=serde_json::json!({"display_name":"公开昵称","bio":"公开简介","email":"private@example.test"});
        let current:SquareContentResponse=serde_json::from_value(payload).unwrap();
        let forwarded=serde_json::to_value(current).unwrap();
        assert_eq!(forwarded["publisher"],serde_json::json!({"display_name":"公开昵称","bio":"公开简介"}));
    }
}

#[tauri::command]
pub async fn content_versions_request(kind: String, id: String, version: Option<i64>, access_token: Option<String>) -> Result<serde_json::Value,String> {
    if !matches!(kind.as_str(),"prompt"|"collection"|"skill") || id.is_empty() || version.is_some_and(|v|v<1) {return Err("无效的版本请求".into())}
    let mut url=url::Url::parse(&api_base()?).map_err(|_|"服务地址无效")?;
    {let mut path=url.path_segments_mut().map_err(|_|"服务地址无效")?;path.extend(["v1","content",&kind,&id,"versions"]);if let Some(v)=version {path.push(&v.to_string());}}
    let mut request=crate::http::client()?.get(url);
    if let Some(token)=access_token {request=request.bearer_auth(token);}
    let response=request.send().await.map_err(|e|e.to_string())?;
    super::ensure_authorized(response.status())?;
    if !response.status().is_success(){return Err(format!("读取版本失败（{}）",response.status().as_u16()))}
    response.json().await.map_err(|e|e.to_string())
}

#[tauri::command]
pub async fn withdraw_publication(id:String,access_token:String)->Result<serde_json::Value,String> {
    let mut url=url::Url::parse(&api_base()?).map_err(|_|"服务地址无效")?;
    url.path_segments_mut().map_err(|_|"服务地址无效")?.extend(["v1","publications",&id,"withdraw"]);
    let response=crate::http::client()?.post(url).bearer_auth(access_token).send().await.map_err(|e|e.to_string())?;
    super::ensure_authorized(response.status())?;
    if !response.status().is_success(){return Err("撤回失败，投稿可能已经完成审核，请刷新后查看".into())}
    response.json().await.map_err(|e|e.to_string())
}
