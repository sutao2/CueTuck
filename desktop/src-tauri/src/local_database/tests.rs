use super::{
    add_prompt_to_collection_in_dir, backup_library_in_dir, collection_member_count,
    count_local_prompts_in_dir, create_category_in_dir, create_collection_in_dir,
    create_prompt_in_dir, create_prompt_in_dir_with_model, delete_prompt_in_dir,
    export_library_zip_in_dir, get_setting_in_dir,
    import_downloaded_prompt_in_dir,
    initialize_in_dir,
    list_categories_in_dir, list_collection_members_in_dir, list_collections_in_dir,
    list_prompts_in_dir,
    list_system_category_names, preview_import_json_in_dir, prompt_deleted_at, prompt_use_count,
    clear_prompt_use_in_dir, record_prompt_use_in_dir, restore_library_in_dir, set_setting_in_dir,
    update_prompt_in_dir, update_prompt_in_dir_with_model, upsert_synced_prompt_in_dir,
};

#[test]
fn custom_root_categories_round_trip_and_delete_safely() {
    use super::{apply_sync_changes, export_sync_changes, export_library_json_in_dir, apply_import_json_in_dir, delete_category_in_dir};
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    initialize_in_dir(a.path()).unwrap();
    initialize_in_dir(b.path()).unwrap();
    let root = create_category_in_dir(a.path(), " 我的项目 ", None).unwrap();
    assert!(root.parent_id.is_none());
    assert!(!root.is_system);
    assert!(create_category_in_dir(a.path(), "我的项目", None).unwrap_err().contains("同名"));
    assert!(create_category_in_dir(a.path(), "软件开发", None).unwrap_err().contains("同名"));
    let child = create_category_in_dir(a.path(), "发布", Some(&root.id)).unwrap();
    assert!(create_category_in_dir(a.path(), "三级", Some(&child.id)).is_err());
    let collection = create_collection_in_dir(a.path(), "合集", Some(&root.id), "none", None).unwrap();
    let prompt = create_prompt_in_dir(a.path(), "成员", "保留", Some(&child.id)).unwrap();
    add_prompt_to_collection_in_dir(a.path(), &prompt.id, &collection.id).unwrap();
    create_prompt_in_dir(a.path(), "根内容", "保留", Some(&root.id)).unwrap();
    assert_eq!(list_prompts_in_dir(a.path(), "", Some(&root.id)).unwrap().len(), 2);
    let mut snapshot = export_sync_changes(a.path()).unwrap();
    snapshot.reverse();
    apply_sync_changes(b.path(), &snapshot, false).unwrap();
    assert_eq!(list_categories_in_dir(b.path()).unwrap().iter().find(|c| c.id == child.id).unwrap().parent_id.as_deref(), Some(root.id.as_str()));
    let mut file: serde_json::Value = serde_json::from_str(&export_library_json_in_dir(a.path()).unwrap()).unwrap();
    file["categories"].as_array_mut().unwrap().reverse();
    apply_import_json_in_dir(b.path(), &file.to_string()).unwrap();
    let categories = list_categories_in_dir(b.path()).unwrap();
    let copied_child = categories.iter().find(|c| c.name == "发布" && c.id != child.id).unwrap();
    let copied_root = categories.iter().find(|c| Some(&c.id) == copied_child.parent_id.as_ref()).unwrap();
    assert_ne!(copied_root.id, root.id);
    assert_eq!(copied_root.name, "我的项目");
    assert!(copied_root.parent_id.is_none());
    let copies = list_prompts_in_dir(b.path(), "", Some(&copied_root.id)).unwrap();
    assert_eq!(copies.len(), 2);
    let copied_member = copies.iter().find(|p| p.title == "成员").unwrap();
    let copied_collection = list_collections_in_dir(b.path(), "", Some(&copied_root.id)).unwrap();
    assert_eq!(copied_member.collection_id.as_deref(), Some(copied_collection[0].id.as_str()));
    assert!(delete_category_in_dir(a.path(), &root.id).unwrap_err().contains("先删除"));
    delete_category_in_dir(a.path(), &child.id).unwrap();
    delete_category_in_dir(a.path(), &root.id).unwrap();
    assert!(list_prompts_in_dir(a.path(), "", None).unwrap().iter().all(|p| p.category_id.is_none()));
    assert!(create_category_in_dir(a.path(), "失效父级", Some(&root.id)).is_err());
    let mut deleted = export_sync_changes(a.path()).unwrap();
    deleted.reverse();
    apply_sync_changes(b.path(), &deleted, false).unwrap();
    apply_sync_changes(b.path(), &snapshot, false).unwrap();
    assert!(!list_categories_in_dir(b.path()).unwrap().iter().any(|c| c.id == root.id || c.id == child.id));
}

