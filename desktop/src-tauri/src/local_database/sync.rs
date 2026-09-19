use rusqlite::{params_from_iter, types::Value as SqlValue, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncChange {
    pub id: String,
    pub kind: String,
    pub payload: Value,
    pub updated_at: String,
    #[serde(default)]
    pub deleted_at: Option<String>,
}

pub(crate) fn validate_payload(kind: &str, payload: &Value) -> Result<(), String> {
    let object = payload.as_object().ok_or("记录必须是对象")?;
    let (_, _, columns) = schema(kind).ok_or("记录类型错误")?;
    for field in columns {
        if let Some(value) = object.get(*field).filter(|value| !value.is_null()) {
            let valid = match *field {
                "sort_order" | "use_count" | "version" => value.as_i64().is_some_and(|n| n >= 0),
                "is_system" => value.is_boolean() || value.as_i64().is_some_and(|n| n == 0 || n == 1),
                _ => value.is_string(),
            };
            if !valid { return Err(format!("字段 {field} 格式错误")); }
        }
    }
    if kind == "setting" {
        serde_json::from_str::<String>(payload["value_json"].as_str().ok_or("设置格式错误")?).map_err(|_| "设置格式错误")?;
    }
    Ok(())
}

const SETTINGS: &[&str] = &[
    "theme",
    "default_model",
    "model_catalog",
    "custom_models",
    "show_model_tags",
    "ui_language",
    "variable_hints",
];

// Table and column names are fixed here, never taken from a remote payload.
fn schema(kind: &str) -> Option<(&'static str, &'static str, &'static [&'static str])> {
    match kind {
        "category" => Some((
            "categories",
            "id",
            &[
                "id",
                "parent_id",
                "name",
                "icon",
                "is_system",
                "sort_order",
                "updated_at",
                "deleted_at",
            ],
        )),
        "collection" => Some((
            "collections",
            "id",
            &[
                "id",
                "title",
                "description",
                "category_id",
                "cover_type",
                "cover_json",
                "created_at",
                "updated_at",
                "deleted_at",
            ],
        )),
        "prompt" => Some((
            "prompts",
            "id",
            &[
                "id",
                "title",
                "summary",
                "content",
                "category_id",
                "collection_id",
                "model",
                "source",
                "remote_id",
                "version",
                "use_count",
                "last_used_at",
                "created_at",
                "updated_at",
                "deleted_at",
                "author",
            ],
        )),
        "setting" => Some(("settings", "key", &["key", "value_json", "updated_at"])),
        _ => None,
    }
}

pub fn export_sync_changes(dir: &Path) -> Result<Vec<SyncChange>, String> {
    export_with_assets(dir, false)
}

pub fn export_with_assets(dir: &Path, include_assets: bool) -> Result<Vec<SyncChange>, String> {
    let mut database = Connection::open(dir.join("promptark.sqlite")).map_err(|e| e.to_string())?;
    let connection = database.transaction().map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for kind in ["category", "collection", "prompt", "setting"] {
        let (table, key, columns) = schema(kind).unwrap();
        let mut statement = connection
            .prepare(&format!("SELECT {} FROM {table}", columns.join(",")))
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| {
                let mut payload = Map::new();
                for (index, name) in columns.iter().enumerate() {
                    let value = match row.get::<_, SqlValue>(index)? {
                        SqlValue::Null => Value::Null,
                        SqlValue::Integer(n) => json!(n),
                        SqlValue::Real(n) => json!(n),
                        SqlValue::Text(s) => json!(s),
                        SqlValue::Blob(_) => Value::Null,
                    };
                    payload.insert(name.to_string(), value);
                }
                Ok(Value::Object(payload))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let mut payload = row.map_err(|e| e.to_string())?;
            let id = payload[key].as_str().unwrap_or("").to_string();
            if include_assets && kind == "prompt" && payload["deleted_at"].is_null() {
                payload["assets"] = serde_json::to_value(super::assets::read(&connection, &id)?).map_err(|e| e.to_string())?;
            }
            if kind == "setting" && !SETTINGS.contains(&id.as_str()) {
                continue;
            }
            result.push(SyncChange {
                id: if kind == "setting" {
                    format!("setting:{id}")
                } else {
                    id
                },
                kind: kind.into(),
                updated_at: super::timestamp_ms(payload["updated_at"].as_str().unwrap_or("0"))
                    .to_string(),
                deleted_at: payload["deleted_at"].as_str().map(str::to_string),
                payload,
            });
        }
    }
    Ok(result)
}

