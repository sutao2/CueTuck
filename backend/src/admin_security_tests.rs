use super::*;
use axum::body::{to_bytes, Body};
use axum::http::Request;
use serde_json::{json, Value};
use tower::ServiceExt;

pub(super) async fn state() -> AppState {
    let url = std::env::var("PROMPTARK_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://pl:pl@127.0.0.1:5432/promptark?sslmode=disable".into());
    let pool = sqlx::PgPool::connect(&url)
        .await
        .expect("local Postgres required");
    AppState::from_pool(pool, &format!("security_{}", Uuid::new_v4().simple()))
        .await
        .unwrap()
}

pub(super) async fn request(
    state: &AppState,
    method: &str,
    path: &str,
    token: &str,
    body: Value,
) -> (StatusCode, Value) {
    let response = app(state.clone())
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn bootstrap_is_once_only_and_preserves_accounts() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    assert!(pg.initialize_admin(None, None, false).await.is_err());
    pg.upsert_account("user@example.com", Some("ordinary-pass"), "user")
        .await
        .unwrap();
    assert!(pg
        .initialize_admin(Some("user@example.com"), Some("new-admin-password"), false)
        .await
        .is_err());
    assert_eq!(pg.role_of("user@example.com").await.unwrap(), "user");
    let (a, b) = tokio::join!(
        pg.initialize_admin(Some("first@example.com"), Some("first-password"), false),
        pg.initialize_admin(Some("second@example.com"), Some("second-password"), false)
    );
    assert!(a.is_ok() && b.is_ok());
    let users = pg.list_users().await.unwrap();
    assert_eq!(users.iter().filter(|u| u.role == "owner").count(), 1);
    let admin = users.iter().find(|u| u.role == "owner").unwrap();
    pg.upsert_account(&admin.email, Some("changed-password"), "admin")
        .await
        .unwrap();
    pg.initialize_admin(Some(&admin.email), Some("replacement-password"), true)
        .await
        .unwrap();
    pg.verify_login(&admin.email, "changed-password")
        .await
        .unwrap();
    pg.upsert_account(&admin.email, None, "user").await.unwrap();
    assert!(pg
        .initialize_admin(Some("third@example.com"), Some("third-password"), false)
        .await
        .is_err());
    assert!(!pg
        .list_users()
        .await
        .unwrap()
        .iter()
        .any(|u| u.email == "third@example.com"));
}

#[tokio::test]
async fn existing_admin_and_development_seeds_never_overwrite_password_or_role() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("existing@example.com", Some("existing-password"), "admin")
        .await
        .unwrap();
    pg.initialize_admin(None, None, false).await.unwrap();
    assert_eq!(pg.list_users().await.unwrap().len(), 1);
    pg.insert_development_user("existing@example.com", "devpass")
        .await
        .unwrap();
    pg.verify_login("existing@example.com", "existing-password")
        .await
        .unwrap();
    assert_eq!(pg.role_of("existing@example.com").await.unwrap(), "admin");
}