#[test]
fn custom_root_categories_reject_invalid_imports_and_sync_atomically() {
    use super::{apply_sync_changes, SyncChange, export_library_json_in_dir, apply_import_json_in_dir};
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let root = create_category_in_dir(dir.path(), "项目", None).unwrap();
    let child = create_category_in_dir(dir.path(), "发布", Some(&root.id)).unwrap();
    let before = export_library_json_in_dir(dir.path()).unwrap();
    let change = SyncChange { id: root.id.clone(), kind: "category".into(), payload: serde_json::json!({ "name": "项目", "parent_id": "cat-office" }), updated_at: super::now_millis(), deleted_at: None };
    assert!(apply_sync_changes(dir.path(), &[change], false).unwrap_err().contains("层级"));
    assert_eq!(export_library_json_in_dir(dir.path()).unwrap(), before);
    let mut file: serde_json::Value = serde_json::from_str(&before).unwrap();
    file["categories"].as_array_mut().unwrap().push(serde_json::json!({ "id": "third", "name": "三级", "parent_id": child.id }));
    assert!(preview_import_json_in_dir(dir.path(), &file.to_string()).is_err());
    assert!(apply_import_json_in_dir(dir.path(), &file.to_string()).is_err());
    assert_eq!(export_library_json_in_dir(dir.path()).unwrap(), before);
}

#[test]
fn category_delete_preserves_content_and_syncs_without_resurrection() {
    use super::{delete_category_in_dir, apply_sync_changes, export_sync_changes, export_library_json_in_dir};
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    initialize_in_dir(a.path()).unwrap();
    initialize_in_dir(b.path()).unwrap();
    let category = create_category_in_dir(a.path(), "周报", Some("cat-office")).unwrap();
    assert!(create_category_in_dir(a.path(), " 周报 ", Some("cat-office")).unwrap_err().contains("同名"));
    assert!(create_category_in_dir(a.path(), "周报", Some("cat-image")).is_ok());
    let collection = create_collection_in_dir(a.path(), "合集", Some(&category.id), "none", None).unwrap();
    let prompt = create_prompt_in_dir(a.path(), "正文", "保留", Some(&category.id)).unwrap();
    add_prompt_to_collection_in_dir(a.path(), &prompt.id, &collection.id).unwrap();
    let before = export_sync_changes(a.path()).unwrap();
    apply_sync_changes(b.path(), &before, false).unwrap();
    assert!(delete_category_in_dir(a.path(), "cat-office").unwrap_err().contains("系统"));
    assert!(delete_category_in_dir(a.path(), "cat-software-0").unwrap_err().contains("系统"));
    assert!(delete_category_in_dir(a.path(), "missing").is_err());
    delete_category_in_dir(a.path(), &category.id).unwrap();
    initialize_in_dir(a.path()).unwrap();
    assert!(!list_categories_in_dir(a.path()).unwrap().iter().any(|c| c.id == category.id));
    assert!(list_prompts_in_dir(a.path(), "", None).unwrap()[0].category_id.is_none());
    assert!(list_collections_in_dir(a.path(), "", None).unwrap()[0].category_id.is_none());
    assert_eq!(list_collection_members_in_dir(a.path(), &collection.id).unwrap()[0].content, "保留");
    let exported: serde_json::Value = serde_json::from_str(&export_library_json_in_dir(a.path()).unwrap()).unwrap();
    assert!(!exported["categories"].as_array().unwrap().iter().any(|c| c["id"] == category.id));
    let tombstones: Vec<_> = export_sync_changes(a.path()).unwrap().into_iter().filter(|c| c.kind == "category" && c.deleted_at.is_some()).collect();
    assert_eq!(tombstones.len(), 1);
    apply_sync_changes(b.path(), &tombstones, false).unwrap();
    apply_sync_changes(b.path(), &before, false).unwrap();
    assert!(!list_categories_in_dir(b.path()).unwrap().iter().any(|c| c.id == category.id));
    assert!(list_prompts_in_dir(b.path(), "", None).unwrap()[0].category_id.is_none());
    assert!(list_collections_in_dir(b.path(), "", None).unwrap()[0].category_id.is_none());
    assert!(create_category_in_dir(a.path(), "周报", Some("cat-office")).is_ok());
}

