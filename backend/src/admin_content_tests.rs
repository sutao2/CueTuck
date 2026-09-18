use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::{json, Value};

#[tokio::test]
async fn cleared_square_can_persistently_disable_demo_seeding() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    assert!(pg.should_seed_square().await.unwrap());
    sqlx::query(&format!("INSERT INTO {} (key,value) VALUES ('square_seed_disabled','true')", pg.t("settings")))
        .execute(&pg.pool).await.unwrap();
    pg.apply_schema(false).await.unwrap();
    let reopened = postgres::Pg::new(pg.pool.clone(), &pg.schema).unwrap();
    assert!(!reopened.should_seed_square().await.unwrap());
    assert!(reopened.list_items().await.unwrap().is_empty());
    sqlx::query(&format!("DELETE FROM {} WHERE key='square_seed_disabled'", pg.t("settings")))
        .execute(&pg.pool).await.unwrap();
    assert!(reopened.should_seed_square().await.unwrap());
    sqlx::query(&format!("INSERT INTO {} (id,title,kind,visibility) VALUES ('old','old','prompt','trashed')", pg.t("square_items")))
        .execute(&pg.pool).await.unwrap();
    assert!(!reopened.should_seed_square().await.unwrap());
    sqlx::query(&format!("DROP SCHEMA {} CASCADE", pg.schema)).execute(&pg.pool).await.unwrap();
}

