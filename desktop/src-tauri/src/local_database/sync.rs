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
            let payload = row.map_err(|e| e.to_string())?;
            let id = payload[key].as_str().unwrap_or("").to_string();
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
        for item in items.iter().filter(|item| item.kind == kind) {
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
                let parent = item.payload["parent_id"]
                    .as_str()
                    .ok_or("同步小分类缺少大分类")?;
                let valid: bool = transaction
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM categories WHERE id=?1 AND parent_id IS NULL)",
                        [parent],
                        |row| row.get(0),
                    )
                    .map_err(|e| e.to_string())?;
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
            let mut payload = item.payload.as_object().unwrap().clone();
            payload.insert(key.into(), json!(id));
            payload.insert(
                "updated_at".into(),
                json!(super::timestamp_ms(&item.updated_at).to_string()),
            );
            if kind == "prompt" || kind == "collection" {
                payload.insert("deleted_at".into(), json!(item.deleted_at));
                if !payload.contains_key("title") && item.deleted_at.is_some() {
                    payload.insert("title".into(), json!(""));
                }
            }
            if kind == "category" {
                payload.insert("is_system".into(), json!(0));
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
        }
    }
    transaction.commit().map_err(|e| e.to_string())?;
    for item in items {
        super::observe_timestamp(super::timestamp_ms(&item.updated_at));
    }
    Ok(())
}