#[test]
fn category_delete_rolls_back_when_content_update_fails() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let category = create_category_in_dir(dir.path(), "周报", Some("cat-office")).unwrap();
    create_prompt_in_dir(dir.path(), "正文", "保留", Some(&category.id)).unwrap();
    create_collection_in_dir(dir.path(), "合集", Some(&category.id), "none", None).unwrap();
    let connection = rusqlite::Connection::open(dir.path().join("promptark.sqlite")).unwrap();
    connection.execute_batch("CREATE TRIGGER reject_category_change BEFORE UPDATE ON collections BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
    assert!(super::delete_category_in_dir(dir.path(), &category.id).is_err());
    assert!(list_categories_in_dir(dir.path()).unwrap().iter().any(|c| c.id == category.id));
    assert_eq!(list_prompts_in_dir(dir.path(), "", None).unwrap()[0].category_id.as_deref(), Some(category.id.as_str()));
}

#[test]
fn sync_round_trip_restores_categories_collections_members_models_and_deletions() {
    use super::{apply_sync_changes, export_sync_changes};
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    initialize_in_dir(a.path()).unwrap();
    initialize_in_dir(b.path()).unwrap();
    let category = create_category_in_dir(a.path(), "我的图片", Some("cat-image")).unwrap();
    let collection = create_collection_in_dir(a.path(), "灵感", Some(&category.id), "single", Some("[\"cover.png\"]")).unwrap();
    let prompt = create_prompt_in_dir_with_model(a.path(), "人像", "正文", Some(&category.id), Some("Flux")).unwrap();
    add_prompt_to_collection_in_dir(a.path(), &prompt.id, &collection.id).unwrap();
    set_setting_in_dir(a.path(), "theme", "dark").unwrap();
    set_setting_in_dir(a.path(), "manual_proxy", "private-proxy").unwrap();
    let snapshot = export_sync_changes(a.path()).unwrap();
    assert!(!snapshot.iter().any(|row| row.payload["value_json"] == "private-proxy"));
    assert!(snapshot.iter().filter(|row| row.kind == "prompt").all(|row| row.updated_at.len() == 13));
    // Deliberately reverse the dependencies to test transactional ordering.
    apply_sync_changes(b.path(), &snapshot.into_iter().rev().collect::<Vec<_>>(), false).unwrap();
    assert!(list_categories_in_dir(b.path()).unwrap().iter().any(|row| row.id == category.id));
    let members = list_collection_members_in_dir(b.path(), &collection.id).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].model.as_deref(), Some("Flux"));
    assert_eq!(get_setting_in_dir(b.path(), "theme").unwrap(), "dark");
    assert_eq!(list_collections_in_dir(b.path(), "", Some("cat-image")).unwrap()[0].cover_json, "[\"cover.png\"]");
    delete_prompt_in_dir(a.path(), &prompt.id).unwrap();
    apply_sync_changes(b.path(), &export_sync_changes(a.path()).unwrap(), false).unwrap();
    assert!(list_prompts_in_dir(b.path(), "", None).unwrap().is_empty());
    assert_eq!(collection_member_count(b.path(), &collection.id).unwrap(), 0);
    // A stale snapshot must not resurrect the deleted member.
    let mut stale = export_sync_changes(a.path()).unwrap();
    let row = stale.iter_mut().find(|row| row.id == prompt.id).unwrap();
    row.updated_at = "1".into();
    row.deleted_at = None;
    apply_sync_changes(b.path(), &stale, false).unwrap();
    assert!(list_prompts_in_dir(b.path(), "", None).unwrap().is_empty());
}

