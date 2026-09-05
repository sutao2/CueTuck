use crate::require_user;
use crate::AppState;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};

pub(crate) fn timestamp_ms(raw: &str) -> u64 {
    let value = raw.parse::<u64>().unwrap_or(0);
    if (1_000_000_000..100_000_000_000).contains(&value) { value * 1000 } else { value }
}

pub(crate) fn validate_changes(items: &[LibraryChange]) -> Result<(), StatusCode> {
    for item in items {
        if item.id.trim().is_empty() || !matches!(item.kind.as_str(), "prompt" | "collection" | "category" | "setting")
            || !item.payload.is_object() || item.updated_at.parse::<u64>().is_err()
            || item.updated_at.len() > 16 {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LibraryChange {
    pub id: String,
    pub kind: String,
    pub payload: serde_json::Value,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LibraryChangeList {
    pub items: Vec<LibraryChange>,
}

#[derive(Deserialize, Default)]
pub struct LibraryChangeQuery {
    pub since: Option<String>,
}

pub async fn list_changes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<LibraryChangeQuery>,
) -> Result<Json<LibraryChangeList>, StatusCode> {
    let email = require_user(&state, &headers).await?;
    Ok(Json(LibraryChangeList {
        items: state
            .list_library_changes(&email, query.since.as_deref().unwrap_or(""))
            .await?,
    }))
}

pub async fn push_changes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LibraryChangeList>,
) -> Result<Json<LibraryChangeList>, StatusCode> {
    let email = require_user(&state, &headers).await?;
    Ok(Json(LibraryChangeList {
        items: state.put_library_changes(&email, body.items).await?,
    }))
}
