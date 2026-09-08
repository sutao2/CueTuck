use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::{json, Value};
#[tokio::test]
async fn ai_configuration_encryption_versions_permissions_and_history() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("owner@ai.test", Some("test-password"), "owner")
        .await
        .unwrap();
    pg.upsert_account("admin@ai.test", Some("test-password"), "admin")
        .await
        .unwrap();
    let owner = state
        .issue_session("owner@ai.test".into())
        .await
        .unwrap()
        .access_token;
    let admin = state
        .issue_session("admin@ai.test".into())
        .await
        .unwrap()
        .access_token;
    let path = "/v1/admin/ai/config";
    let (_, view) = request(&state, "GET", path, &owner, json!({})).await;
    assert_eq!(view["revision"], 0);
    assert_eq!(view["skills"][0]["id"], "general");
    assert_eq!(
        request(&state, "GET", path, &admin, json!({})).await.0,
        StatusCode::FORBIDDEN
    );
    let mut payload = json!({"revision":0,"current_password":"wrong","models":[{"id":"first","name":"Test","endpoint":"https://api.example.com/v1/chat/completions","model":"configured-model","enabled":true,"json_mode":true,"redact":true,"timeout_seconds":5}],"secrets":{"first":"super-secret-key"},"skills":view["skills"]});
    assert_eq!(
        request(&state, "PUT", path, &owner, payload.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    payload["current_password"] = json!("test-password");
    let (status, saved) = request(&state, "PUT", path, &owner, payload.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["revision"], 1);
    assert_eq!(saved["models"][0]["secret_configured"], true);
    assert_eq!(saved["models"][0]["vision"], false, "old model configurations must not opt into image transmission");
    assert!(!saved.to_string().contains("super-secret"));
    assert!(pg.has_ai_secrets().await.unwrap());
    let raw: serde_json::Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} WHERE revision=1",
        pg.t("ai_configuration")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert!(!raw.to_string().contains("super-secret"));
    let encrypted = raw["models"][0]["encrypted_secret"].as_str().unwrap();
    assert_eq!(
        crate::oauth_admin::unseal(&state.oauth_config.key, "ai:first", encrypted).unwrap(),
        "super-secret-key"
    );
    assert!(crate::oauth_admin::unseal(&state.oauth_config.key, "ai:other", encrypted).is_err());
    assert_eq!(
        request(&state, "PUT", path, &owner, payload.clone())
            .await
            .0,
        StatusCode::CONFLICT
    );
    payload["revision"] = json!(1);
    payload["secrets"] = json!({});
    payload["skills"][0]["name"] = json!("Revised");
    assert_eq!(
        request(&state, "PUT", path, &owner, payload.clone())
            .await
            .0,
        StatusCode::OK
    );
    let (_, history) = request(&state, "GET", "/v1/admin/ai/history", &owner, json!({})).await;
    assert_eq!(history["items"].as_array().unwrap().len(), 3);
    assert!(!history.to_string().contains("encrypted_secret"));
    assert_eq!(history["items"][0]["data"]["skills"][0]["name"], "Revised");
    assert_eq!(
        history["items"][1]["data"]["skills"][0]["name"],
        "通用内容安全"
    );
    payload["revision"] = json!(2);
    payload["models"][0]["endpoint"] = json!("https://127.0.0.1/secret");
    assert_eq!(
        request(&state, "PUT", path, &owner, payload).await.0,
        StatusCode::BAD_REQUEST
    );
    let (status, result) = request(
        &state,
        "POST",
        "/v1/admin/ai/test",
        &owner,
        json!({"revision":2,"skill_id":"general","text":"sample"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(result["verdict"].is_null());
    assert!(result["error"].as_str().unwrap().contains("数量不足"));
    let (_, view) = request(&state, "GET", path, &owner, json!({})).await;
    assert_eq!(view["tests"][0]["success"], false);
    let audit: String = sqlx::query_scalar(&format!(
        "SELECT string_agg(details::text,' ') FROM {}",
        pg.t("security_audit")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert!(!audit.contains("super-secret"));
    assert!(!audit.contains("test-password"));
}
#[test]
fn routing_never_approves_missing_conflicting_or_duplicate_results() {
    use crate::{admin_ai::combine, ai_transport::Verdict};
    let approve = Verdict {
        risk_score: 10,
        decision: "approve".into(),
        reasons: vec![],
        matched_rules: vec![],
    };
    let reject = Verdict {
        risk_score: 90,
        decision: "reject".into(),
        ..approve.clone()
    };
    assert!(combine("consensus", &[approve.clone()]).is_none());
    assert!(combine("consensus", &[approve.clone(), reject.clone()]).is_none());
    let result = combine("majority", &[approve.clone(), approve.clone(), reject]).unwrap();
    assert_eq!(result.decision, "approve");
    assert_eq!(result.risk_score, 90);
    assert_eq!(combine("fallback", &[approve]).unwrap().decision, "approve");
    assert_eq!(
        crate::ai_transport::redact("mail@example.com sk-secret ok\n"),
        "[已脱敏] [已脱敏] ok\n"
    );
}

#[tokio::test]
async fn ai_finalization_is_atomic_and_never_overwrites_humans_or_changed_policy() {
    use crate::{admin_ai::Run, ai_transport::Verdict};
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("writer@ai.test", Some("test-password"), "user")
        .await
        .unwrap();
    let token = state
        .issue_session("writer@ai.test".into())
        .await
        .unwrap()
        .access_token;
    let policy = crate::admin_moderation::Policy {
        enabled: true,
        auto_approve: true,
        require_ai: true,
        ..Default::default()
    };
    sqlx::query(&format!(
        "UPDATE {} SET data=$1 WHERE id=1",
        pg.t("moderation_policy")
    ))
    .bind(json!(policy))
    .execute(&pg.pool)
    .await
    .unwrap();
    let runs = vec![Run {
        revision: 0,
        skill_id: "general".into(),
        verdict: Some(Verdict {
            risk_score: 5,
            decision: "approve".into(),
            reasons: vec![],
            matched_rules: vec![],
        }),
        models: vec![],
        error: None,
    }];
    for scenario in [
        "approve",
        "human",
        "migration",
        "policy",
        "missing",
        "audit",
    ] {
        let publication = Publication { asset_refs: vec![],
            id: format!("pub.{scenario}"),
            source_id: scenario.into(),
            status: "pending".into(),
            title: Some(scenario.into()),
            content: Some(format!("Unique {scenario}")),
            author_email: Some("writer@ai.test".into()),
            category_id: None,
            model: (scenario == "migration").then(|| "GPT".into()),
            kind: "prompt".into(),
            members: vec![],
        };
        pg.moderate_publication(&publication, &token).await.unwrap();
        let local: Value = sqlx::query_scalar(&format!(
            "SELECT moderation FROM {} WHERE id=$1",
            pg.t("publications")
        ))
        .bind(&publication.id)
        .fetch_one(&pg.pool)
        .await
        .unwrap();
        if scenario == "human" {
            pg.review_publication(&publication.id, "rejected", Some("Human decision"), None)
                .await
                .unwrap();
        }
        if scenario == "migration" {
            sqlx::query(&format!(
                "INSERT INTO {} (kind,source,target) VALUES ('models','GPT','Claude')",
                pg.t("catalog_redirects")
            ))
            .execute(&pg.pool)
            .await
            .unwrap();
        }
        if scenario == "policy" {
            sqlx::query(&format!(
                "UPDATE {} SET data=jsonb_set(data,'{{revision}}','1') WHERE id=1",
                pg.t("moderation_policy")
            ))
            .execute(&pg.pool)
            .await
            .unwrap();
        }
        if scenario == "audit" {
            sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT fail_ai CHECK(action<>'publication_ai_screened') NOT VALID",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
        }
        let final_result = pg
            .finish_ai(
                &publication,
                &local,
                0,
                if scenario == "missing" { &[] } else { &runs },
                None,
            )
            .await;
        if scenario == "audit" {
            assert_eq!(final_result.err(), Some(StatusCode::INTERNAL_SERVER_ERROR));
        } else {
            assert_eq!(
                final_result.unwrap().status,
                match scenario {
                    "approve" => "approved",
                    "human" => "rejected",
                    _ => "pending",
                }
            );
        }
        let listed: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE id=$1)",
            pg.t("square_items")
        ))
        .bind(&publication.id)
        .fetch_one(&pg.pool)
        .await
        .unwrap();
        assert_eq!(listed, scenario == "approve");
        if scenario == "policy" {
            sqlx::query(&format!(
                "UPDATE {} SET data=$1 WHERE id=1",
                pg.t("moderation_policy")
            ))
            .bind(json!(policy))
            .execute(&pg.pool)
            .await
            .unwrap();
        }
    }
}
