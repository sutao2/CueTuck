use crate::require_user;
use crate::AppState;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};

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
