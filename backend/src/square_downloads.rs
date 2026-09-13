use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

pub async fn record(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let count = state.increment_download(&id).await?;
    Ok(Json(json!({ "download_count": count })))
}