#[test]
fn sync_rolls_back_bad_references_and_normalizes_legacy_seconds() {
    use super::{apply_sync_changes, SyncChange};
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let change = |id: &str, stamp: &str, payload| SyncChange { id: id.into(), kind: "prompt".into(), payload, updated_at: stamp.into(), deleted_at: None };
    let valid = change("p", "1700000000000", json!({"title":"旧版", "content":"旧"}));
    let invalid = change("bad", "2", json!({"title":"坏引用", "category_id":"missing"}));
    assert!(apply_sync_changes(dir.path(), &[valid.clone(), invalid], false).is_err());
    assert!(list_prompts_in_dir(dir.path(), "", None).unwrap().is_empty());
    apply_sync_changes(dir.path(), &[valid], false).unwrap();
    let newer = change("p", "1700000001", json!({"title":"新版", "content":"新"}));
    apply_sync_changes(dir.path(), &[newer.clone()], true).unwrap();
    assert_eq!(list_prompts_in_dir(dir.path(), "", None).unwrap()[0].content, "旧");
    apply_sync_changes(dir.path(), &[newer], false).unwrap();
    assert_eq!(list_prompts_in_dir(dir.path(), "", None).unwrap()[0].content, "新");
    let tombstone = SyncChange { deleted_at: Some("1700000002000".into()), ..change("p", "1700000002000", json!({})) };
    apply_sync_changes(dir.path(), &[tombstone], false).unwrap();
    apply_sync_changes(dir.path(), &[change("p", "1700000003000", json!({"title":"不要复活"}))], true).unwrap();
    assert!(list_prompts_in_dir(dir.path(), "", None).unwrap().is_empty());
}

#[test]
fn collections_can_be_edited_moved_and_deleted_without_deleting_prompts() {
    use super::{delete_collection_in_dir, remove_prompt_from_collection_in_dir, update_collection_in_dir};
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let a = create_collection_in_dir(dir.path(), "A", None, "none", None).unwrap();
    let b = create_collection_in_dir(dir.path(), "B", None, "none", None).unwrap();
    let p = create_prompt_in_dir(dir.path(), "成员", "保留正文", None).unwrap();
    assert!(add_prompt_to_collection_in_dir(dir.path(), &p.id, "missing").is_err());
    add_prompt_to_collection_in_dir(dir.path(), &p.id, &a.id).unwrap();
    add_prompt_to_collection_in_dir(dir.path(), &p.id, &b.id).unwrap();
    assert_eq!(collection_member_count(dir.path(), &a.id).unwrap(), 0);
    assert_eq!(collection_member_count(dir.path(), &b.id).unwrap(), 1);
    remove_prompt_from_collection_in_dir(dir.path(), &p.id, &b.id).unwrap();
    assert_eq!(list_prompts_in_dir(dir.path(), "", None).unwrap()[0].collection_id, None);
    update_collection_in_dir(dir.path(), &b.id, "新名称", Some("cat-image"), "single", "[\"cover.png\"]").unwrap();
    let updated = list_collections_in_dir(dir.path(), "新名称", None).unwrap();
    assert_eq!(updated[0].cover_json, "[\"cover.png\"]");
    add_prompt_to_collection_in_dir(dir.path(), &p.id, &b.id).unwrap();
    delete_collection_in_dir(dir.path(), &b.id).unwrap();
    assert!(list_collections_in_dir(dir.path(), "新名称", None).unwrap().is_empty());
    let rows = list_prompts_in_dir(dir.path(), "", None).unwrap();
    assert_eq!(rows[0].content, "保留正文");
    assert_eq!(rows[0].collection_id, None);
    assert!(add_prompt_to_collection_in_dir(dir.path(), &p.id, &b.id).is_err());
}

#[tokio::test]
async fn status_is_ready_after_initialize() {
    let dir = tempfile::tempdir().unwrap();
    let status = initialize_in_dir(dir.path()).unwrap();
    assert_eq!(status, "ready");
}

