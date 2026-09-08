use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::{json, Value};

async fn fixture() -> (AppState, String, String) {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("admin@example.com", Some("test-password"), "admin")
        .await
        .unwrap();
    pg.upsert_account("user@example.com", Some("test-password"), "user")
        .await
        .unwrap();
    let admin = state
        .issue_session("admin@example.com".into())
        .await
        .unwrap()
        .access_token;
    let user = state
        .issue_session("user@example.com".into())
        .await
        .unwrap()
        .access_token;
    (state, admin, user)
}
fn category(id: &str, parent: Option<&str>) -> Value {
    json!({"id":id,"name":id,"parent_id":parent,"icon":"folder","color":"#728080","enabled":true,"sort_index":0})
}
async fn create(state: &AppState, admin: &str, value: Value) -> Value {
    let (status, body) = request(state, "POST", "/v1/admin/catalog/categories", admin, value).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[tokio::test]
async fn migration_preserves_submission_snapshots_and_flattens_history_targets() {
    let (state, admin, user) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    for id in ["a", "b", "c"] {
        create(&state, &admin, category(id, None)).await;
    }
    create(&state, &admin, category("child", Some("a"))).await;
    sqlx::query(&format!("INSERT INTO {} (id,source_id,status,title,content,category_id,kind,members) VALUES ('old','local','pending','Original','Original body','a','collection',$1)",pg.t("publications"))).bind(json!([{"id":"member","title":"Member","content":"Member body","category_id":"a"}])).execute(&pg.pool).await.unwrap();
    let payload = json!({"target":"b","revision":0,"target_revision":0,"reason":"归并分类"});
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/categories/a/migrate",
            &user,
            payload.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/categories/a/migrate",
            &admin,
            payload
        )
        .await
        .1["migrated"],
        true
    );
    let original: String = sqlx::query_scalar(&format!(
        "SELECT category_id FROM {} WHERE id='old'",
        pg.t("publications")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(original, "a");
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/publications/old/approve",
            &admin,
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    let item:Value=sqlx::query_scalar(&format!("SELECT json_build_object('category',category_id,'members',members,'content',content) FROM {} WHERE id='old'",pg.t("square_items"))).fetch_one(&pg.pool).await.unwrap();
    assert_eq!(item["category"], "b");
    assert_eq!(item["members"][0]["category_id"], "b");
    assert_eq!(item["members"][0]["content"], "Member body");
    let result = request(
        &state,
        "POST",
        "/v1/admin/catalog/categories/b/migrate",
        &admin,
        json!({"target":"c","revision":0,"target_revision":0,"reason":"再次归并"}),
    )
    .await;
    assert_eq!(result.1["migrated"], true);
    assert_eq!(result.1["content"], 1);
    let targets: Vec<String> =
        sqlx::query_scalar(&format!("SELECT target FROM {}", pg.t("catalog_redirects")))
            .fetch_all(&pg.pool)
            .await
            .unwrap();
    assert_eq!(targets, vec!["c", "c"]);
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/catalog/categories/c",
            &admin,
            json!({"revision":0})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let child: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} WHERE id='child'",
        pg.t("catalog")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(child["parent_id"], "c");
    assert_eq!(child["revision"], 2);
    // Reopening the same schema must preserve redirects and tombstones.
    let reopened = postgres::Pg::new(pg.pool.clone(), &pg.schema).unwrap();
    reopened.apply_schema(false).await.unwrap();
    assert!(!reopened
        .catalog_entries("categories", false)
        .await
        .unwrap()
        .iter()
        .any(|e| e.id == "a"));
}

