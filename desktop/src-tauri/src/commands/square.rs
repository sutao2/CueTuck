use crate::local_database::{import_downloaded_prompt_in_dir, PromptRecord};
use serde::Deserialize;
use tauri::{AppHandle, Manager};

#[derive(Deserialize)]
struct SquareListResponse {
    items: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct SquareContentResponse {
    id: String,
    title: String,
    content: String,
}

fn api_base() -> String {
    std::env::var("PROMPTARK_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:8787".into())
}

fn data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_square_items(sort: Option<String>, query: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/v1/square/items", api_base()))
        .query(&[
            ("sort", sort.unwrap_or_else(|| "推荐".into())),
            ("q", query.unwrap_or_default()),
        ])
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
pub async fn download_square_item(app: AppHandle, id: String) -> Result<PromptRecord, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/v1/square/items/{}/content", api_base(), id))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err("广场暂时不可用".to_string());
    }
    let payload: SquareContentResponse = response.json().await.map_err(|error| error.to_string())?;
    import_downloaded_prompt_in_dir(
        &data_dir(&app)?,
        &payload.title,
        &payload.content,
        Some(&payload.id),
    )
}