pub fn apply_sync_changes(
    dir: &Path,
    items: &[SyncChange],
    keep_local: bool,
) -> Result<(), String> {
    apply_changes(dir, items, keep_local, false)
}

pub(crate) fn apply_changes(dir: &Path, items: &[SyncChange], keep_local: bool, local_import: bool) -> Result<(), String> {
    apply_with_assets(dir, items, keep_local, local_import, false)
}

pub fn apply_with_assets(dir: &Path, items: &[SyncChange], keep_local: bool, local_import: bool, merge_assets: bool) -> Result<(), String> {
    if items.iter().any(|item| {
        schema(&item.kind).is_none()
            || item.updated_at.parse::<u64>().is_err()
            || item.updated_at.len() > 16
    }) {
        return Err("同步记录类型或时间无效".into());
    }
    let mut connection =
        Connection::open(dir.join("promptark.sqlite")).map_err(|e| e.to_string())?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    for kind in ["category", "collection", "prompt", "setting"] {
        let (table, key, columns) = schema(kind).unwrap();
        let mut ordered: Vec<_> = items.iter().filter(|item| item.kind == kind).collect();
        if kind == "category" { ordered.sort_by_key(|item| !item.payload["parent_id"].is_null()); }
        for item in ordered {
            let id = if kind == "setting" {
                item.id.strip_prefix("setting:").unwrap_or("")
            } else {
                &item.id
            };
            if id.is_empty() {
                return Err("同步记录缺少 id".into());
            }
            if kind == "setting" && !SETTINGS.contains(&id) {
                continue;
            }
            if !item.payload.is_object() {
                return Err("同步记录格式错误".into());
            }
            validate_payload(kind, &item.payload)?;
            let local: Option<String> = transaction
                .query_row(
                    &format!("SELECT COALESCE(updated_at, '0') FROM {table} WHERE {key} = ?1"),
                    [id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            // A text-only sync can already have this revision. Fill deferred covers
            // without treating the rest of an equal-time record as a new edit.
            if kind == "collection"
                && local.as_ref().is_some_and(|stamp| {
                    super::timestamp_ms(stamp) == super::timestamp_ms(&item.updated_at)
                })
            {
                if let (Some(cover_json), Some(cover_type)) = (
                    item.payload["cover_json"].as_str(),
                    item.payload["cover_type"].as_str(),
                ) {
                    transaction
                        .execute(
                            "UPDATE collections SET cover_json=?1, cover_type=?2 WHERE id=?3",
                            rusqlite::params![cover_json, cover_type, id],
                        )
                        .map_err(|e| e.to_string())?;
                }
            }
            if merge_assets && kind == "prompt" && item.deleted_at.is_none() && local.as_ref().is_some_and(|stamp| !keep_local && super::timestamp_ms(stamp) == super::timestamp_ms(&item.updated_at)) {
                merge_prompt_assets(&transaction, id, &item.payload)?;
            }
            if local.as_ref().is_some_and(|stamp| {
                (keep_local && kind == "prompt")
                    || super::timestamp_ms(stamp) >= super::timestamp_ms(&item.updated_at)
            }) {
                continue;
            }
            if kind == "category" {
                let system: Option<bool> = transaction
                    .query_row(
                        "SELECT is_system FROM categories WHERE id=?1",
                        [id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|e| e.to_string())?;
                if system == Some(true) {
                    continue;
                }
                let valid = if item.payload["parent_id"].is_null() { true } else {
                    let parent = item.payload["parent_id"].as_str().ok_or("同步分类无效")?;
                    transaction
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM categories WHERE id=?1 AND parent_id IS NULL)",
                        [parent],
                        |row| row.get(0),
                    )
                    .map_err(|e| e.to_string())?
                };
                if !valid
                    || item.payload["name"]
                        .as_str()
                        .unwrap_or("")
                        .trim()
                        .is_empty()
                {
                    return Err("同步分类无效".into());
                }
            }
            for (field, target) in [
                ("category_id", "categories"),
                ("collection_id", "collections"),
            ] {
                if let Some(reference) = item.payload[field].as_str() {
                    let exists: bool = transaction
                        .query_row(
                            &format!("SELECT EXISTS(SELECT 1 FROM {target} WHERE id=?1)"),
                            [reference],
                            |row| row.get(0),
                        )
                        .map_err(|e| e.to_string())?;
                    if !exists {
                        return Err(format!("同步记录引用不存在的 {field}"));
                    }
                }
            }
            if kind == "prompt" && !local_import {
                super::history::preserve(&transaction, id, &item.payload, item.deleted_at.is_some())?;
            }
            let mut payload = item.payload.as_object().unwrap().clone();
            payload.insert(key.into(), json!(id));
            payload.insert(
                "updated_at".into(),
                json!(super::timestamp_ms(&item.updated_at).to_string()),
            );
            if kind == "prompt" || kind == "collection" || kind == "category" {
                payload.insert("deleted_at".into(), json!(item.deleted_at));
                if kind != "category" && !payload.contains_key("title") && item.deleted_at.is_some() {
                    payload.insert("title".into(), json!(""));
                }
            }
            if kind == "category" {
                payload.insert("is_system".into(), json!(0));
                payload.insert("parent_id".into(), item.payload["parent_id"].clone());
            }
            let fields: Vec<_> = columns
                .iter()
                .copied()
                .filter(|field| payload.contains_key(*field))
                .collect();
            let values: Vec<SqlValue> = fields
                .iter()
                .map(|field| match &payload[*field] {
                    Value::String(s) => SqlValue::Text(s.clone()),
                    Value::Bool(b) => SqlValue::Integer(i64::from(*b)),
                    Value::Number(n) => SqlValue::Integer(n.as_i64().unwrap_or(0)),
                    _ => SqlValue::Null,
                })
                .collect();
            let updates = fields
                .iter()
                .filter(|field| **field != key)
                .map(|field| format!("{field}=excluded.{field}"))
                .collect::<Vec<_>>()
                .join(",");
            let placeholders = vec!["?"; fields.len()].join(",");
            transaction.execute(&format!("INSERT INTO {table} ({}) VALUES ({placeholders}) ON CONFLICT({key}) DO UPDATE SET {updates}", fields.join(",")), params_from_iter(values)).map_err(|e| e.to_string())?;
            if local_import && kind == "prompt" {
                let assets = serde_json::from_value::<Vec<super::assets::Asset>>(item.payload.get("assets").cloned().unwrap_or_else(|| json!([]))).map_err(|_| "附件格式错误")?;
                super::assets::replace(&transaction, id, &assets)?;
            }
            if merge_assets && kind == "prompt" && item.deleted_at.is_none() {
                merge_prompt_assets(&transaction, id, &item.payload)?;
            }
        }
    }
    let invalid_tree: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM categories child LEFT JOIN categories parent ON parent.id=child.parent_id
         WHERE child.parent_id IS NOT NULL AND (parent.id IS NULL OR parent.parent_id IS NOT NULL
         OR (child.deleted_at IS NULL AND parent.deleted_at IS NOT NULL)))",
        [], |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if invalid_tree { return Err("同步分类层级无效".into()); }
    for table in ["prompts", "collections"] {
        transaction.execute(&format!("UPDATE {table} SET category_id=NULL WHERE category_id IN (SELECT id FROM categories WHERE deleted_at IS NOT NULL)"), []).map_err(|e| e.to_string())?;
    }
    transaction.commit().map_err(|e| e.to_string())?;
    for item in items {
        super::observe_timestamp(super::timestamp_ms(&item.updated_at));
    }
    Ok(())
}