async fn fixture() -> (AppState, String, String, String) {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    for (email, role) in [
        ("operator@example.com", "admin"),
        ("reviewer@example.com", "reviewer"),
        ("author@example.com", "user"),
    ] {
        pg.upsert_account(email, Some("test-password"), role)
            .await
            .unwrap();
    }
    let admin = state
        .issue_session("operator@example.com".into())
        .await
        .unwrap()
        .access_token;
    let reviewer = state
        .issue_session("reviewer@example.com".into())
        .await
        .unwrap()
        .access_token;
    let author = state
        .issue_session("author@example.com".into())
        .await
        .unwrap()
        .access_token;
    for id in ["a", "b"] {
        state
            .insert_publication(&Publication { cover: None, asset_refs: vec![],
                id: id.into(),
                source_id: id.into(),
                status: "pending".into(),
                title: Some(format!("Title {id}")),
                content: Some("作者原始正文".into()),
                author_email: Some("author@example.com".into()),
                category_id: None,
                model: None,
                kind: "prompt".into(),
                members: vec![],
            })
            .await
            .unwrap();
        pg.review_publication(id, "approved", None, Some(("operator@example.com", &admin)))
            .await
            .unwrap();
    }
    (state, admin, reviewer, author)
}
fn edit(revision: i64, visibility: &str) -> Value {
    json!({"revision":revision,"visibility":visibility,"excerpt":"运营展示摘要","category_id":"cat-image-0","model":"Flux","recommended":true,"sort_index":-10,"reason":"运营调整"})
}
#[tokio::test]
async fn offline_trash_restore_blocks_all_public_paths_and_preserves_favorites() {
    let (state, admin, _, author) = fixture().await;
    state.put_favorite("author@example.com", "a").await.unwrap();
    state.increment_download("a").await.unwrap();
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/content/a",
            &admin,
            edit(0, "offline")
        )
        .await
        .0,
        StatusCode::OK
    );
    for token in ["", &author] {
        for path in ["/v1/square/items/a", "/v1/square/items/a/content"] {
            assert_eq!(
                request(&state, "GET", path, token, json!({})).await.0,
                StatusCode::NOT_FOUND
            );
        }
    }
    assert_eq!(
        request(
            &state,
            "POST",
            "/v1/square/items/a/downloads",
            "",
            json!({})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(&state, "PUT", "/v1/favorites/a", &author, json!({}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert!(state
        .favorite_ids("author@example.com")
        .await
        .unwrap()
        .contains(&"a".to_string()));
    let square = request(
        &state,
        "GET",
        "/v1/square/items?sort=favorites",
        &author,
        json!({}),
    )
    .await
    .1;
    assert_eq!(square["items"], json!([]));
    let mine = request(&state, "GET", "/v1/publications/mine", &author, json!({}))
        .await
        .1;
    assert_eq!(
        mine["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == "a")
            .unwrap()["visibility"],
        "offline"
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/content/a",
            &admin,
            edit(1, "trashed")
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/content/a",
            &admin,
            edit(2, "online")
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/content/a",
            &admin,
            edit(2, "offline")
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/content/a",
            &admin,
            edit(3, "online")
        )
        .await
        .0,
        StatusCode::OK
    );
    let visible = state.get_item("a").await.unwrap().unwrap();
    assert_eq!(visible.content.as_deref(), Some("作者原始正文"));
    assert_eq!(visible.excerpt.as_deref(), Some("运营展示摘要"));
    assert_eq!(state.download_counts().await.unwrap()["a"], 1);
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/square/items?sort=favorites",
            &author,
            json!({})
        )
        .await
        .1["items"][0]["id"],
        "a"
    );
}
#[tokio::test]
async fn content_role_revision_and_audit_are_enforced_without_body_edits() {
    let (state, admin, reviewer, author) = fixture().await;
    for token in [reviewer, author] {
        for path in ["/v1/admin/content", "/v1/admin/content/a"] {
            assert_eq!(
                request(&state, "GET", path, &token, json!({})).await.0,
                StatusCode::FORBIDDEN
            );
        }
        assert_eq!(
            request(
                &state,
                "PUT",
                "/v1/admin/content/a",
                &token,
                edit(0, "offline")
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    let mut body = edit(0, "offline");
    body["content"] = json!("非法改作者正文");
    assert_eq!(
        request(&state, "PUT", "/v1/admin/content/a", &admin, body)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let (first, second) = tokio::join!(
        request(
            &state,
            "PUT",
            "/v1/admin/content/a",
            &admin,
            edit(0, "offline")
        ),
        request(
            &state,
            "PUT",
            "/v1/admin/content/a",
            &admin,
            edit(0, "trashed")
        )
    );
    assert!(matches!(
        (first.0, second.0),
        (StatusCode::OK, StatusCode::CONFLICT) | (StatusCode::CONFLICT, StatusCode::OK)
    ));
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT fail_content CHECK (details->>'item_id' IS DISTINCT FROM 'b')",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    assert_eq!(
        request(
            &state,
            "PUT",
            "/v1/admin/content/b",
            &admin,
            edit(0, "offline")
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    let row = request(&state, "GET", "/v1/admin/content/b", &admin, json!({}))
        .await
        .1;
    assert_eq!(row["visibility"], "online");
    assert_eq!(row["revision"], 0);
    assert_eq!(row["history"], json!([]));
}
#[tokio::test]
async fn content_queries_restart_and_recommendation_do_not_invert_latest() {
    let (state, admin, _, _) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!("UPDATE {} SET listed_at=CASE id WHEN 'a' THEN '2026-01-01'::timestamptz ELSE '2026-02-01'::timestamptz END",pg.t("square_items"))).execute(&pg.pool).await.unwrap();
    request(
        &state,
        "PUT",
        "/v1/admin/content/a",
        &admin,
        edit(0, "online"),
    )
    .await;
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/square/items?sort=recommended",
            "",
            json!({})
        )
        .await
        .1["items"][0]["id"],
        "a"
    );
    assert_eq!(
        request(&state, "GET", "/v1/square/items?sort=latest", "", json!({}))
            .await
            .1["items"][0]["id"],
        "b"
    );
    let list = request(
        &state,
        "GET",
        "/v1/admin/content?limit=1&offset=1",
        &admin,
        json!({}),
    )
    .await
    .1;
    assert_eq!(list["total"], 2);
    assert_eq!(list["items"][0]["id"], "b");
    assert!(list["categories"].as_array().unwrap().len() > 30);
    assert_eq!(
        request(&state, "GET", "/v1/admin/content?q=%25", &admin, json!({}))
            .await
            .1["total"],
        0
    );
    for query in ["visibility=private", "limit=0", "offset=-1"] {
        assert_eq!(
            request(
                &state,
                "GET",
                &format!("/v1/admin/content?{query}"),
                &admin,
                json!({})
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    request(
        &state,
        "PUT",
        "/v1/admin/content/a",
        &admin,
        edit(1, "offline"),
    )
    .await;
    request(
        &state,
        "PUT",
        "/v1/admin/content/b",
        &admin,
        edit(0, "trashed"),
    )
    .await;
    pg.apply_schema(false).await.unwrap();
    let restarted = AppState {
        db: state.db.clone(),
        ..AppState::default()
    };
    assert!(restarted.all_items().await.unwrap().is_empty());
    assert!(pg.has_square_records().await.unwrap());
    assert_eq!(
        request(
            &restarted,
            "GET",
            "/v1/admin/content?visibility=trashed",
            &admin,
            json!({})
        )
        .await
        .1["total"],
        1
    );
}
#[tokio::test]
async fn private_square_allows_authenticated_read_but_never_fails_open() {
    let (state, _, _, author) = fixture().await;
    state.set_square_public(false).await.unwrap();
    assert_eq!(
        request(&state, "GET", "/v1/square/items", "", json!({}))
            .await
            .1["items"],
        json!([])
    );
    assert_eq!(
        request(&state, "GET", "/v1/square/items", &author, json!({}))
            .await
            .1["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    for path in ["/v1/square/items/a", "/v1/square/items/a/content"] {
        assert_eq!(
            request(&state, "GET", path, "", json!({})).await.0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            request(&state, "GET", path, &author, json!({})).await.0,
            StatusCode::OK
        );
    }
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!("DROP TABLE {}", pg.t("settings")))
        .execute(&pg.pool)
        .await
        .unwrap();
    for path in [
        "/v1/square/items",
        "/v1/square/items/a",
        "/v1/square/items/a/content",
    ] {
        assert_eq!(
            request(&state, "GET", path, &author, json!({})).await.0,
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