#[tokio::test]
async fn migration_rejects_conflicts_and_rolls_back_with_failed_audit() {
    let (state, admin, _) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    create(&state, &admin, category("a", None)).await;
    create(&state, &admin, category("b", None)).await;
    let mut a = category("child-a", Some("a"));
    a["name"] = json!("重复");
    create(&state, &admin, a).await;
    let mut b = category("child-b", Some("b"));
    b["name"] = json!("重复");
    let mut b = create(&state, &admin, b).await;
    let payload = json!({"target":"b","revision":0,"target_revision":0,"reason":"归并"});
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/categories/a/migrate",
            &admin,
            payload.clone()
        )
        .await
        .1["blocked"],
        "children"
    );
    b["name"] = json!("已改名");
    request(
        &state,
        "PUT",
        "/v1/admin/catalog/categories/child-b",
        &admin,
        b,
    )
    .await;
    sqlx::query(&format!(
        "UPDATE {} SET data=jsonb_set(data,'{{skills,0,categories}}','[\"a\"]')",
        pg.t("ai_configuration")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/categories/a/migrate",
            &admin,
            payload.clone()
        )
        .await
        .1["blocked"],
        "skills"
    );
    sqlx::query(&format!(
        "UPDATE {} SET data=jsonb_set(data,'{{skills,0,categories}}','[]')",
        pg.t("ai_configuration")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    let mut stale = payload.clone();
    stale["target_revision"] = json!(1);
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/categories/a/migrate",
            &admin,
            stale
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    sqlx::query(&format!(
        "ALTER TABLE {} ADD CONSTRAINT fail_migration CHECK(action<>'catalog_migrated') NOT VALID",
        pg.t("security_audit")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/categories/a/migrate",
            &admin,
            payload
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    let deleted: bool = sqlx::query_scalar(&format!(
        "SELECT deleted FROM {} WHERE kind='categories' AND id='a'",
        pg.t("catalog")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert!(!deleted);
    let count: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {}",
        pg.t("catalog_redirects")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn model_migration_and_concurrent_approval_never_reintroduce_source() {
    let (state, admin, _) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!("INSERT INTO {} (id,source_id,status,title,content,model) VALUES ('race','local','pending','Race','Body','GPT')",pg.t("publications"))).execute(&pg.pool).await.unwrap();
    let (migration, review) = tokio::join!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/models/GPT/migrate",
            &admin,
            json!({"target":"Claude","revision":0,"target_revision":0,"reason":"模型归并"})
        ),
        request(
            &state,
            "POST",
            "/v1/admin/publications/race/approve",
            &admin,
            json!({})
        )
    );
    assert_eq!(migration.1["migrated"], true);
    assert_eq!(review.0, StatusCode::OK);
    let model: String = sqlx::query_scalar(&format!(
        "SELECT model FROM {} WHERE id='race'",
        pg.t("square_items")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(model, "Claude");
    let original: String = sqlx::query_scalar(&format!(
        "SELECT model FROM {} WHERE id='race'",
        pg.t("publications")
    ))
    .fetch_one(&pg.pool)
    .await
    .unwrap();
    assert_eq!(original, "GPT");
}

#[tokio::test]
async fn catalog_validates_permissions_hierarchy_duplicates_and_revisions() {
    let (state, admin, user) = fixture().await;
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/categories",
            &user,
            category("root", None)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let root = create(&state, &admin, category("root", None)).await;
    create(&state, &admin, category("child", Some("root"))).await;
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/categories",
            &admin,
            category("grandchild", Some("child"))
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let mut duplicate = category("duplicate", None);
    duplicate["name"] = json!("root");
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/categories",
            &admin,
            duplicate
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let mut edit = category("root", None);
    edit["revision"] = root["revision"].clone();
    edit["name"] = json!("New name");
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/catalog/categories/root",
            &admin,
            edit.clone()
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/catalog/categories/root",
            &admin,
            edit
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/catalog/categories/root",
            &admin,
            json!({"revision":1})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    state
        .db
        .as_ref()
        .unwrap()
        .apply_schema(false)
        .await
        .unwrap();
    let rows = request(
        &state,
        "GET",
        "/v1/admin/catalog/categories",
        &admin,
        json!({}),
    )
    .await
    .1;
    assert!(rows["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == "root" && r["name"] == "New name"));
}

#[tokio::test]
async fn catalog_publication_references_filtering_and_disabled_parent() {
    let (state, admin, user) = fixture().await;
    create(&state, &admin, category("root", None)).await;
    create(&state, &admin, category("child", Some("root"))).await;
    let publication =
        json!({"source_id":"local","title":"Prompt","content":"body","category_id":"child"});
    let (code, body) = request(
        &state,
        "POST",
        "/v1/publications",
        &user,
        publication.clone(),
    )
    .await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/catalog/categories/child",
            &admin,
            json!({"revision":0})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let rows = request(
        &state,
        "GET",
        "/v1/admin/catalog/categories",
        &admin,
        json!({}),
    )
    .await
    .1;
    assert!(rows["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == "child" && r["references"] == 1));
    let id = body["id"].as_str().unwrap();
    assert_eq!(
        request(
            &state,
            "POST",
            &format!("/v1/admin/publications/{id}/approve"),
            &admin,
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    let items = request(
        &state,
        "GET",
        "/v1/square/items?category_id=root",
        "",
        json!({}),
    )
    .await
    .1;
    assert_eq!(items["items"].as_array().unwrap().len(), 1);
    let mut edit = category("root", None);
    edit["revision"] = json!(0);
    edit["enabled"] = json!(false);
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/catalog/categories/root",
            &admin,
            edit
        )
        .await
        .0,
        StatusCode::OK
    );
    let public = request(&state, "GET", "/v1/square/catalog", "", json!({}))
        .await
        .1;
    assert!(!public["categories"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == "child" || r["id"] == "root"));
    assert_eq!(
        request(&state, "POST", "/v1/publications", &user, publication)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn catalog_model_crud_deletion_tombstone_and_disabled_model() {
    let (state, admin, user) = fixture().await;
    let model = json!({"id":"test-model","name":"Test model","vendor":"Test","group":"language","enabled":true,"sort_index":3});
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/admin/catalog/models",
            &admin,
            model.clone()
        )
        .await
        .0,
        StatusCode::OK
    );
    let mut edit = model;
    edit["revision"] = json!(0);
    edit["enabled"] = json!(false);
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/catalog/models/test-model",
            &admin,
            edit
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/publications",
            &user,
            json!({"source_id":"local","model":"test-model"})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    create(&state, &admin, category("empty", None)).await;
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/catalog/categories/empty",
            &admin,
            json!({"revision":0})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/catalog/categories/cat-video-0",
            &admin,
            json!({"revision":0})
        )
        .await
        .0,
        StatusCode::OK
    );
    state
        .db
        .as_ref()
        .unwrap()
        .apply_schema(false)
        .await
        .unwrap();
    let rows = request(
        &state,
        "GET",
        "/v1/admin/catalog/categories",
        &admin,
        json!({}),
    )
    .await
    .1;
    assert!(!rows["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == "empty" || r["id"] == "cat-video-0"));
    let audit: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {} WHERE action LIKE 'catalog_%'",
        state.db.as_ref().unwrap().t("security_audit")
    ))
    .fetch_one(&state.db.as_ref().unwrap().pool)
    .await
    .unwrap();
    assert_eq!(audit, 5);
}

#[tokio::test]
async fn catalog_audit_failure_rolls_back_and_concurrent_writers_conflict() {
    let (state, admin, _) = fixture().await;
    create(&state, &admin, category("race", None)).await;
    let mut a = category("race", None);
    a["revision"] = json!(0);
    a["name"] = json!("first");
    let mut b = a.clone();
    b["name"] = json!("second");
    let (left, right) = tokio::join!(
        request(
            &state,
            "PUT",
            "/v1/admin/catalog/categories/race",
            &admin,
            a
        ),
        request(
            &state,
            "PUT",
            "/v1/admin/catalog/categories/race",
            &admin,
            b
        )
    );
    assert!(
        (left.0 == StatusCode::OK && right.0 == StatusCode::CONFLICT)
            || (right.0 == StatusCode::OK && left.0 == StatusCode::CONFLICT)
    );
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!(
        "ALTER TABLE {} ADD CONSTRAINT fail_catalog_audit CHECK (action <> 'catalog_deleted')",
        pg.t("security_audit")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/catalog/categories/race",
            &admin,
            json!({"revision":1})
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    let rows = request(
        &state,
        "GET",
        "/v1/admin/catalog/categories",
        &admin,
        json!({}),
    )
    .await
    .1;
    assert!(rows["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["id"] == "race"));
    pg.upsert_account("admin@example.com", None, "reviewer")
        .await
        .unwrap();
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/admin/catalog/categories",
            &admin,
            json!({})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn catalog_member_references_and_delete_publish_race_never_leave_dangling_ids() {
    let (state, admin, user) = fixture().await;
    create(&state, &admin, category("member", None)).await;
    let body = json!({"source_id":"local","title":"Collection","kind":"collection","members":[{"title":"Member","content":"Body","category_id":"member","model":"Flux"}]});
    assert_eq!(
        request(&state, "POST", "/v1/publications", &user, body)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/catalog/categories/member",
            &admin,
            json!({"revision":0})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &state,
            "DELETE",
            "/v1/admin/catalog/models/Flux",
            &admin,
            json!({"revision":0})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    create(&state, &admin, category("race-ref", None)).await;
    let (deleted, published) = tokio::join!(
        request(
            &state,
            "DELETE",
            "/v1/admin/catalog/categories/race-ref",
            &admin,
            json!({"revision":0})
        ),
        request(
            &state,
            "POST",
            "/v1/publications",
            &user,
            json!({"source_id":"race","category_id":"race-ref"})
        )
    );
    assert!(
        (deleted.0 == StatusCode::OK && published.0 == StatusCode::BAD_REQUEST)
            || (deleted.0 == StatusCode::CONFLICT && published.0 == StatusCode::OK)
    );
}
