use crate::require_user;
use crate::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct BillingStatus {
    pub pro: bool,
    pub payment_enabled: bool,
    pub note: String,
}

fn payment_configured() -> bool {
    ["STRIPE_SECRET_KEY", "PROMPTARK_STRIPE_SECRET"]
        .iter()
        .any(|key| {
            std::env::var(key)
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
        })
}

pub async fn status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<BillingStatus>, StatusCode> {
    let _email = require_user(&state, &headers).await?;
    let payment_enabled = payment_configured();
    Ok(Json(BillingStatus {
        pro: false,
        payment_enabled,
        note: if payment_enabled {
            String::new()
        } else {
            "支付未开通".into()
        },
    }))
}
