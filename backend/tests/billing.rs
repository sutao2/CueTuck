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
