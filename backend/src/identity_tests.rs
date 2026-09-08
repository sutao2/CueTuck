use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::{json, Value};
#[tokio::test]
async fn role_actions_permanently_revoke_pending_invitations() {
    let (state, owner) = setup().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("second@example.com", Some("second-password"), "owner")
        .await
        .unwrap();
    let second = state
        .issue_session("second@example.com".into())
        .await
        .unwrap()
        .access_token;
    request(
        &state,
        "POST",
        "/v1/admin/identity/invitations",
        &owner,
        json!({"email":"pending@example.com","role":"admin","current_password":"test-password"}),
    )
    .await;
    let token = code(&state, "pending@example.com").await;
    for role in ["admin", "owner"] {
        assert_eq!(
            request(
                &state,
                "POST",
                "/v1/admin/users/owner%40identity.test/actions",
                &second,
                json!({"action":"role","role":role,"current_password":"second-password"})
            )
            .await
            .0,
            StatusCode::OK
        );
    }
    assert_eq!(request(&state,"POST","/v1/session/identity/confirm","",json!({"kind":"invitation","email":"pending@example.com","token":token,"new_password":"new-password-123"})).await.0,StatusCode::BAD_REQUEST);
    let status: String = sqlx::query_scalar(&format!(
        "SELECT status FROM {} WHERE email='pending@example.com'",
        pg.t("identity_challenges")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(status, "revoked");
}
#[tokio::test]
async fn registration_policy_applies_to_oauth_without_blocking_existing_accounts() {
    let (state, owner) = setup().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("Existing@Example.com", Some("test-password"), "user")
        .await
        .unwrap();
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/identity/policy",
            &owner,
            json!({"revision":0,"registration_open":false})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/identity/policy",
            &owner,
            json!({"revision":0,"registration_open":true})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/session/identity/request",
            "",
            json!({"kind":"registration","email":"new@example.com"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        pg.link_oauth("google", "new-id", "new@example.com")
            .await
            .unwrap_err(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        pg.link_oauth("google", "existing-id", "existing@example.com")
            .await
            .unwrap(),
        "Existing@Example.com"
    );
    assert_eq!(
        pg.link_oauth("google", "different-id", "existing@example.com")
            .await
            .unwrap_err(),
        StatusCode::CONFLICT
    );
    assert!(pg
        .password_session("Existing@Example.com", "test-password")
        .await
        .is_ok());
    sqlx::query(&format!(
        "UPDATE {} SET data=jsonb_set(data,'{{enabled}}','false')",
        pg.t("mail_configuration")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    let a = request(
        &state,
        "POST",
        "/v1/session/identity/request",
        "",
        json!({"kind":"reset","email":"existing@example.com"}),
    )
    .await;
    let b = request(
        &state,
        "POST",
        "/v1/session/identity/request",
        "",
        json!({"kind":"reset","email":"absent@example.com"}),
    )
    .await;
    assert_eq!(a, b);
    assert_eq!(a.0, StatusCode::SERVICE_UNAVAILABLE);
}
#[tokio::test]
async fn expired_codes_and_failed_audit_never_change_passwords_or_consume_challenges() {
    let (state, _) = setup().await;
    let pg = state.db.as_ref().unwrap();
    let email = "reset@example.com";
    pg.upsert_account(email, Some("old-password"), "user")
        .await
        .unwrap();
    let session = pg.password_session(email, "old-password").await.unwrap();
    request(
        &state,
        "POST",
        "/v1/session/identity/request",
        "",
        json!({"kind":"reset","email":email}),
    )
    .await;
    let token = code(&state, email).await;
    let body =
        json!({"kind":"reset","email":email,"token":token,"new_password":"replacement-password"});
    sqlx::query(&format!(
        "UPDATE {} SET expires_at=now()-interval '1 second'",
        pg.t("identity_challenges")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/session/identity/confirm",
            "",
            body.clone()
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    sqlx::query(&format!(
        "UPDATE {} SET expires_at=now()+interval '1 minute'",
        pg.t("identity_challenges")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    sqlx::query(&format!(
        "ALTER TABLE {} ADD CONSTRAINT identity_audit_failure CHECK(action<>'identity_verified')",
        pg.t("security_audit")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/session/identity/confirm",
            "",
            body.clone()
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert!(pg.password_session(email, "old-password").await.is_ok());
    assert!(state
        .email_for_access(&session.access_token)
        .await
        .unwrap()
        .is_some());
    let status: String = sqlx::query_scalar(&format!(
        "SELECT status FROM {} WHERE email=$1",
        pg.t("identity_challenges")
    ))
    .bind(email)
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(status, "pending");
    sqlx::query(&format!(
        "ALTER TABLE {} DROP CONSTRAINT identity_audit_failure",
        pg.t("security_audit")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(&state, "POST", "/v1/session/identity/confirm", "", body)
            .await
            .0,
        StatusCode::OK
    );
}
#[tokio::test]
async fn daily_mail_budget_and_admin_reset_permissions_are_enforced() {
    let (state, owner) = setup().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("admin@example.com", Some("admin-password"), "admin")
        .await
        .unwrap();
    pg.upsert_account("user@example.com", Some("user-password"), "user")
        .await
        .unwrap();
    let admin = state
        .issue_session("admin@example.com".into())
        .await
        .unwrap()
        .access_token;
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/users/owner%40identity.test/password-reset",
            &admin,
            json!({"current_password":"admin-password"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/users/user%40example.com/password-reset",
            &admin,
            json!({"current_password":"wrong"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/users/user%40example.com/password-reset",
            &admin,
            json!({"current_password":"admin-password"})
        )
        .await
        .0,
        StatusCode::ACCEPTED
    );
    assert!(pg
        .password_session("user@example.com", "user-password")
        .await
        .is_ok());
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/users/admin%40example.com/password-reset",
            &owner,
            json!({"current_password":"test-password"})
        )
        .await
        .0,
        StatusCode::ACCEPTED
    );
    for _ in 0..6 {
        assert_eq!(
            request(
                &state,
                "POST",
                "/v1/session/identity/request",
                "",
                json!({"kind":"registration","email":"budget@example.com"})
            )
            .await
            .0,
            StatusCode::ACCEPTED
        );
    }
    let count: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {} WHERE recipient='budget@example.com'",
        pg.t("mail_outbox")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(count, 5);
}
async fn setup() -> (AppState, String) {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("owner@identity.test", Some("test-password"), "owner")
        .await
        .unwrap();
    let owner = state
        .issue_session("owner@identity.test".into())
        .await
        .unwrap()
        .access_token;
    let secret = crate::oauth_admin::seal(&state.oauth_config.key, "smtp", "test-secret").unwrap();
    sqlx::query(&format!("UPDATE {} SET data=$1 WHERE id=1",pg.t("mail_configuration"))).bind(json!({"revision":1,"enabled":true,"host":"smtp.example.com","port":465,"tls":"implicit","from":"test@example.com","username":"test","encrypted_secret":secret})).execute(&pg.pool).await.unwrap();
    (state, owner)
}
async fn code(state: &AppState, email: &str) -> String {
    let pg = state.db.as_ref().unwrap();
    let row: Value = sqlx::query_scalar(&format!(
        "SELECT to_jsonb(m) FROM {} m WHERE recipient=$1 ORDER BY created_at DESC LIMIT 1",
        pg.t("mail_outbox")
    ))
    .bind(email)
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    let raw = crate::oauth_admin::unseal(
        &state.oauth_config.key,
        &format!("mail:{}", row["id"].as_str().unwrap()),
        row["payload"].as_str().unwrap(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&raw).unwrap();
    value["body"]
        .as_str()
        .unwrap()
        .split("验证码：\n")
        .nth(1)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .into()
}
#[tokio::test]
async fn registration_is_verified_once_and_reset_revokes_existing_sessions() {
    let (state, _) = setup().await;
    let path = "/v1/session/identity/request";
    let data = json!({"kind":"registration","email":"New@Example.com"});
    let (status, response) = request(&state, "POST", path, "", data.clone()).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(!response.to_string().contains("token"));
    let pg = state.db.as_ref().unwrap();
    assert!(pg
        .verify_login("new@example.com", "new-password-123")
        .await
        .is_err());
    let old = code(&state, "new@example.com").await;
    request(&state, "POST", path, "", data).await;
    let token = code(&state, "new@example.com").await;
    assert_ne!(old, token);
    let mut confirm = json!({"kind":"registration","email":"new@example.com","token":old,"new_password":"new-password-123"});
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/session/identity/confirm",
            "",
            confirm.clone()
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    confirm["token"] = json!(token);
    let (a, b) = tokio::join!(
        request(
            &state,
            "POST",
            "/v1/session/identity/confirm",
            "",
            confirm.clone()
        ),
        request(&state, "POST", "/v1/session/identity/confirm", "", confirm)
    );
    assert_eq!(
        [a.0, b.0].iter().filter(|s| **s == StatusCode::OK).count(),
        1
    );
    let session = pg
        .password_session("new@example.com", "new-password-123")
        .await
        .unwrap();
    let known = request(
        &state,
        "POST",
        path,
        "",
        json!({"kind":"reset","email":"new@example.com"}),
    )
    .await;
    let unknown = request(
        &state,
        "POST",
        path,
        "",
        json!({"kind":"reset","email":"absent@example.com"}),
    )
    .await;
    assert_eq!(known, unknown);
    let reset = code(&state, "new@example.com").await;
    assert_eq!(request(&state,"POST","/v1/session/identity/confirm","",json!({"kind":"reset","email":"new@example.com","token":reset,"new_password":"replacement-pass"})).await.0,StatusCode::OK);
    assert!(state
        .email_for_access(&session.access_token)
        .await
        .unwrap()
        .is_none());
    assert!(pg
        .password_session("new@example.com", "new-password-123")
        .await
        .is_err());
    assert!(pg
        .password_session("new@example.com", "replacement-pass")
        .await
        .is_ok());
}
#[tokio::test]
async fn invitations_cannot_overwrite_accounts_or_survive_inviter_revocation() {
    let (state, owner) = setup().await;
    let pg = state.db.as_ref().unwrap();
    let invite = json!({"email":"reviewer@example.com","role":"reviewer","current_password":"test-password"});
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/identity/invitations",
            &owner,
            invite.clone()
        )
        .await
        .0,
        StatusCode::ACCEPTED
    );
    let token = code(&state, "reviewer@example.com").await;
    pg.upsert_account("owner@identity.test", None, "admin")
        .await
        .unwrap();
    assert_eq!(request(&state,"POST","/v1/session/identity/confirm","",json!({"kind":"invitation","email":"reviewer@example.com","token":token,"new_password":"reviewer-password"})).await.0,StatusCode::BAD_REQUEST);
    pg.upsert_account("owner@identity.test", None, "owner")
        .await
        .unwrap();
    let (status, _) = request(
        &state,
        "POST",
        "/v1/admin/identity/invitations",
        &owner,
        invite,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let token = code(&state, "reviewer@example.com").await;
    assert_eq!(request(&state,"POST","/v1/session/identity/confirm","",json!({"kind":"invitation","email":"reviewer@example.com","token":token,"new_password":"reviewer-password"})).await.0,StatusCode::OK);
    assert_eq!(
        pg.role_of("reviewer@example.com").await.unwrap(),
        "reviewer"
    );
    assert_eq!(request(&state,"POST","/v1/admin/identity/invitations",&owner,json!({"email":"reviewer@example.com","role":"admin","current_password":"test-password"})).await.0,StatusCode::CONFLICT);
    let (_, list) = request(
        &state,
        "GET",
        "/v1/admin/identity/invitations",
        &owner,
        json!({}),
    )
    .await;
    assert!(!list.to_string().contains("token_hash"));
    assert!(!list.to_string().contains(&token));
}
