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

#[derive(Serialize)]
pub struct CheckoutResponse {
    pub pro: bool,
    pub payment_enabled: bool,
    pub note: String,
    pub checkout_url: Option<String>,
}

#[derive(Deserialize)]
pub struct RedeemRequest {
    pub code: String,
}

fn configured_secret(state: &AppState) -> Option<String> {
    if let Some(secret) = state
        .stripe_secret
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(secret.to_string());
    }
    ["STRIPE_SECRET_KEY", "PROMPTARK_STRIPE_SECRET"]
        .iter()
        .find_map(|key| {
            std::env::var(key)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
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
    let payment_enabled = configured_secret(state).is_some();
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

pub async fn checkout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<CheckoutResponse>), StatusCode> {
    let email = require_user(&state, &headers).await?;
    let pro = state.account_is_pro(&email).await?;
    let Some(secret) = configured_secret(&state) else {
        return Ok((
            StatusCode::CONFLICT,
            Json(CheckoutResponse {
                pro,
                payment_enabled: false,
                note: "支付未开通".into(),
                checkout_url: None,
            }),
        ));
    };
    if !secret.starts_with("sk_test_") {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(CheckoutResponse {
                pro,
                payment_enabled: true,
                note: "预发只接受测试密钥".into(),
                checkout_url: None,
            }),
        ));
    }
    let checkout_url = if let Some(url) = state.checkout_url.clone() {
        url
    } else {
        create_stripe_session(&secret, &email).await?
    };
    Ok((
        StatusCode::OK,
        Json(CheckoutResponse {
            pro,
            payment_enabled: true,
            note: String::new(),
            checkout_url: Some(checkout_url),
        }),
    ))
}

async fn create_stripe_session(secret: &str, email: &str) -> Result<String, StatusCode> {
    let success = std::env::var("PROMPTARK_STRIPE_SUCCESS_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:1420/?billing=success".into());
    let cancel = std::env::var("PROMPTARK_STRIPE_CANCEL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:1420/?billing=cancel".into());
    let response = reqwest::Client::new()
        .post("https://api.stripe.com/v1/checkout/sessions")
        .basic_auth(secret, None::<&str>)
        .form(&[
            ("mode", "payment"),
            ("success_url", success.as_str()),
            ("cancel_url", cancel.as_str()),
            ("client_reference_id", email),
            ("line_items[0][quantity]", "1"),
            ("line_items[0][price_data][currency]", "cny"),
            ("line_items[0][price_data][unit_amount]", "1"),
            (
                "line_items[0][price_data][product_data][name]",
                "PromptArk Pro",
            ),
        ])
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    payload["url"]
        .as_str()
        .filter(|url| url.starts_with("https://"))
        .map(str::to_string)
        .ok_or(StatusCode::BAD_GATEWAY)
}