fn merge_prompt_assets(connection: &Connection, id: &str, payload: &Value) -> Result<(), String> {
    let Some(value) = payload.get("assets") else { return Ok(()) };
    let incoming: Vec<super::assets::Asset> = serde_json::from_value(value.clone()).map_err(|_| "附件格式错误")?;
    super::assets::validate(&incoming)?;
    let mut assets = super::assets::read(connection, id)?;
    for asset in incoming {
        if !assets.iter().any(|existing| existing.id == asset.id) { assets.push(asset); }
    }
    super::assets::replace(connection, id, &assets)
}

#[cfg(test)]
mod asset_sync_tests {
    use super::*;
    use crate::local_database::{self as db, assets::{self, Asset}};

    fn file() -> Asset {
        Asset { id: uuid::Uuid::new_v4().to_string(), name: "notes.txt".into(), mime: "text/plain".into(), data: "cHJpdmF0ZQ==".into() }
    }

    #[test]
    fn two_sqlite_libraries_round_trip_private_assets_only_with_explicit_consent() {
        let a = tempfile::tempdir().unwrap(); let b = tempfile::tempdir().unwrap();
        db::initialize_in_dir(a.path()).unwrap(); db::initialize_in_dir(b.path()).unwrap();
        let original = file();
        let prompt = assets::save_prompt(a.path(), None, "notes", "body", None, None, &[original.clone()]).unwrap();
        let ordinary = export_sync_changes(a.path()).unwrap();
        assert!(ordinary.iter().all(|item| item.payload.get("assets").is_none()));
        let complete = export_with_assets(a.path(), true).unwrap();
        apply_sync_changes(b.path(), &complete, false).unwrap();
        assert!(assets::list(b.path(), &prompt.id).unwrap().is_empty());
        // The text already has exactly the same version: file hydration must still work.
        apply_with_assets(b.path(), &complete, false, false, true).unwrap();
        let restored = assets::list(b.path(), &prompt.id).unwrap();
        assert_eq!(restored.len(), 1); assert_eq!(restored[0].data, original.data);
        apply_with_assets(b.path(), &complete, false, false, true).unwrap();
        assert_eq!(assets::list(b.path(), &prompt.id).unwrap().len(), 1);
        assert_eq!(db::list_prompts_in_dir(b.path(), "", None).unwrap()[0].asset_count, 1);
    }

