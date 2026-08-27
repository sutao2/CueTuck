use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;

pub async fn record(State(state): State<AppState>, Path(id): Path<String>) -> StatusCode {
    match state.increment_download(&id).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(status) => status,
    }
}
