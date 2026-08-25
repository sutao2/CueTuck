use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use super::prompts::PromptRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRecord {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub cover_type: String,
    pub cover_json: String,
    pub member_count: i64,
}

fn open_db(dir: &Path) -> Result<Connection, String> {
    Connection::open(dir.join("promptark.sqlite")).map_err(|error| error.to_string())
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn normalize_cover_json(cover_type: &str, raw: Option<&str>) -> String {
    if cover_type == "none" {
        return "[]".to_string();
    }
    let parsed: Result<Vec<String>, _> = serde_json::from_str(raw.unwrap_or("[]"));
    let mut urls = parsed.unwrap_or_default();
    urls.retain(|url| !url.trim().is_empty());
    let limit = if cover_type == "single" { 1 } else { 9 };
    urls.truncate(limit);
    serde_json::to_string(&urls).unwrap_or_else(|_| "[]".to_string())
}

pub fn create_collection_in_dir(
    dir: &Path,
    title: &str,
    category_id: Option<&str>,
    cover_type: &str,
    cover_json: Option<&str>,
) -> Result<CollectionRecord, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("合集名称不能为空".to_string());
    }
    let cover_type = match cover_type {
        "single" | "grid" => cover_type,
        _ => "none",
    };
    let cover_json = normalize_cover_json(cover_type, cover_json);
    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    let connection = open_db(dir)?;
    connection
        .execute(
            "INSERT INTO collections (id, title, description, category_id, cover_type, cover_json, created_at, updated_at)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?6)",
            rusqlite::params![id, title, category_id, cover_type, cover_json, now],
        )
        .map_err(|error| error.to_string())?;
    Ok(CollectionRecord {
        id,
        title: title.to_string(),
        description: None,
        category_id: category_id.map(str::to_string),
        cover_type: cover_type.to_string(),
        cover_json,
        member_count: 0,
    })
}

pub fn list_collections_in_dir(
    dir: &Path,
    query: &str,
    category_id: Option<&str>,
) -> Result<Vec<CollectionRecord>, String> {
    let connection = open_db(dir)?;
    let pattern = format!("%{}%", query.trim());
    let mut statement = connection
        .prepare(
            "SELECT col.id, col.title, col.description, col.category_id, col.cover_type, col.cover_json,
                    (SELECT COUNT(*) FROM prompts p WHERE p.collection_id = col.id AND p.deleted_at IS NULL)
             FROM collections col
             LEFT JOIN categories c ON c.id = col.category_id
             LEFT JOIN categories parent ON parent.id = c.parent_id
             WHERE col.deleted_at IS NULL
               AND (?1 = '' OR col.title LIKE ?2 OR IFNULL(c.name, '') LIKE ?2 OR IFNULL(parent.name, '') LIKE ?2)
               AND (
                    ?3 IS NULL
                    OR col.category_id = ?3
                    OR c.parent_id = ?3
               )
             ORDER BY col.updated_at DESC, col.title",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(rusqlite::params![query.trim(), pattern, category_id], |row| {
            Ok(CollectionRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                category_id: row.get(3)?,
                cover_type: row.get(4)?,
                cover_json: row.get(5)?,
                member_count: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}

pub fn add_prompt_to_collection_in_dir(
    dir: &Path,
    prompt_id: &str,
    collection_id: &str,
) -> Result<(), String> {
    let connection = open_db(dir)?;
    let changed = connection
        .execute(
            "UPDATE prompts SET collection_id = ?1, updated_at = ?2
             WHERE id = ?3 AND deleted_at IS NULL",
            rusqlite::params![collection_id, now_iso(), prompt_id],
        )
        .map_err(|error| error.to_string())?;
    if changed == 0 {
        return Err("提示词不存在".to_string());
    }
    Ok(())
}

pub fn collection_member_count(dir: &Path, collection_id: &str) -> Result<i64, String> {
    let connection = open_db(dir)?;
    connection
        .query_row(
            "SELECT COUNT(*) FROM prompts WHERE collection_id = ?1 AND deleted_at IS NULL",
            [collection_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())
}

pub fn list_collection_members_in_dir(
    dir: &Path,
    collection_id: &str,
) -> Result<Vec<PromptRecord>, String> {
    let connection = open_db(dir)?;
    let mut statement = connection
        .prepare(
            "SELECT id, title, summary, content, category_id, collection_id, COALESCE(use_count, 0),
                    COALESCE(source, 'local'), author
             FROM prompts
             WHERE deleted_at IS NULL AND collection_id = ?1
             ORDER BY title",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([collection_id], super::prompts::map_prompt_row)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}
