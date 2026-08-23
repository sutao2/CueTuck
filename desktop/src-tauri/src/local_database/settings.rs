use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub prompt_count: usize,
    pub collection_count: usize,
    pub titles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ImportFile {
    #[serde(default)]
    prompts: Vec<ImportPrompt>,
    #[serde(default)]
    collections: Vec<ImportCollection>,
}

#[derive(Debug, Deserialize)]
struct ImportPrompt {
    title: String,
    #[serde(default)]
    content: String,
}

#[derive(Debug, Deserialize)]
struct ImportCollection {
    title: String,
}

fn open_db(dir: &Path) -> Result<Connection, String> {
    Connection::open(dir.join("promptark.sqlite")).map_err(|error| error.to_string())
}

pub fn set_setting_in_dir(dir: &Path, key: &str, value: &str) -> Result<(), String> {
    let connection = open_db(dir)?;
    let encoded = serde_json::to_string(value).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO settings (key, value_json) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
            rusqlite::params![key, encoded],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn get_setting_in_dir(dir: &Path, key: &str) -> Result<String, String> {
    let connection = open_db(dir)?;
    let raw: String = connection
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

pub fn preview_import_json_in_dir(dir: &Path, json: &str) -> Result<ImportPreview, String> {
    let _ = dir;
    let file: ImportFile = serde_json::from_str(json).map_err(|error| error.to_string())?;
    Ok(ImportPreview {
        prompt_count: file.prompts.len(),
        collection_count: file.collections.len(),
        titles: file
            .prompts
            .iter()
            .map(|prompt| prompt.title.clone())
            .chain(file.collections.iter().map(|collection| collection.title.clone()))
            .collect(),
    })
}

pub fn export_library_json_in_dir(dir: &Path) -> Result<String, String> {
    let connection = open_db(dir)?;
    let mut prompt_statement = connection
        .prepare(
            "SELECT title, content FROM prompts WHERE deleted_at IS NULL ORDER BY title",
        )
        .map_err(|error| error.to_string())?;
    let prompts = prompt_statement
        .query_map([], |row| {
            Ok(serde_json::json!({
                "title": row.get::<_, String>(0)?,
                "content": row.get::<_, String>(1)?,
            }))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut collection_statement = connection
        .prepare("SELECT title FROM collections WHERE deleted_at IS NULL ORDER BY title")
        .map_err(|error| error.to_string())?;
    let collections = collection_statement
        .query_map([], |row| {
            Ok(serde_json::json!({ "title": row.get::<_, String>(0)? }))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    serde_json::to_string_pretty(&serde_json::json!({
        "prompts": prompts,
        "collections": collections,
    }))
    .map_err(|error| error.to_string())
}

pub fn apply_import_json_in_dir(dir: &Path, json: &str) -> Result<ImportPreview, String> {
    let preview = preview_import_json_in_dir(dir, json)?;
    let file: ImportFile = serde_json::from_str(json).map_err(|error| error.to_string())?;
    for prompt in file.prompts {
        super::create_prompt_in_dir(dir, &prompt.title, &prompt.content, None)?;
    }
    for collection in file.collections {
        super::create_collection_in_dir(dir, &collection.title, None, "none", None)?;
    }
    Ok(preview)
}