#[tokio::test]
async fn password_change_revokes_every_session_persists_and_is_account_scoped() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("admin@example.com", Some("current-password"), "admin")
        .await
        .unwrap();
    pg.upsert_account("other@example.com", Some("other-password"), "admin")
        .await
        .unwrap();
    let old = state
        .issue_session("admin@example.com".into())
        .await
        .unwrap();
    let second = state
        .issue_session("admin@example.com".into())
        .await
        .unwrap();
    let other = state
        .issue_session("other@example.com".into())
        .await
        .unwrap();
    let (status, view) = request(
        &state,
        "GET",
        "/v1/admin/security",
        &old.access_token,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["active_access_count"], 2);
    let (status, result) = request(
        &state,
        "PUT",
        "/v1/admin/security/password",
        &old.access_token,
        json!({"current_password":"current-password","new_password":"changed-password"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result, json!({"signed_out":true}));
    let restarted = AppState {
        db: state.db.clone(),
        ..AppState::default()
    };
    for session in [old, second] {
        assert_eq!(
            request(
                &restarted,
                "GET",
                "/v1/admin/me",
                &session.access_token,
                json!({})
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            request(
                &restarted,
                "POST",
                "/v1/session/refresh",
                "",
                json!({"refresh_token":session.refresh_token})
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        request(
            &restarted,
            "GET",
            "/v1/admin/me",
            &other.access_token,
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &restarted,
            "POST",
            "/v1/session",
            "",
            json!({"email":"admin@example.com","password":"current-password"})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &restarted,
            "POST",
            "/v1/session",
            "",
            json!({"email":"admin@example.com","password":"changed-password"})
        )
        .await
        .0,
        StatusCode::OK
    );
    let actions: Vec<String> =
        sqlx::query_scalar(&format!("SELECT action FROM {}", pg.t("security_audit")))
            .fetch_all(&pg.pool)
            .await
            .unwrap();
    assert_eq!(actions, vec!["password_changed"]);
}

#[tokio::test]
async fn security_rejects_wrong_password_invalid_fields_and_non_admins() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    for (email, role) in [("admin@example.com", "admin"), ("user@example.com", "user")] {
        pg.upsert_account(email, Some("current-password"), role)
            .await
            .unwrap();
    }
    let admin = state
        .issue_session("admin@example.com".into())
        .await
        .unwrap();
    let user = state
        .issue_session("user@example.com".into())
        .await
        .unwrap();
    for (method, path) in [
        ("GET", "/v1/admin/security"),
        ("DELETE", "/v1/admin/security/sessions"),
        ("PUT", "/v1/admin/security/password"),
    ] {
        let body = json!({"current_password":"current-password","new_password":"changed-password"});
        assert_eq!(
            request(&state, method, path, "", body.clone()).await.0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            request(&state, method, path, &user.access_token, body)
                .await
                .0,
            StatusCode::FORBIDDEN
        );
    }
    for (body, expected) in [
        (
            json!({"current_password":"wrong","new_password":"changed-password"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({"current_password":"current-password","new_password":"short"}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"current_password":"current-password","new_password":"current-password"}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"current_password":"current-password","new_password":"changed-password","email":"other@example.com"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ] {
        assert_eq!(
            request(
                &state,
                "PUT",
                "/v1/admin/security/password",
                &admin.access_token,
                body
            )
            .await
            .0,
            expected
        );
    }
    pg.verify_login("admin@example.com", "current-password")
        .await
        .unwrap();
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/admin/me",
            &admin.access_token,
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    let count: i64 =
        sqlx::query_scalar(&format!("SELECT count(*) FROM {}", pg.t("security_audit")))
            .fetch_one(&pg.pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn revoke_all_sessions_and_audit_failure_rollback() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("admin@example.com", Some("current-password"), "admin")
        .await
        .unwrap();
    let session = state
        .issue_session("admin@example.com".into())
        .await
        .unwrap();
    sqlx::query(&format!(
        "ALTER TABLE {} ADD CONSTRAINT audit_failure CHECK (action = 'never')",
        pg.t("security_audit")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/security/password",
            &session.access_token,
            json!({"current_password":"current-password","new_password":"changed-password"})
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    pg.verify_login("admin@example.com", "current-password")
        .await
        .unwrap();
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/security/sessions",
            &session.access_token,
            json!({})
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/admin/me",
            &session.access_token,
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    sqlx::query(&format!(
        "ALTER TABLE {} DROP CONSTRAINT audit_failure",
        pg.t("security_audit")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/security/sessions",
            &session.access_token,
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/session/refresh",
            "",
            json!({"refresh_token":session.refresh_token})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn racing_old_password_login_cannot_survive_password_change() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("admin@example.com", Some("current-password"), "admin")
        .await
        .unwrap();
    let session = state
        .issue_session("admin@example.com".into())
        .await
        .unwrap();
    // Hold the account lock while starting the request; either login/change ordering must be safe.
    let mut lock = pg.pool.begin().await.unwrap();
    sqlx::query(&format!(
        "SELECT email FROM {} WHERE email=$1 FOR UPDATE",
        pg.t("accounts")
    ))
    .bind("admin@example.com")
    .fetch_one(&mut *lock)
    .await
    .unwrap();
    let router_state = state.clone();
    let old_access = session.access_token.clone();
    let change = tokio::spawn(async move {
        request(
            &router_state,
            "PUT",
            "/v1/admin/security/password",
            &old_access,
            json!({"current_password":"current-password","new_password":"changed-password"}),
        )
        .await
    });
    lock.commit().await.unwrap();
    let (change_result, login_result) = tokio::join!(
        change,
        request(
            &state,
            "POST",
            "/v1/session",
            "",
            json!({"email":"admin@example.com","password":"current-password"})
        )
    );
    assert_eq!(change_result.unwrap().0, StatusCode::OK);
    if login_result.0 == StatusCode::OK {
        assert_eq!(
            request(
                &state,
                "GET",
                "/v1/admin/me",
                login_result.1["access_token"].as_str().unwrap(),
                json!({})
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            request(
                &state,
                "POST",
                "/v1/session/refresh",
                "",
                json!({"refresh_token":login_result.1["refresh_token"]})
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
    } else {
        assert_eq!(login_result.0, StatusCode::UNAUTHORIZED);
    }
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/session/refresh",
            "",
            json!({"refresh_token":session.refresh_token})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn racing_refresh_cannot_resurrect_revoked_sessions() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("admin@example.com", Some("current-password"), "admin")
        .await
        .unwrap();
    for _ in 0..8 {
        let session = state
            .issue_session("admin@example.com".into())
            .await
            .unwrap();
        let (revoked, refreshed) = tokio::join!(
            request(
                &state,
                "DELETE",
                "/v1/admin/security/sessions",
                &session.access_token,
                json!({})
            ),
            request(
                &state,
                "POST",
                "/v1/session/refresh",
                "",
                json!({"refresh_token":session.refresh_token})
            ),
        );
        if revoked.0 == StatusCode::UNAUTHORIZED {
            // Refresh won and invalidated the request's Access; explicitly revoke using the new session.
            assert_eq!(refreshed.0, StatusCode::OK);
            assert_eq!(
                request(
                    &state,
                    "DELETE",
                    "/v1/admin/security/sessions",
                    refreshed.1["access_token"].as_str().unwrap(),
                    json!({})
                )
                .await
                .0,
                StatusCode::OK
            );
        } else {
            assert_eq!(revoked.0, StatusCode::OK);
        }
        if refreshed.0 == StatusCode::OK {
            assert_eq!(
                request(
                    &state,
                    "GET",
                    "/v1/admin/me",
                    refreshed.1["access_token"].as_str().unwrap(),
                    json!({})
                )
                .await
                .0,
                StatusCode::UNAUTHORIZED
            );
            assert_eq!(
                request(
                    &state,
                    "POST",
                    "/v1/session/refresh",
                    "",
                    json!({"refresh_token":refreshed.1["refresh_token"]})
                )
                .await
                .0,
                StatusCode::UNAUTHORIZED
            );
        } else {
            assert_eq!(refreshed.0, StatusCode::UNAUTHORIZED);
        }
    }
}

#[tokio::test]
async fn session_pairs_and_refresh_rollback_together() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("admin@example.com", Some("current-password"), "admin")
        .await
        .unwrap();
    let session = state
        .issue_session("admin@example.com".into())
        .await
        .unwrap();
    sqlx::query(&format!(
        "ALTER TABLE {} ADD CONSTRAINT fail_new_refresh CHECK (token = '{}') NOT VALID",
        pg.t("refresh_tokens"),
        session.refresh_token
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/session",
            "",
            json!({"email":"admin@example.com","password":"current-password"})
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/session/refresh",
            "",
            json!({"refresh_token":session.refresh_token})
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/admin/me",
            &session.access_token,
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        pg.security_view("admin@example.com").await.unwrap()["active_access_count"],
        1
    );
    sqlx::query(&format!(
        "ALTER TABLE {} DROP CONSTRAINT fail_new_refresh",
        pg.t("refresh_tokens")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/session/refresh",
            "",
            json!({"refresh_token":session.refresh_token})
        )
        .await
        .0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn explicit_development_bootstrap_and_passwordless_admin_boundaries() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    assert!(pg
        .initialize_admin(Some("admin@example.com"), Some("short"), false)
        .await
        .is_err());
    pg.initialize_admin(None, None, true).await.unwrap();
    pg.verify_login("admin@promptark.local", "adminpass")
        .await
        .unwrap();
    pg.upsert_account("oauth@example.com", None, "admin")
        .await
        .unwrap();
    let session = state
        .issue_session("oauth@example.com".into())
        .await
        .unwrap();
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/admin/security",
            &session.access_token,
            json!({})
        )
        .await
        .1["has_password"],
        false
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/security/password",
            &session.access_token,
            json!({"current_password":"","new_password":"new-password-long"})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/security/sessions",
            &session.access_token,
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert!(!admin_security::valid_password(&"a".repeat(129)));
    assert!(admin_security::valid_password(&"舟".repeat(12)));
}
