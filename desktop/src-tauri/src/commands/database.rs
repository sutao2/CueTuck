use crate::local_database::{
    add_prompt_to_collection_in_dir, apply_import_json_in_dir, backup_library_in_dir,
    clear_prompt_use_in_dir, count_local_prompts_in_dir, create_category_in_dir,
    create_collection_in_dir, create_prompt_in_dir_with_model, delete_prompt_in_dir, delete_category_in_dir,
    import_downloaded_prompt_with_metadata, export_library_json_in_dir, export_library_zip_in_dir,
    get_setting_in_dir, list_categories_in_dir, list_collection_members_in_dir,
    list_collections_in_dir, list_prompts_in_dir, preview_import_json_in_dir,
    record_prompt_use_in_dir, set_setting_in_dir,
    update_prompt_in_dir_with_model,
    upsert_synced_prompt_in_dir, CategoryRecord, CollectionRecord, ImportPreview, LocalDatabase,
    PromptRecord,
};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_deleted_local_items(app: AppHandle, query: String) -> Result<Vec<crate::local_database::recovery::DeletedItem>, String> {
    crate::local_database::recovery::list_deleted(&data_dir(&app)?, &query)
}

#[tauri::command]
pub fn restore_deleted_local_item(app: AppHandle, id: String, kind: String) -> Result<(), String> {
    crate::local_database::recovery::restore_deleted(&data_dir(&app)?, &id, &kind)
}

#[tauri::command]
pub fn create_collection_prompt(app: AppHandle, collection_id: String, title: String, content: String, category_id: Option<String>, model: Option<String>, assets: Vec<crate::local_database::assets::Asset>) -> Result<PromptRecord, String> {
    crate::local_database::collections::create_member_in_dir(&data_dir(&app)?, &collection_id, &title, &content, category_id.as_deref(), model.as_deref(), &assets)
}

#[tauri::command]
pub fn save_local_prompt_with_assets(app: AppHandle, id: Option<String>, title: String, content: String, category_id: Option<String>, model: Option<String>, assets: Vec<crate::local_database::assets::Asset>) -> Result<PromptRecord, String> {
    crate::local_database::assets::save_prompt(&data_dir(&app)?, id.as_deref(), &title, &content, category_id.as_deref(), model.as_deref(), &assets)
}

#[tauri::command]
pub fn list_local_prompt_assets(app: AppHandle, prompt_id: String) -> Result<Vec<crate::local_database::assets::Asset>, String> {
    crate::local_database::assets::list(&data_dir(&app)?, &prompt_id)
}

