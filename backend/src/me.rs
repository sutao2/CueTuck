use crate::require_user;
use crate::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MeProfile {
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
}

#[derive(Deserialize)]
pub struct MeUpdate {
    pub display_name: Option<String>,
    pub bio: Option<String>,
}

fn trim_profile_field(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub async fn get_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MeProfile>, StatusCode> {
    let email = require_user(&state, &headers).await?;
    Ok(Json(state.get_profile(&email).await?))
}

pub async fn put_me(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MeUpdate>,
) -> Result<Json<MeProfile>, StatusCode> {
    let email = require_user(&state, &headers).await?;
    Ok(Json(
        state
            .put_profile(
                &email,
                trim_profile_field(body.display_name),
                trim_profile_field(body.bio),
            )
            .await?,
    ))
}
