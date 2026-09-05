use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use promptark_api::{app, AppState};
use tower::ServiceExt;

async fn login(app: axum::Router) -> (axum::Router, promptark_api::SessionResponse) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/session")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"dev@promptark.local","password":"devpass"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (app, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn mock_payment_is_account_scoped_and_never_grants_real_pro() {
    let state = AppState::with_user("dev@promptark.local", "devpass")
        .with_billing_mock().with_stripe_secret("sk_live_must_never_be_called");
    let (app, session) = login(app(state)).await;
    for (outcome, expected) in [("failure", false), ("cancel", false), ("success", true), ("success", true), ("failure", true), ("cancel", true), ("reset", false)] {
        let response = app.clone().oneshot(Request::builder().method("POST").uri("/v1/billing/checkout")
            .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::json!({"mock_outcome": outcome}).to_string())).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let payload: serde_json::Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(payload["mock"], true);
        assert_eq!(payload["mock_pro"], expected);
        assert_eq!(payload["pro"], false);
        assert_eq!(payload["payment_enabled"], false);
        assert!(payload["checkout_url"].is_null());
        assert!(payload["note"].as_str().unwrap().contains("Mock"));
    }
    for (path, body) in [("checkout", r#"{"mock_outcome":"success"}"#), ("redeem", r#"{"code":"anything"}"#)] {
        let denied = app.clone().oneshot(Request::builder().method("POST").uri(format!("/v1/billing/{path}"))
            .header(header::CONTENT_TYPE, "application/json").body(Body::from(body)).unwrap()).await.unwrap();
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    }
    let redeem = app.oneshot(Request::builder().method("POST").uri("/v1/billing/redeem")
        .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
        .header(header::CONTENT_TYPE, "application/json").body(Body::from(r#"{"code":"anything"}"#)).unwrap()).await.unwrap();
    assert_eq!(redeem.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn mock_checkout_is_rejected_when_disabled() {
    let (app, session) = login(app(AppState::with_user("dev@promptark.local", "devpass"))).await;
    let response = app.oneshot(Request::builder().method("POST").uri("/v1/billing/checkout")
        .header(header::AUTHORIZATION, format!("Bearer {}", session.access_token))
        .header(header::CONTENT_TYPE, "application/json").body(Body::from(r#"{"mock_outcome":"success"}"#)).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn unsigned_status_is_not_pro_when_payment_is_unconfigured() {
    std::env::remove_var("STRIPE_SECRET_KEY");
    std::env::remove_var("PROMPTARK_STRIPE_SECRET");
    let (app, session) = login(app(AppState::with_user("dev@promptark.local", "devpass"))).await;
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/billing/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let listed = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/billing/status")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["pro"], false);
    assert_eq!(payload["payment_enabled"], false);
    assert!(payload["note"]
        .as_str()
        .unwrap_or("")
        .contains("支付未开通"));
    assert!(!payload.to_string().contains("商店"));
}

#[tokio::test]
async fn valid_redeem_code_marks_account_pro_and_cannot_be_reused() {
    std::env::remove_var("STRIPE_SECRET_KEY");
    std::env::remove_var("PROMPTARK_STRIPE_SECRET");
    let state = AppState::with_users(&[
        ("dev@promptark.local", "devpass", "user"),
        ("other@promptark.local", "otherpass", "user"),
    ])
    .with_redeem_code("PREVIEW-PRO");
    let (app, session) = login(app(state)).await;
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/redeem")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"code":"PREVIEW-PRO"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let redeemed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/redeem")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", session.access_token),
                )
                .body(Body::from(r#"{"code":"PREVIEW-PRO"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(redeemed.status(), StatusCode::OK);
    let body = to_bytes(redeemed.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["pro"], true);
    let listed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/billing/status")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let listed_body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let listed_payload: serde_json::Value = serde_json::from_slice(&listed_body).unwrap();
    assert_eq!(listed_payload["pro"], true);
    assert!(listed_payload["note"]
        .as_str()
        .unwrap_or("")
        .contains("支付未开通"));
    let reused = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/redeem")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", session.access_token),
                )
                .body(Body::from(r#"{"code":"PREVIEW-PRO"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reused.status(), StatusCode::CONFLICT);
    let other = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/session")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"other@promptark.local","password":"otherpass"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(other.status(), StatusCode::OK);
    let other_body = to_bytes(other.into_body(), usize::MAX).await.unwrap();
    let other_session: promptark_api::SessionResponse =
        serde_json::from_slice(&other_body).unwrap();
    let other_redeem = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/redeem")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", other_session.access_token),
                )
                .body(Body::from(r#"{"code":"PREVIEW-PRO"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(other_redeem.status(), StatusCode::CONFLICT);
    let other_status = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/billing/status")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", other_session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(other_status.status(), StatusCode::OK);
    let other_listed = to_bytes(other_status.into_body(), usize::MAX)
        .await
        .unwrap();
    let other_payload: serde_json::Value = serde_json::from_slice(&other_listed).unwrap();
    assert_eq!(other_payload["pro"], false);
}

#[tokio::test]
async fn checkout_requires_test_secret_and_does_not_mark_pro() {
    std::env::remove_var("STRIPE_SECRET_KEY");
    std::env::remove_var("PROMPTARK_STRIPE_SECRET");
    let (unconfigured_app, session) =
        login(app(AppState::with_user("dev@promptark.local", "devpass"))).await;
    let denied = unconfigured_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/checkout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let unconfigured = unconfigured_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/checkout")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unconfigured.status(), StatusCode::CONFLICT);
    let unconfigured_body = to_bytes(unconfigured.into_body(), usize::MAX)
        .await
        .unwrap();
    let unconfigured_payload: serde_json::Value =
        serde_json::from_slice(&unconfigured_body).unwrap();
    assert!(unconfigured_payload["checkout_url"].is_null());
    assert!(unconfigured_payload["note"]
        .as_str()
        .unwrap_or("")
        .contains("支付未开通"));
    assert_eq!(unconfigured_payload["pro"], false);

    let (live_app, live_session) = login(app(
        AppState::with_user("dev@promptark.local", "devpass")
            .with_stripe_secret("sk_live_not_for_preview"),
    ))
    .await;
    let live = live_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/checkout")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", live_session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(live.status(), StatusCode::FORBIDDEN);
    let live_body = to_bytes(live.into_body(), usize::MAX).await.unwrap();
    let live_payload: serde_json::Value = serde_json::from_slice(&live_body).unwrap();
    assert!(live_payload["checkout_url"].is_null());
    assert!(!live_payload.to_string().contains("商店"));

    let (test_app, test_session) = login(app(
        AppState::with_user("dev@promptark.local", "devpass")
            .with_stripe_secret("sk_test_preview")
            .with_checkout_url("https://checkout.stripe.com/c/pay/cs_test_preview"),
    ))
    .await;
    let started = test_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/checkout")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", test_session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(started.status(), StatusCode::OK);
    let started_body = to_bytes(started.into_body(), usize::MAX).await.unwrap();
    let started_payload: serde_json::Value = serde_json::from_slice(&started_body).unwrap();
    assert_eq!(
        started_payload["checkout_url"],
        "https://checkout.stripe.com/c/pay/cs_test_preview"
    );
    assert_eq!(started_payload["pro"], false);
    let listed = test_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/billing/status")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", test_session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let listed_body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let listed_payload: serde_json::Value = serde_json::from_slice(&listed_body).unwrap();
    assert_eq!(listed_payload["pro"], false);
    assert_eq!(listed_payload["payment_enabled"], true);
}

fn stripe_signature(secret: &str, payload: &str, timestamp: i64) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(format!("{timestamp}.{payload}").as_bytes());
    let digest = mac.finalize().into_bytes();
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("t={timestamp},v1={hex}")
}

#[tokio::test]
async fn signed_checkout_webhook_marks_pro_and_rejects_invalid_signatures() {
    std::env::remove_var("STRIPE_SECRET_KEY");
    std::env::remove_var("PROMPTARK_STRIPE_SECRET");
    std::env::remove_var("STRIPE_WEBHOOK_SECRET");
    std::env::remove_var("PROMPTARK_STRIPE_WEBHOOK_SECRET");
    let secret = "whsec_preview";
    let payload = serde_json::json!({
        "type": "checkout.session.completed",
        "data": {
            "object": {
                "client_reference_id": "dev@promptark.local",
                "payment_status": "paid"
            }
        }
    })
    .to_string();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let (signed_app, session) = login(app(
        AppState::with_user("dev@promptark.local", "devpass").with_webhook_secret(secret),
    ))
    .await;
    let unsigned = signed_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/webhook")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unsigned.status(), StatusCode::BAD_REQUEST);
    let forged = signed_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/webhook")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    "Stripe-Signature",
                    stripe_signature("whsec_other", &payload, timestamp),
                )
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forged.status(), StatusCode::BAD_REQUEST);
    let listed = signed_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/billing/status")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let listed_body = to_bytes(listed.into_body(), usize::MAX).await.unwrap();
    let listed_payload: serde_json::Value = serde_json::from_slice(&listed_body).unwrap();
    assert_eq!(listed_payload["pro"], false);
    let accepted = signed_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/webhook")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    "Stripe-Signature",
                    stripe_signature(secret, &payload, timestamp),
                )
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(accepted.status(), StatusCode::OK);
    let after = signed_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/billing/status")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(after.status(), StatusCode::OK);
    let after_body = to_bytes(after.into_body(), usize::MAX).await.unwrap();
    let after_payload: serde_json::Value = serde_json::from_slice(&after_body).unwrap();
    assert_eq!(after_payload["pro"], true);
    let (unconfigured_app, unconfigured_session) =
        login(app(AppState::with_user("dev@promptark.local", "devpass"))).await;
    let skipped = unconfigured_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/billing/webhook")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    "Stripe-Signature",
                    stripe_signature(secret, &payload, timestamp),
                )
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(skipped.status(), StatusCode::CONFLICT);
    let skipped_status = unconfigured_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/billing/status")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", unconfigured_session.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let skipped_body = to_bytes(skipped_status.into_body(), usize::MAX)
        .await
        .unwrap();
    let skipped_payload: serde_json::Value = serde_json::from_slice(&skipped_body).unwrap();
    assert_eq!(skipped_payload["pro"], false);
}