#[tauri::command]
pub async fn get_local_prompt_image(app: AppHandle, prompt_id: String) -> Result<Option<crate::local_database::assets::Asset>, String> {
    let dir = data_dir(&app)?;
    tauri::async_runtime::spawn_blocking(move || crate::local_database::assets::first_image(&dir, &prompt_id)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_local_prompt_thumbnail(app: AppHandle, prompt_id: String) -> Result<Option<crate::local_database::assets::Asset>, String> {
    static WORKERS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
    let dir = data_dir(&app)?;
    let permit = WORKERS.acquire().await.map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        crate::local_database::assets::first_thumbnail(&dir, &prompt_id)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn append_downloaded_assets(app: AppHandle, prompt_id: String, remote_id: String, assets: Vec<crate::local_database::assets::Asset>) -> Result<PromptRecord,String> {
    crate::local_database::assets::append_downloaded(&data_dir(&app)?,&prompt_id,&remote_id,&assets)
}

#[tauri::command]
pub fn export_local_prompt_asset(app: AppHandle, prompt_id: String, asset_id: String) -> Result<String, String> {
    let path = crate::local_database::assets::export_file(&data_dir(&app)?, &prompt_id, &asset_id)?;
    // Reveal the containing directory, never execute the attachment.
    open::that(std::path::Path::new(&path).parent().ok_or("导出路径无效")?).map_err(|e| e.to_string())?;
    Ok(path)
}

#[tauri::command]
pub fn export_local_sync_changes(app: AppHandle, include_assets: Option<bool>) -> Result<Vec<crate::local_database::SyncChange>, String> {
    if include_assets.unwrap_or(false) { crate::local_database::sync::export_with_assets(&data_dir(&app)?, true) }
    else { crate::local_database::export_sync_changes(&data_dir(&app)?) }
}

#[tauri::command]
pub fn apply_local_sync_changes(app: AppHandle, items: Vec<crate::local_database::SyncChange>, keep_local: bool, include_assets: Option<bool>) -> Result<(), String> {
    if include_assets.unwrap_or(false) { crate::local_database::sync::apply_with_assets(&data_dir(&app)?, &items, keep_local, false, true) }
    else { crate::local_database::apply_sync_changes(&data_dir(&app)?, &items, keep_local) }
}

#[tauri::command]
pub fn get_local_database_status(database: State<'_, LocalDatabase>) -> String {
    database.status().as_str().to_string()
}

#[tauri::command]
pub fn initialize_local_database(
    app: AppHandle,
    database: State<'_, LocalDatabase>,
) -> Result<String, String> {
    database.initialize(&data_dir(&app)?)?;
    crate::http::load_from_dir(&data_dir(&app)?);
    Ok(database.status().as_str().to_string())
}

#[tauri::command]
pub fn count_local_prompts(app: AppHandle) -> Result<i64, String> {
    count_local_prompts_in_dir(&data_dir(&app)?)
}

#[tauri::command]
pub fn create_local_prompt(
    app: AppHandle,
    title: String,
    content: String,
    category_id: Option<String>,
    model: Option<String>,
) -> Result<PromptRecord, String> {
    create_prompt_in_dir_with_model(
        &data_dir(&app)?,
        &title,
        &content,
        category_id.as_deref(),
        model.as_deref(),
    )
}

#[tauri::command]
pub fn import_downloaded_prompt(
    app: AppHandle,
    title: String,
    content: String,
    remote_id: Option<String>,
    author: Option<String>,
    category_id: Option<String>,
    model: Option<String>,
) -> Result<PromptRecord, String> {
    import_downloaded_prompt_with_metadata(
        &data_dir(&app)?,
        &title,
        &content,
        remote_id.as_deref(),
        author.as_deref(),
        category_id.as_deref(),
        model.as_deref(),
    )
}

#[tauri::command]
pub fn upsert_synced_local_prompt(
    app: AppHandle,
    id: String,
    title: String,
    content: String,
    category_id: Option<String>,
    updated_at: Option<String>,
) -> Result<PromptRecord, String> {
    upsert_synced_prompt_in_dir(
        &data_dir(&app)?,
        &id,
        &title,
        &content,
        category_id.as_deref(),
        updated_at.as_deref().unwrap_or("0"),
    )
}

#[tauri::command]
pub fn list_local_prompts(
    app: AppHandle,
    query: Option<String>,
    category_id: Option<String>,
) -> Result<Vec<PromptRecord>, String> {
    list_prompts_in_dir(
        &data_dir(&app)?,
        query.as_deref().unwrap_or(""),
        category_id.as_deref(),
    )
}

#[tauri::command]
pub fn update_local_prompt(
    app: AppHandle,
    id: String,
    title: String,
    content: String,
    category_id: Option<String>,
    model: Option<String>,
) -> Result<PromptRecord, String> {
    update_prompt_in_dir_with_model(
        &data_dir(&app)?,
        &id,
        &title,
        &content,
        category_id.as_deref(),
        model.as_deref(),
    )
}

#[tauri::command]
pub fn delete_local_prompt(app: AppHandle, id: String) -> Result<(), String> {
    delete_prompt_in_dir(&data_dir(&app)?, &id)
}

#[tauri::command]
pub fn list_local_categories(app: AppHandle) -> Result<Vec<CategoryRecord>, String> {
    list_categories_in_dir(&data_dir(&app)?)
}

#[tauri::command]
pub fn create_local_category(
    app: AppHandle,
    name: String,
    parent_id: Option<String>,
) -> Result<CategoryRecord, String> {
    create_category_in_dir(&data_dir(&app)?, &name, parent_id.as_deref())
}

#[tauri::command]
pub fn delete_local_category(app: AppHandle, id: String) -> Result<(), String> {
    delete_category_in_dir(&data_dir(&app)?, &id)
}

#[tauri::command]
pub fn record_local_prompt_use(app: AppHandle, id: String) -> Result<PromptRecord, String> {
    record_prompt_use_in_dir(&data_dir(&app)?, &id)
}

#[tauri::command]
pub fn create_local_collection(
    app: AppHandle,
    title: String,
    category_id: Option<String>,
    cover_type: Option<String>,
    cover_json: Option<String>,
) -> Result<CollectionRecord, String> {
    create_collection_in_dir(
        &data_dir(&app)?,
        &title,
        category_id.as_deref(),
        cover_type.as_deref().unwrap_or("none"),
        cover_json.as_deref(),
    )
}

#[tauri::command]
pub fn list_local_collections(
    app: AppHandle,
    query: Option<String>,
    category_id: Option<String>,
) -> Result<Vec<CollectionRecord>, String> {
    list_collections_in_dir(
        &data_dir(&app)?,
        query.as_deref().unwrap_or(""),
        category_id.as_deref(),
    )
}

#[tauri::command]
pub fn add_prompt_to_local_collection(
    app: AppHandle,
    prompt_id: String,
    collection_id: String,
) -> Result<(), String> {
    add_prompt_to_collection_in_dir(&data_dir(&app)?, &prompt_id, &collection_id)
}

#[tauri::command]
pub fn list_local_collection_members(
    app: AppHandle,
    collection_id: String,
) -> Result<Vec<PromptRecord>, String> {
    list_collection_members_in_dir(&data_dir(&app)?, &collection_id)
}

#[tauri::command]
pub fn remove_prompt_from_local_collection(app: AppHandle, prompt_id: String, collection_id: String) -> Result<(), String> {
    crate::local_database::remove_prompt_from_collection_in_dir(&data_dir(&app)?, &prompt_id, &collection_id)
}

#[tauri::command]
pub fn update_local_collection(app: AppHandle, id: String, title: String, category_id: Option<String>, cover_type: String, cover_json: String) -> Result<(), String> {
    crate::local_database::update_collection_in_dir(&data_dir(&app)?, &id, &title, category_id.as_deref(), &cover_type, &cover_json)
}

#[tauri::command]
pub fn delete_local_collection(app: AppHandle, id: String) -> Result<(), String> {
    crate::local_database::delete_collection_in_dir(&data_dir(&app)?, &id)
}

#[tauri::command]
pub fn get_local_setting(app: AppHandle, key: String) -> Result<String, String> {
    get_setting_in_dir(&data_dir(&app)?, &key)
}

#[tauri::command]
pub fn set_local_setting(app: AppHandle, key: String, value: String) -> Result<(), String> {
    if key == "http_proxy" {
        crate::http::set_runtime_proxy(&value)?;
    }
    set_setting_in_dir(&data_dir(&app)?, &key, &value)
}

#[tauri::command]
pub fn export_local_library(app: AppHandle) -> Result<String, String> {
    export_library_json_in_dir(&data_dir(&app)?)
}

#[tauri::command]
pub fn preview_local_import(app: AppHandle, json: String) -> Result<ImportPreview, String> {
    preview_import_json_in_dir(&data_dir(&app)?, &json)
}

#[tauri::command]
pub fn apply_local_import(app: AppHandle, json: String) -> Result<ImportPreview, String> {
    apply_import_json_in_dir(&data_dir(&app)?, &json)
}

#[tauri::command]
pub fn backup_local_library(app: AppHandle, dest: Option<String>) -> Result<String, String> {
    let dir = data_dir(&app)?;
    let dest_path = match dest.filter(|value| !value.trim().is_empty()) {
        Some(path) => {
            let path = PathBuf::from(path);
            if path.is_absolute() { path } else { dir.join(path) }
        },
        None => {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_secs();
            dir.join("backups").join(format!("promptark-{stamp}-{}.sqlite", uuid::Uuid::new_v4()))
        }
    };
    backup_library_in_dir(&dir, &dest_path)
}

#[tauri::command]
pub fn set_auto_backup(app: AppHandle, enabled: bool) -> Result<(), String> {
    crate::local_database::set_auto_backup_in_dir(&data_dir(&app)?, enabled)
}

#[tauri::command]
pub fn restore_local_library(app: AppHandle, src: String, expected_digest: Option<String>) -> Result<String, String> {
    crate::local_database::restore_library_checked(&data_dir(&app)?, PathBuf::from(src).as_path(), expected_digest.as_deref())
}

#[tauri::command]
pub fn preview_library_restore(app: AppHandle, src: String) -> Result<crate::local_database::RestorePreview, String> {
    crate::local_database::preview_library_restore(&data_dir(&app)?, PathBuf::from(src).as_path())
}

#[tauri::command]
pub fn open_library_dir(app: AppHandle) -> Result<String, String> {
    let dir = data_dir(&app)?;
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .status()
            .map_err(|error| error.to_string())?;
    }
    Ok(dir.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn export_library_zip(app: AppHandle, dest: Option<String>) -> Result<String, String> {
    let dir = data_dir(&app)?;
    let dest_path = match dest.filter(|value| !value.trim().is_empty()) {
        Some(path) => PathBuf::from(path),
        None => dir.join("backups").join("promptark-library.zip"),
    };
    export_library_zip_in_dir(&dir, &dest_path)
}

#[tauri::command]
pub fn clear_local_prompt_use(app: AppHandle) -> Result<(), String> {
    clear_prompt_use_in_dir(&data_dir(&app)?)
}


#[tauri::command]
pub fn move_local_prompt_category(app: AppHandle, id: String, category_id: Option<String>) -> Result<(), String> {
    crate::local_database::move_prompt_category_in_dir(&data_dir(&app)?, &id, category_id.as_deref())
}


#[tauri::command]
pub async fn choose_library_backup(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().set_title("选择提示词库备份")
            .add_filter("SQLite 备份", &["sqlite", "sqlite3", "db"])
            .blocking_pick_file()
            .map(|file| file.into_path().map(|path| path.to_string_lossy().into_owned()).map_err(|error| error.to_string()))
            .transpose()
    }).await.map_err(|error| error.to_string())?
}
