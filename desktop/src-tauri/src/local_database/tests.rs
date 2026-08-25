use super::{
    add_prompt_to_collection_in_dir, backup_library_in_dir, collection_member_count,
    count_local_prompts_in_dir, create_category_in_dir, create_collection_in_dir,
    create_prompt_in_dir, delete_prompt_in_dir, export_library_zip_in_dir, get_setting_in_dir,
    import_downloaded_prompt_in_dir,
    initialize_in_dir,
    list_categories_in_dir, list_collection_members_in_dir, list_collections_in_dir,
    list_prompts_in_dir,
    list_system_category_names, preview_import_json_in_dir, prompt_deleted_at, prompt_use_count,
    clear_prompt_use_in_dir, record_prompt_use_in_dir, restore_library_in_dir, set_setting_in_dir,
    update_prompt_in_dir, upsert_synced_prompt_in_dir,
};

#[tokio::test]
async fn status_is_ready_after_initialize() {
    let dir = tempfile::tempdir().unwrap();
    let status = initialize_in_dir(dir.path()).unwrap();
    assert_eq!(status, "ready");
}

#[tokio::test]
async fn empty_library_counts_zero() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    assert_eq!(count_local_prompts_in_dir(dir.path()).unwrap(), 0);
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
    let created = create_prompt_in_dir(dir.path(), "本地仍在", "正文", None).unwrap();
    let kept = upsert_synced_prompt_in_dir(
        dir.path(),
        &created.id,
        "远端标题",
        "远端正文",
        None,
        "9",
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
    let created = create_category_in_dir(dir.path(), "周报", "cat-office").unwrap();
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
    let error = create_category_in_dir(dir.path(), "再下一层", "cat-software-1").unwrap_err();
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
    record_prompt_use_in_dir(dir.path(), &created.id).unwrap();
    assert_eq!(prompt_use_count(dir.path(), &created.id).unwrap(), 4);
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