#[test]
fn json_transfer_preserves_structure_as_new_copies_and_rejects_partial_imports() {
    use super::{apply_import_json_in_dir, export_library_json_in_dir};
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let category = create_category_in_dir(dir.path(), "自定义图片", Some("cat-image")).unwrap();
    let collection = create_collection_in_dir(dir.path(), "合集", Some(&category.id), "single", Some("[\"one.png\"]")).unwrap();
    let prompt = create_prompt_in_dir_with_model(dir.path(), "成员", "正文", Some(&category.id), Some("Flux")).unwrap();
    add_prompt_to_collection_in_dir(dir.path(), &prompt.id, &collection.id).unwrap();
    let exported = export_library_json_in_dir(dir.path()).unwrap();
    apply_import_json_in_dir(dir.path(), &exported).unwrap();
    let prompts = list_prompts_in_dir(dir.path(), "", None).unwrap();
    assert_eq!(prompts.len(), 2);
    let copy = prompts.iter().find(|row| row.id != prompt.id).unwrap();
    assert_eq!(copy.model.as_deref(), Some("Flux"));
    assert_ne!(copy.category_id.as_deref(), Some(category.id.as_str()));
    assert_ne!(copy.collection_id.as_deref(), Some(collection.id.as_str()));
    let collections = list_collections_in_dir(dir.path(), "", None).unwrap();
    assert_eq!(collections.iter().find(|row| Some(&row.id) == copy.collection_id.as_ref()).unwrap().cover_json, "[\"one.png\"]");
    for raw in [r#"{"prompts":[{"title":"不能部分写入"},{"title":""}]}"#,
        r#"{"prompts":[{"title":"坏引用","category_id":"missing"}]}"#,
        r#"{"prompts":[{"title":"坏字段","use_count":"not-a-number"}]}"#,
        r#"{"prompts":null}"#,
        r#"{"prompts":[{"id":"x","title":"一"},{"id":"x","title":"二"}]}"#] {
        assert!(preview_import_json_in_dir(dir.path(), raw).is_err());
        assert!(apply_import_json_in_dir(dir.path(), raw).is_err());
        assert_eq!(list_prompts_in_dir(dir.path(), "", None).unwrap().len(), 2);
    }
    apply_import_json_in_dir(dir.path(), r#"{"prompts":[{"title":"旧格式","content":"旧正文"}]}"#).unwrap();
    assert_eq!(list_prompts_in_dir(dir.path(), "旧格式", None).unwrap()[0].content, "旧正文");
}

#[tokio::test]
async fn empty_library_counts_zero() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    assert_eq!(count_local_prompts_in_dir(dir.path()).unwrap(), 0);
}

#[test]
fn uncategorized_filters_prompts_and_collections() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    create_prompt_in_dir(dir.path(), "待分类", "正文", None).unwrap();
    create_prompt_in_dir(dir.path(), "人像", "正文", Some("cat-image-0")).unwrap();
    create_collection_in_dir(dir.path(), "待分类合集", None, "none", None).unwrap();
    create_collection_in_dir(dir.path(), "图片合集", Some("cat-image"), "none", None).unwrap();
    let prompts = list_prompts_in_dir(dir.path(), "", Some("__uncategorized__")).unwrap();
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].title, "待分类");
    let collections = list_collections_in_dir(dir.path(), "", Some("__uncategorized__")).unwrap();
    assert_eq!(collections.len(), 1);
    assert_eq!(collections[0].title, "待分类合集");
}

#[tokio::test]
async fn seeds_ten_system_categories() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let names = list_system_category_names(dir.path()).unwrap();
    assert_eq!(
        names,
        vec![
            "软件开发",
            "图片生成",
            "视频创作",
            "办公效率",
            "内容写作",
            "产品设计",
            "市场营销",
            "数据分析",
            "教育学习",
            "生活助手",
        ]
    );
}

#[tokio::test]
async fn creates_prompt_and_lists_it() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    create_prompt_in_dir(dir.path(), "测试", "正文", None).unwrap();
    let rows = list_prompts_in_dir(dir.path(), "测试", None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "测试");
}

#[tokio::test]
async fn imports_downloaded_prompt_with_source() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let created = import_downloaded_prompt_in_dir(dir.path(), "自然光群像", "正文", Some("sq-1"), None).unwrap();
    assert_eq!(created.source, "downloaded");
    let rows = list_prompts_in_dir(dir.path(), "自然光群像", None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, "downloaded");
}

#[test]
fn download_preserves_category_and_model() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    super::import_downloaded_prompt_with_metadata(dir.path(), "人像", "正文", Some("sq-p"), None,
        Some("cat-image-0"), Some("Flux")).unwrap();
    let rows = list_prompts_in_dir(dir.path(), "", Some("cat-image")).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].model.as_deref(), Some("Flux"));
}

