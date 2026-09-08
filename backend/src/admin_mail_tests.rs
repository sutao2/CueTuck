use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::{json, Value};
#[tokio::test]
async fn smtp_configuration_and_outbox_are_encrypted_versioned_and_private() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("owner@mail.test", Some("test-password"), "owner")
        .await
        .unwrap();
    let owner = state
        .issue_session("owner@mail.test".into())
        .await
        .unwrap()
        .access_token;
    let mut config = json!({"revision":0,"enabled":false,"host":"smtp.example.com","port":465,"tls":"implicit","from":"noreply@example.com","username":"smtp-user","password":"smtp-password","current_password":"wrong"});
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/mail/config",
            &owner,
            config.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    config["current_password"] = json!("test-password");
    let (status, view) = request(
        &state,
        "PUT",
        "/v1/admin/mail/config",
        &owner,
        config.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["revision"], 1);
    assert_eq!(view["secret_configured"], true);
    assert!(!view.to_string().contains("smtp-password"));
    assert_eq!(
        request(&state, "PUT", "/v1/admin/mail/config", &owner, config)
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert!(pg.has_mail_secrets().await.unwrap());
    let (status, job) = request(
        &state,
        "POST",
        "/v1/admin/mail/test",
        &owner,
        json!({"revision":1,"to":"qa@example.com"}),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let (_, list) = request(
        &state,
        "GET",
        "/v1/admin/mail/deliveries",
        &owner,
        json!({}),
    )
    .await;
    assert_eq!(list["total"], 1);
    assert_eq!(list["items"][0]["status"], "queued");
    assert!(!list.to_string().contains("payload"));
    let id = job["id"].as_str().unwrap();
    let payload: String = sqlx::query_scalar(&format!(
        "SELECT payload FROM {} WHERE id=$1",
        pg.t("mail_outbox")
    ))
    .bind(id)
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert!(!payload.contains("测试邮件"));
    let (a, b) = tokio::join!(pg.claim_mail(false), pg.claim_mail(false));
    let claims = [a.unwrap(), b.unwrap()];
    assert_eq!(claims.iter().filter(|j| j.is_some()).count(), 1);
    let claim = claims.iter().flatten().next().unwrap();
    assert!(!pg.complete_mail(id, "wrong-claim", Ok(())).await.unwrap());
    assert!(pg
        .complete_mail(id, &claim.claim, Err("TLS failed".into()))
        .await
        .unwrap());
    let (_, list) = request(
        &state,
        "GET",
        "/v1/admin/mail/deliveries?status=failed",
        &owner,
        json!({}),
    )
    .await;
    assert_eq!(list["total"], 1);
    let revision = list["items"][0]["revision"].as_i64().unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            &format!("/v1/admin/mail/deliveries/{id}/retry"),
            &owner,
            json!({"revision":revision-1})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &state,
            "POST",
            &format!("/v1/admin/mail/deliveries/{id}/retry"),
            &owner,
            json!({"revision" :revision})
        )
        .await
        .0,
        StatusCode::OK
    );
    let claim = pg.claim_mail(false).await.unwrap().unwrap();
    assert!(pg.complete_mail(id, &claim.claim, Ok(())).await.unwrap());
    let payload: String = sqlx::query_scalar(&format!(
        "SELECT payload FROM {} WHERE id=$1",
        pg.t("mail_outbox")
    ))
    .bind(id)
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert!(payload.is_empty());
    assert_eq!(
        request(&state, "GET", "/v1/admin/mail/config", "invalid", json!({}))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let raw: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} WHERE id=1",
        pg.t("mail_configuration")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert!(!raw.to_string().contains("smtp-password"));
}

#[tokio::test]
async fn mail_leases_expiry_attempt_limits_and_completion_failures_are_safe() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    let mut tx = pg.pool.begin().await.unwrap();
    let a = pg
        .queue_mail(
            &mut tx,
            &state.oauth_config.key,
            0,
            "registration",
            "user@example.com",
            "verify",
            "secret-code",
            "system",
            3600,
        )
        .await
        .unwrap();
    let b = pg
        .queue_mail(
            &mut tx,
            &state.oauth_config.key,
            0,
            "test",
            "qa@example.com",
            "test",
            "test-body",
            "system",
            1,
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(pg.claim_mail(false).await.unwrap().is_none());
    let first = pg.claim_mail(true).await.unwrap().unwrap();
    assert_eq!(first.id, a);
    sqlx::query(&format!(
        "ALTER TABLE {} ADD CONSTRAINT fail_accept CHECK(status<>'accepted') NOT VALID",
        pg.t("mail_attempts")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        pg.complete_mail(&a, &first.claim, Ok(())).await.err(),
        Some(StatusCode::INTERNAL_SERVER_ERROR)
    );
    sqlx::query(&format!(
        "ALTER TABLE {} DROP CONSTRAINT fail_accept",
        pg.t("mail_attempts")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    sqlx::query(&format!(
        "UPDATE {} SET lease_until=now()-interval '1 second' WHERE id=$1",
        pg.t("mail_outbox")
    ))
    .bind(&a)
    .execute(&pg.pool)
    .await
    .unwrap();
    let second = pg.claim_mail(true).await.unwrap().unwrap();
    assert_ne!(first.claim, second.claim);
    assert!(!pg.complete_mail(&a, &first.claim, Ok(())).await.unwrap());
    assert!(pg
        .complete_mail(&a, &second.claim, Err("retry".into()))
        .await
        .unwrap());
    sqlx::query(&format!(
        "UPDATE {} SET next_at=now() WHERE id=$1",
        pg.t("mail_outbox")
    ))
    .bind(&a)
    .execute(&pg.pool)
    .await
    .unwrap();
    let third = pg.claim_mail(true).await.unwrap().unwrap();
    assert!(pg
        .complete_mail(&a, &third.claim, Err("retry".into()))
        .await
        .unwrap());
    sqlx::query(&format!("UPDATE {} SET next_at=now(),expires_at=CASE WHEN id=$1 THEN now()-interval '1 second' ELSE expires_at END",pg.t("mail_outbox"))).bind(&b).execute(&pg.pool).await.unwrap();
    assert!(pg.claim_mail(true).await.unwrap().is_none());
    let row: Value = sqlx::query_scalar(&format!(
        "SELECT to_jsonb(m) FROM {} m WHERE id=$1",
        pg.t("mail_outbox")
    ))
    .bind(b)
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(row["status"], "expired");
    assert_eq!(row["payload"], "");
    // Existing key material must not be regenerated while an encrypted pending message remains.
    assert!(pg.has_mail_secrets().await.unwrap());
}
