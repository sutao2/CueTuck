use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub prompt_count: usize,
    pub collection_count: usize,
    pub titles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ImportFile {
    #[serde(default)]
    version: Option<u32>,
    #[serde(default)]
    prompts: Vec<Value>,
    #[serde(default)]
    collections: Vec<Value>,
    #[serde(default)]
    categories: Vec<Value>,
}

fn open_db(dir: &Path) -> Result<Connection, String> {
    Connection::open(dir.join("promptark.sqlite")).map_err(|error| error.to_string())
}

pub fn set_setting_in_dir(dir: &Path, key: &str, value: &str) -> Result<(), String> {
    let connection = open_db(dir)?;
    let encoded = serde_json::to_string(value).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO settings (key, value_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
            rusqlite::params![key, encoded, super::now_millis()],
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
    prepare_import(json, "0").map(|(preview, _)| preview)
}

pub fn export_library_json_in_dir(dir: &Path) -> Result<String, String> {
    let snapshot = super::export_sync_changes(dir)?;
    let rows = |kind| snapshot.iter().filter(|row| row.kind == kind && row.deleted_at.is_none()).map(|row| row.payload.clone()).collect::<Vec<_>>();
    serde_json::to_string_pretty(&json!({ "version": 2,
        "prompts": rows("prompt"), "collections": rows("collection"), "categories": rows("category"),
    }))
    .map_err(|error| error.to_string())
}

pub fn apply_import_json_in_dir(dir: &Path, json: &str) -> Result<ImportPreview, String> {
    let (preview, changes) = prepare_import(json, &super::now_millis())?;
    super::apply_sync_changes(dir, &changes, false)?;
    Ok(preview)
}

fn prepare_import(raw: &str, timestamp: &str) -> Result<(ImportPreview, Vec<super::SyncChange>), String> {
    let file: ImportFile = serde_json::from_str(raw).map_err(|error| error.to_string())?;
    if file.version.is_some_and(|version| version != 1 && version != 2) { return Err("不支持的导入格式".into()); }
    let system_ids: Vec<String> = super::SYSTEM_CATEGORIES.iter().flat_map(|(id, _, children)| {
        std::iter::once(id.to_string()).chain((0..children.len()).map(move |index| format!("{id}-{index}")))
    }).collect();
    let groups = [("category", &file.categories), ("collection", &file.collections), ("prompt", &file.prompts)];
    let mut ids = HashMap::new();
    for (kind, rows) in &groups {
        for row in *rows {
            let field = if *kind == "category" { "name" } else { "title" };
            if row[field].as_str().unwrap_or("").trim().is_empty() { return Err("导入记录缺少名称".into()); }
            super::sync::validate_payload(kind, row)?;
            let original = row["id"].as_str().filter(|id| !id.is_empty());
            if *kind == "category" && original.is_none() { return Err("分类缺少 id".into()); }
            if let Some(original) = original.filter(|id| !id.is_empty()) {
                let id = if *kind == "category" && system_ids.iter().any(|id| id == original) { original.into() } else { Uuid::new_v4().to_string() };
                if ids.insert((*kind, original.to_string()), id).is_some() { return Err("导入记录 id 重复".into()); }
            }
        }
    }
    let reference = |kind, value: &Value| -> Result<Value, String> {
        if value.is_null() { return Ok(Value::Null); }
        let id = value.as_str().ok_or("导入关联格式错误")?;
        ids.get(&(kind, id.to_string())).cloned()
            .or_else(|| (kind == "category" && system_ids.iter().any(|item| item == id)).then(|| id.to_string()))
            .map(|id| json!(id)).ok_or_else(|| format!("导入记录引用不存在的 {kind}"))
    };
    let mut changes = Vec::new();
    for (kind, rows) in groups {
        for row in rows {
            if kind == "category" && system_ids.iter().any(|id| row["id"] == *id) { continue; }
            let id = ids.get(&(kind, row["id"].as_str().unwrap_or("").to_string())).cloned().unwrap_or_else(|| Uuid::new_v4().to_string());
            let mut payload = row.clone();
            payload["id"] = json!(id);
            payload["updated_at"] = json!(timestamp);
            payload["deleted_at"] = Value::Null;
            if kind == "category" {
                if !super::SYSTEM_CATEGORIES.iter().any(|(id, _, _)| row["parent_id"] == *id) { return Err("导入小分类必须属于系统大分类".into()); }
                payload["is_system"] = json!(0);
            } else {
                payload["title"] = json!(row["title"].as_str().unwrap().trim());
                payload["category_id"] = reference("category", &row["category_id"])?;
                payload["created_at"] = json!(timestamp);
                if kind == "prompt" {
                    payload["collection_id"] = reference("collection", &row["collection_id"])?;
                    payload["content"] = json!(row["content"].as_str().unwrap_or(""));
                    payload["source"] = json!(row["source"].as_str().unwrap_or("local"));
                } else {
                    payload["cover_type"] = json!(row["cover_type"].as_str().unwrap_or("none"));
                    payload["cover_json"] = json!(row["cover_json"].as_str().unwrap_or("[]"));
                    serde_json::from_str::<Vec<String>>(payload["cover_json"].as_str().unwrap()).map_err(|_| "导入封面格式错误")?;
                }
            }
            changes.push(super::SyncChange { id, kind: kind.into(), payload, updated_at: timestamp.into(), deleted_at: None });
        }
    }
    let preview = ImportPreview { prompt_count: file.prompts.len(), collection_count: file.collections.len(),
        titles: file.prompts.iter().chain(&file.collections).map(|row| row["title"].as_str().unwrap().to_string()).collect() };
    Ok((preview, changes))
}