#[tokio::test]
async fn keeps_author_on_downloaded_prompt_without_rewriting_content() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let created = import_downloaded_prompt_in_dir(
        dir.path(),
        "自然光群像",
        "清透蓝天下的多元人物群像。",
        Some("sq-1"),
        Some("林晚"),
    )
    .unwrap();
    assert_eq!(created.author.as_deref(), Some("林晚"));
    assert_eq!(created.content, "清透蓝天下的多元人物群像。");
    let skipped = import_downloaded_prompt_in_dir(
        dir.path(),
        "夜景街拍",
        "潮湿路面的霓虹倒影。",
        Some("sq-2"),
        None,
    )
    .unwrap();
    assert_eq!(skipped.author, None);
    assert_eq!(skipped.content, "潮湿路面的霓虹倒影。");
}

#[tokio::test]
async fn keeps_existing_local_prompt_when_upserting_a_synced_id() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    upsert_synced_prompt_in_dir(dir.path(), "p-keep", "本地仍在", "正文", None, "2").unwrap();
    let kept = upsert_synced_prompt_in_dir(
        dir.path(),
        "p-keep",
        "远端标题",
        "远端正文",
        None,
        "1",
    )
    .unwrap();
    assert_eq!(kept.title, "本地仍在");
    assert_eq!(kept.content, "正文");
    upsert_synced_prompt_in_dir(dir.path(), "remote-1", "远端新增", "拉下来", None, "2").unwrap();
    let rows = list_prompts_in_dir(dir.path(), "", None).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|row| row.title == "远端新增"));
}

#[tokio::test]
async fn newer_remote_body_replaces_older_local_prompt() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    upsert_synced_prompt_in_dir(dir.path(), "p-1", "本地仍在", "本机正文", None, "1").unwrap();
    let updated = upsert_synced_prompt_in_dir(
        dir.path(),
        "p-1",
        "本地仍在",
        "远端正文",
        None,
        "2",
    )
    .unwrap();
    assert_eq!(updated.content, "远端正文");
}

#[tokio::test]
async fn search_hits_content() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    create_prompt_in_dir(dir.path(), "报表助手", "用 Power Query 清洗", None).unwrap();
    let rows = list_prompts_in_dir(dir.path(), "Power Query", None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "报表助手");
}

#[tokio::test]
async fn soft_deleted_prompt_is_hidden() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let id = create_prompt_in_dir(dir.path(), "过期模板", "x", None)
        .unwrap()
        .id;
    delete_prompt_in_dir(dir.path(), &id).unwrap();
    assert!(list_prompts_in_dir(dir.path(), "过期", None)
        .unwrap()
        .is_empty());
    assert!(prompt_deleted_at(dir.path(), &id).unwrap().is_some());
}

#[tokio::test]
async fn update_prompt_rewrites_content() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let created = create_prompt_in_dir(dir.path(), "问候", "你好", None).unwrap();
    update_prompt_in_dir(dir.path(), &created.id, "问候", "你好 {{姓名}}", None).unwrap();
    let rows = list_prompts_in_dir(dir.path(), "", None).unwrap();
    assert_eq!(rows[0].content, "你好 {{姓名}}");
}

#[tokio::test]
async fn lists_children_under_software() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let categories = list_categories_in_dir(dir.path()).unwrap();
    let software = categories
        .iter()
        .find(|category| category.name == "软件开发")
        .unwrap();
    let children: Vec<&str> = categories
        .iter()
        .filter(|category| category.parent_id.as_deref() == Some(software.id.as_str()))
        .map(|category| category.name.as_str())
        .collect();
    assert!(children.contains(&"网站开发"));
    assert!(children.contains(&"前端工程"));
}

#[tokio::test]
async fn creates_user_child_under_office() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let created = create_category_in_dir(dir.path(), "周报", Some("cat-office")).unwrap();
    assert_eq!(created.name, "周报");
    assert_eq!(created.parent_id.as_deref(), Some("cat-office"));
    assert!(!created.is_system);
    let names: Vec<_> = list_categories_in_dir(dir.path())
        .unwrap()
        .into_iter()
        .filter(|row| row.parent_id.as_deref() == Some("cat-office"))
        .map(|row| row.name)
        .collect();
    assert!(names.contains(&"周报".to_string()));
}

