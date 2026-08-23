use crate::local_database::{
    add_prompt_to_collection_in_dir, apply_import_json_in_dir, backup_library_in_dir,
    count_local_prompts_in_dir, create_category_in_dir, create_collection_in_dir,
    create_prompt_in_dir, delete_prompt_in_dir, import_downloaded_prompt_in_dir,
    export_library_json_in_dir, get_setting_in_dir, list_categories_in_dir,
    list_collection_members_in_dir, list_collections_in_dir, list_prompts_in_dir,
    preview_import_json_in_dir, record_prompt_use_in_dir, restore_library_in_dir, set_setting_in_dir,
    update_prompt_in_dir, CategoryRecord, CollectionRecord, ImportPreview, LocalDatabase,
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
pub fn get_local_database_status(database: State<'_, LocalDatabase>) -> String {
    database.status().as_str().to_string()
}

#[tauri::command]
pub fn initialize_local_database(
    app: AppHandle,
    database: State<'_, LocalDatabase>,
) -> Result<String, String> {
    database.initialize(&data_dir(&app)?)
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
) -> Result<PromptRecord, String> {
    create_prompt_in_dir(&data_dir(&app)?, &title, &content, category_id.as_deref())
}

#[tauri::command]
pub fn import_downloaded_prompt(
    app: AppHandle,
    title: String,
    content: String,
    remote_id: Option<String>,
) -> Result<PromptRecord, String> {
    import_downloaded_prompt_in_dir(&data_dir(&app)?, &title, &content, remote_id.as_deref())
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
) -> Result<PromptRecord, String> {
    update_prompt_in_dir(&data_dir(&app)?, &id, &title, &content, category_id.as_deref())
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
    parent_id: String,
) -> Result<CategoryRecord, String> {
    create_category_in_dir(&data_dir(&app)?, &name, &parent_id)
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
pub fn get_local_setting(app: AppHandle, key: String) -> Result<String, String> {
    get_setting_in_dir(&data_dir(&app)?, &key)
}

#[tauri::command]
pub fn set_local_setting(app: AppHandle, key: String, value: String) -> Result<(), String> {
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
        Some(path) => PathBuf::from(path),
        None => {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_secs();
            dir.join("backups").join(format!("promptark-{stamp}.sqlite"))
        }
    };
    backup_library_in_dir(&dir, &dest_path)
}

#[tauri::command]
pub fn restore_local_library(app: AppHandle, src: String) -> Result<(), String> {
    restore_library_in_dir(&data_dir(&app)?, PathBuf::from(src).as_path())
}