    #[test]
    fn merge_keeps_local_files_and_rolls_back_all_files_when_any_record_is_invalid() {
        let dir = tempfile::tempdir().unwrap(); db::initialize_in_dir(dir.path()).unwrap();
        let local = file(); let addition = file();
        let prompt = assets::save_prompt(dir.path(), None, "original", "body", None, None, &[local.clone()]).unwrap();
        let mut change = export_sync_changes(dir.path()).unwrap().into_iter().find(|item| item.id == prompt.id).unwrap();
        change.payload["assets"] = json!([addition.clone()]);
        apply_with_assets(dir.path(), &[change.clone()], true, false, true).unwrap();
        assert_eq!(assets::list(dir.path(), &prompt.id).unwrap().len(), 1);
        let mut bad = change.clone(); bad.id = "bad".into(); bad.payload["category_id"] = json!("missing");
        assert!(apply_with_assets(dir.path(), &[change.clone(), bad], false, false, true).is_err());
        assert_eq!(assets::list(dir.path(), &prompt.id).unwrap().len(), 1);
        apply_with_assets(dir.path(), &[change.clone()], false, false, true).unwrap();
        let merged = assets::list(dir.path(), &prompt.id).unwrap();
        assert_eq!(merged.len(), 2); assert_eq!(merged[0].id, local.id); assert_eq!(merged[1].id, addition.id);
        change.payload["assets"] = json!([]);
        apply_with_assets(dir.path(), &[change.clone()], false, false, true).unwrap();
        assert_eq!(assets::list(dir.path(), &prompt.id).unwrap().len(), 2);
        change.updated_at = (db::timestamp_ms(&change.updated_at) + 1).to_string();
        change.payload["title"] = json!("must not land");
        change.payload["assets"] = json!([{"id": uuid::Uuid::new_v4().to_string(), "name":"bad.txt", "mime":"text/plain", "data":"not base64"}]);
        assert!(apply_with_assets(dir.path(), &[change], false, false, true).is_err());
        assert_eq!(db::list_prompts_in_dir(dir.path(), "", None).unwrap()[0].title, "original");
        assert_eq!(assets::list(dir.path(), &prompt.id).unwrap().len(), 2);
    }
}
