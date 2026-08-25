use crate::require_user;
use crate::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct BillingStatus {
    pub pro: bool,
    pub payment_enabled: bool,
    pub note: String,
}

#[derive(Deserialize)]
pub struct RedeemRequest {
    pub code: String,
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

fn payment_note(payment_enabled: bool) -> String {
    if payment_enabled {
        String::new()
    } else {
        "支付未开通".into()
    }
}

async fn status_for(state: &AppState, email: &str) -> Result<BillingStatus, StatusCode> {
    let payment_enabled = payment_configured();
    Ok(BillingStatus {
        pro: state.account_is_pro(email).await?,
        payment_enabled,
        note: payment_note(payment_enabled),
    })
}

pub async fn status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<BillingStatus>, StatusCode> {
    let email = require_user(&state, &headers).await?;
    Ok(Json(status_for(&state, &email).await?))
}

pub async fn redeem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RedeemRequest>,
) -> Result<Json<BillingStatus>, StatusCode> {
    let email = require_user(&state, &headers).await?;
    state.redeem_code(&email, &body.code).await?;
    Ok(Json(status_for(&state, &email).await?))
}