#[tokio::test]
async fn rejects_grandchild_under_frontend() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let error = create_category_in_dir(dir.path(), "再下一层", Some("cat-software-1")).unwrap_err();
    assert!(error.contains("小分类下不能再创建子分类"));
    assert!(!list_categories_in_dir(dir.path())
        .unwrap()
        .iter()
        .any(|row| row.name == "再下一层"));
}

#[tokio::test]
async fn selecting_parent_lists_child_prompts() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let categories = list_categories_in_dir(dir.path()).unwrap();
    let image = categories
        .iter()
        .find(|category| category.name == "图片生成")
        .unwrap();
    let portrait = categories
        .iter()
        .find(|category| category.name == "人像摄影")
        .unwrap();
    let product = categories
        .iter()
        .find(|category| category.name == "商品视觉")
        .unwrap();
    let web = categories
        .iter()
        .find(|category| category.name == "网站开发")
        .unwrap();
    create_prompt_in_dir(dir.path(), "人像", "a", Some(&portrait.id)).unwrap();
    create_prompt_in_dir(dir.path(), "商品", "b", Some(&product.id)).unwrap();
    create_prompt_in_dir(dir.path(), "官网", "c", Some(&web.id)).unwrap();
    let rows = list_prompts_in_dir(dir.path(), "", Some(&image.id)).unwrap();
    let titles: Vec<_> = rows.iter().map(|row| row.title.as_str()).collect();
    assert_eq!(titles.len(), 2);
    assert!(titles.contains(&"人像"));
    assert!(titles.contains(&"商品"));
}

#[tokio::test]
async fn recording_use_increments_count() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let created = create_prompt_in_dir(dir.path(), "计数", "正文", None).unwrap();
    record_prompt_use_in_dir(dir.path(), &created.id).unwrap();
    record_prompt_use_in_dir(dir.path(), &created.id).unwrap();
    record_prompt_use_in_dir(dir.path(), &created.id).unwrap();
    let used = record_prompt_use_in_dir(dir.path(), &created.id).unwrap();
    assert_eq!(prompt_use_count(dir.path(), &created.id).unwrap(), 4);
    assert!(used.last_used_at.is_some());
}

#[tokio::test]
async fn create_and_update_persist_model() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let created =
        create_prompt_in_dir_with_model(dir.path(), "模型片", "正文", None, Some("Flux")).unwrap();
    assert_eq!(created.model.as_deref(), Some("Flux"));
    let updated = update_prompt_in_dir_with_model(
        dir.path(),
        &created.id,
        "模型片",
        "正文",
        None,
        Some("GPT-5"),
    )
    .unwrap();
    assert_eq!(updated.model.as_deref(), Some("GPT-5"));
}

#[tokio::test]
async fn creates_empty_collection() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let image = list_categories_in_dir(dir.path())
        .unwrap()
        .into_iter()
        .find(|category| category.name == "图片生成")
        .unwrap();
    let collection =
        create_collection_in_dir(dir.path(), "人像灵感", Some(&image.id), "none", None).unwrap();
    assert_eq!(collection.title, "人像灵感");
    assert_eq!(
        collection_member_count(dir.path(), &collection.id).unwrap(),
        0
    );
}

#[tokio::test]
async fn adds_member_via_collection_id() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let collection = create_collection_in_dir(dir.path(), "合集A", None, "none", None).unwrap();
    let prompt = create_prompt_in_dir(dir.path(), "提示词B", "正文", None).unwrap();
    add_prompt_to_collection_in_dir(dir.path(), &prompt.id, &collection.id).unwrap();
    let members = list_collection_members_in_dir(dir.path(), &collection.id).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].title, "提示词B");
    assert_eq!(members[0].collection_id.as_deref(), Some(collection.id.as_str()));
}

