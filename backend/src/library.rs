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
        if item.payload.get("assets").is_some() || (item.kind != "prompt" && item.payload.get("asset_refs").is_some()) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetReference {
    pub id: String,
    pub media_id: String,
    pub name: String,
    pub mime: String,
    pub size: i64,
    pub sha256: String,
}

pub(crate) async fn validate_asset_refs(pg: &crate::postgres::Pg, owner: &str, items: &[LibraryChange]) -> Result<(), StatusCode> {
    for item in items {
        let Some(value) = item.payload.get("asset_refs") else { continue };
        let refs: Vec<AssetReference> = serde_json::from_value(value.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
        if refs.len() > 12 { return Err(StatusCode::PAYLOAD_TOO_LARGE); }
        let mut ids = std::collections::HashSet::new();
        let mut total = 0;
        for file in refs {
            if uuid::Uuid::parse_str(&file.id).is_err() || !ids.insert(file.id.clone()) || !(0..=crate::media::MAX_FILE as i64).contains(&file.size) {
                return Err(StatusCode::BAD_REQUEST);
            }
            total += file.size;
            if total > 20 * 1024 * 1024 { return Err(StatusCode::PAYLOAD_TOO_LARGE); }
            let valid: bool = sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE id=$1 AND owner_email=$2 AND ready=TRUE AND file_name=$3 AND content_type=$4 AND size=$5 AND sha256=$6)", pg.t("media_objects")))
                .bind(&file.media_id).bind(owner).bind(&file.name).bind(&file.mime).bind(file.size).bind(&file.sha256)
                .fetch_one(&pg.pool).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            if !valid { return Err(StatusCode::NOT_FOUND); }
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
