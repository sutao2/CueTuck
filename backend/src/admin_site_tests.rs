use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::json;
#[tokio::test]
async fn site_revision_legacy_setting_permissions_validation_and_rollback() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("owner@site.test", Some("test-password"), "owner")
        .await
        .unwrap();
    pg.upsert_account("admin@site.test", Some("test-password"), "admin")
        .await
        .unwrap();
    let owner = state
        .issue_session("owner@site.test".into())
        .await
        .unwrap()
        .access_token;
    let admin = state
        .issue_session("admin@site.test".into())
        .await
        .unwrap()
        .access_token;
    let (_, mut config) = request(&state, "GET", "/v1/admin/site", &owner, json!({})).await;
    assert_eq!(config["revision"], 0);
    assert_eq!(
        request(&state, "PUT", "/v1/admin/site", &admin, config.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    config["name"] = json!("社区站点");
    config["square_public"] = json!(false);
    let (status, saved) = request(&state, "PUT", "/v1/admin/site", &owner, config.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["revision"], 1);
    assert_eq!(
        request(&state, "PUT", "/v1/admin/site", &owner, config)
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/settings",
            &owner,
            json!({"square_public":true})
        )
        .await
        .0,
        StatusCode::OK
    );
    let (_, mut current) = request(&state, "GET", "/v1/admin/site", &owner, json!({})).await;
    assert_eq!(current["revision"], 2);
    assert_eq!(current["name"], "社区站点");
    assert_eq!(current["square_public"], true);
    current["logo_url"] = json!("javascript:alert(1)");
    assert_eq!(
        request(&state, "PUT", "/v1/admin/site", &owner, current.clone())
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    current["logo_url"] = json!("");
    sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT site_audit_fail CHECK(action<>'site_configuration_saved') NOT VALID",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    current["square_public"] = json!(false);
    current["name"] = json!("不应保存");
    assert_eq!(
        request(&state, "PUT", "/v1/admin/site", &owner, current)
            .await
            .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert!(pg.square_public().await.unwrap());
    assert_eq!(pg.site().await.unwrap().name, "社区站点");
}
#[tokio::test]
async fn publishing_gate_and_announcement_schedule_are_consumed_by_public_paths() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("owner@site.test", Some("test-password"), "owner")
        .await
        .unwrap();
    let owner = state
        .issue_session("owner@site.test".into())
        .await
        .unwrap()
        .access_token;
    let (_, mut config) = request(&state, "GET", "/v1/admin/site", &owner, json!({})).await;
    config["publishing_open"] = json!(false);
    config["announcement"] = json!("<script>literal only</script>");
    config["announcement_start"] =
        json!((chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339());
    assert_eq!(
        request(&state, "PUT", "/v1/admin/site", &owner, config.clone())
            .await
            .0,
        StatusCode::OK
    );
    let (_, public) = request(&state, "GET", "/v1/site", "", json!({})).await;
    assert_eq!(public["announcement"], "");
    assert_eq!(public["publishing_open"], false);
    let (_, catalog) = request(&state, "GET", "/v1/square/catalog", "", json!({})).await;
    assert_eq!(catalog["site"], public);
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/publications",
            &owner,
            json!({"source_id":"site-test","title":"Blocked","content":"body"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let count: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {} WHERE source_id='site-test'",
        pg.t("publications")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(count, 0);
    config["revision"] = json!(1);
    config["publishing_open"] = json!(true);
    config["announcement_start"] = json!(null);
    config["announcement_end"] =
        json!((chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339());
    assert_eq!(
        request(&state, "PUT", "/v1/admin/site", &owner, config.clone())
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(&state, "GET", "/v1/site", "", json!({})).await.1["announcement"],
        config["announcement"]
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/publications",
            &owner,
            json!({"source_id":"site-test","title":"Allowed","content":"body"})
        )
        .await
        .0,
        StatusCode::OK
    );
}