#[tokio::test]
async fn persists_grid_cover_refs() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let created = create_collection_in_dir(
        dir.path(),
        "人像灵感",
        None,
        "grid",
        Some(r#"["one.jpg","two.jpg","three.jpg"]"#),
    )
    .unwrap();
    assert_eq!(created.cover_type, "grid");
    assert_eq!(created.cover_json, r#"["one.jpg","two.jpg","three.jpg"]"#);
    let listed = list_collections_in_dir(dir.path(), "", None).unwrap();
    assert_eq!(listed[0].cover_json, created.cover_json);
}

#[tokio::test]
async fn theme_persists_as_dark() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    set_setting_in_dir(dir.path(), "theme", "dark").unwrap();
    assert_eq!(get_setting_in_dir(dir.path(), "theme").unwrap(), "dark");
}

#[tokio::test]
async fn import_preview_does_not_write() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    create_prompt_in_dir(dir.path(), "已有", "x", None).unwrap();
    let preview = preview_import_json_in_dir(
        dir.path(),
        r#"{"prompts":[{"title":"一","content":"a"},{"title":"二","content":"b"}]}"#,
    )
    .unwrap();
    assert_eq!(preview.prompt_count, 2);
    assert_eq!(count_local_prompts_in_dir(dir.path()).unwrap(), 1);
}

#[tokio::test]
async fn restore_replaces_library() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    create_prompt_in_dir(dir.path(), "条目A", "a", None).unwrap();
    let backup = dir.path().join("backup.sqlite");
    backup_library_in_dir(dir.path(), &backup).unwrap();
    create_prompt_in_dir(dir.path(), "条目B", "b", None).unwrap();
    assert_eq!(count_local_prompts_in_dir(dir.path()).unwrap(), 2);
    restore_library_in_dir(dir.path(), &backup).unwrap();
    let rows = list_prompts_in_dir(dir.path(), "", None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "条目A");
}

#[tokio::test]
async fn failed_restore_leaves_library() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    create_prompt_in_dir(dir.path(), "条目A", "a", None).unwrap();
    let garbage = dir.path().join("garbage.sqlite");
    std::fs::write(&garbage, "not a database").unwrap();
    assert!(restore_library_in_dir(dir.path(), &garbage).is_err());
    let rows = list_prompts_in_dir(dir.path(), "", None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "条目A");
}

#[tokio::test]
async fn export_zip_does_not_remove_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    create_prompt_in_dir(dir.path(), "条目A", "a", None).unwrap();
    let zip = dir.path().join("backups").join("library.zip");
    export_library_zip_in_dir(dir.path(), &zip).unwrap();
    assert!(zip.exists());
    assert!(dir.path().join("promptark.sqlite").exists());
    assert_eq!(count_local_prompts_in_dir(dir.path()).unwrap(), 1);
}

#[tokio::test]
async fn auto_backup_leaves_existing_backup() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    create_prompt_in_dir(dir.path(), "条目A", "a", None).unwrap();
    let existing = dir.path().join("backups").join("manual.sqlite");
    backup_library_in_dir(dir.path(), &existing).unwrap();
    let auto = dir.path().join("backups").join("auto-latest.sqlite");
    backup_library_in_dir(dir.path(), &auto).unwrap();
    assert!(existing.exists());
    assert!(auto.exists());
}

#[tokio::test]
async fn clear_use_history_keeps_prompt_content() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let created = create_prompt_in_dir(dir.path(), "条目A", "中文 English", None).unwrap();
    record_prompt_use_in_dir(dir.path(), &created.id).unwrap();
    assert_eq!(prompt_use_count(dir.path(), &created.id).unwrap(), 1);
    clear_prompt_use_in_dir(dir.path()).unwrap();
    let rows = list_prompts_in_dir(dir.path(), "", None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].content, "中文 English");
    assert_eq!(prompt_use_count(dir.path(), &created.id).unwrap(), 0);
}

#[tokio::test]
#[ignore]
async fn search_ten_thousand_prompts_bench() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    for index in 0..10_000 {
        let title = if index == 5_000 {
            "官网生成器".to_string()
        } else {
            format!("条目{index}")
        };
        create_prompt_in_dir(dir.path(), &title, "正文", None).unwrap();
    }
    let started = std::time::Instant::now();
    let rows = list_prompts_in_dir(dir.path(), "官网", None).unwrap();
    let millis = started.elapsed().as_secs_f64() * 1000.0;
    eprintln!(
        "search_ten_thousand_prompts_bench {millis:.2}ms hits={}",
        rows.len()
    );
    assert!(!rows.is_empty());
}
