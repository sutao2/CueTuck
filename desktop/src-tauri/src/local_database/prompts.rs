use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRecord {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub content: String,
    pub category_id: Option<String>,
    pub collection_id: Option<String>,
    pub use_count: i64,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
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

pub(crate) fn map_prompt_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PromptRecord> {
    Ok(PromptRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        summary: row.get(2)?,
        content: row.get(3)?,
        category_id: row.get(4)?,
        collection_id: row.get(5)?,
        use_count: row.get(6)?,
        source: row.get(7)?,
        author: row.get(8)?,
    })
}

fn read_prompt(connection: &Connection, id: &str) -> Result<PromptRecord, String> {
    connection
        .query_row(
            "SELECT id, title, summary, content, category_id, collection_id, COALESCE(use_count, 0),
                    COALESCE(source, 'local'), author
             FROM prompts WHERE id = ?1",
            [id],
            map_prompt_row,
        )
        .map_err(|error| error.to_string())
}

pub fn create_prompt_in_dir(
    dir: &Path,
    title: &str,
    content: &str,
    category_id: Option<&str>,
) -> Result<PromptRecord, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("标题不能为空".to_string());
    }
    let now = now_iso();
    let id = Uuid::new_v4().to_string();
    let connection = open_db(dir)?;
    connection
        .execute(
            "INSERT INTO prompts (
                id, title, summary, content, category_id, source, version, use_count, created_at, updated_at
            ) VALUES (?1, ?2, NULL, ?3, ?4, 'local', 1, 0, ?5, ?5)",
            rusqlite::params![id, title, content, category_id, now],
        )
        .map_err(|error| error.to_string())?;
    read_prompt(&connection, &id)
}

pub fn import_downloaded_prompt_in_dir(
    dir: &Path,
    title: &str,
    content: &str,
    remote_id: Option<&str>,
    author: Option<&str>,
) -> Result<PromptRecord, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("标题不能为空".to_string());
    }
    let now = now_iso();
    let id = Uuid::new_v4().to_string();
    let author = author.map(str::trim).filter(|value| !value.is_empty());
    let connection = open_db(dir)?;
    connection
        .execute(
            "INSERT INTO prompts (
                id, title, summary, content, category_id, source, remote_id, version, use_count, created_at, updated_at, author
            ) VALUES (?1, ?2, NULL, ?3, NULL, 'downloaded', ?4, 1, 0, ?5, ?5, ?6)",
            rusqlite::params![id, title, content, remote_id, now, author],
        )
        .map_err(|error| error.to_string())?;
    read_prompt(&connection, &id)
}

pub fn upsert_synced_prompt_in_dir(
    dir: &Path,
    id: &str,
    title: &str,
    content: &str,
    category_id: Option<&str>,
    updated_at: &str,
) -> Result<PromptRecord, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("标题不能为空".to_string());
    }
    let connection = open_db(dir)?;
    let exists = connection
        .query_row("SELECT id FROM prompts WHERE id = ?1", [id], |row| {
            row.get::<_, String>(0)
        })
        .ok();
    if exists.is_some() {
        return read_prompt(&connection, id);
    }
    let stamp = if updated_at.trim().is_empty() {
        now_iso()
    } else {
        updated_at.to_string()
    };
    connection
        .execute(
            "INSERT INTO prompts (
                id, title, summary, content, category_id, source, version, use_count, created_at, updated_at
            ) VALUES (?1, ?2, NULL, ?3, ?4, 'local', 1, 0, ?5, ?5)",
            rusqlite::params![id, title, content, category_id, stamp],
        )
        .map_err(|error| error.to_string())?;
    read_prompt(&connection, id)
}

pub fn update_prompt_in_dir(
    dir: &Path,
    id: &str,
    title: &str,
    content: &str,
    category_id: Option<&str>,
) -> Result<PromptRecord, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("标题不能为空".to_string());
    }
    let connection = open_db(dir)?;
    let changed = connection
        .execute(
            "UPDATE prompts SET title = ?1, content = ?2, category_id = ?3, updated_at = ?4
             WHERE id = ?5 AND deleted_at IS NULL",
            rusqlite::params![title, content, category_id, now_iso(), id],
        )
        .map_err(|error| error.to_string())?;
    if changed == 0 {
        return Err("提示词不存在".to_string());
    }
    read_prompt(&connection, id)
}

pub fn delete_prompt_in_dir(dir: &Path, id: &str) -> Result<(), String> {
    let connection = open_db(dir)?;
    let changed = connection
        .execute(
            "UPDATE prompts SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
            rusqlite::params![now_iso(), id],
        )
        .map_err(|error| error.to_string())?;
    if changed == 0 {
        return Err("提示词不存在".to_string());
    }
    Ok(())
}

pub fn prompt_deleted_at(dir: &Path, id: &str) -> Result<Option<String>, String> {
    let connection = open_db(dir)?;
    connection
        .query_row(
            "SELECT deleted_at FROM prompts WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())
}

pub fn prompt_use_count(dir: &Path, id: &str) -> Result<i64, String> {
    Ok(read_prompt(&open_db(dir)?, id)?.use_count)
}

pub fn clear_prompt_use_in_dir(dir: &Path) -> Result<(), String> {
    open_db(dir)?
        .execute("UPDATE prompts SET use_count = 0 WHERE deleted_at IS NULL", [])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn record_prompt_use_in_dir(dir: &Path, id: &str) -> Result<PromptRecord, String> {
    let connection = open_db(dir)?;
    let changed = connection
        .execute(
            "UPDATE prompts
             SET use_count = COALESCE(use_count, 0) + 1, last_used_at = ?1, updated_at = ?1
             WHERE id = ?2 AND deleted_at IS NULL",
            rusqlite::params![now_iso(), id],
        )
        .map_err(|error| error.to_string())?;
    if changed == 0 {
        return Err("提示词不存在".to_string());
    }
    read_prompt(&connection, id)
}

pub fn list_prompts_in_dir(
    dir: &Path,
    query: &str,
    category_id: Option<&str>,
) -> Result<Vec<PromptRecord>, String> {
    let connection = open_db(dir)?;
    let pattern = format!("%{}%", query.trim());
    let mut statement = connection
        .prepare(
            "SELECT p.id, p.title, p.summary, p.content, p.category_id, p.collection_id, COALESCE(p.use_count, 0),
                    COALESCE(p.source, 'local'), p.author
             FROM prompts p
             LEFT JOIN categories c ON c.id = p.category_id
             LEFT JOIN categories parent ON parent.id = c.parent_id
             WHERE p.deleted_at IS NULL
               AND (?1 = '' OR p.title LIKE ?2 OR p.content LIKE ?2 OR IFNULL(c.name, '') LIKE ?2 OR IFNULL(parent.name, '') LIKE ?2)
               AND (
                    ?3 IS NULL
                    OR p.category_id = ?3
                    OR c.parent_id = ?3
               )
             ORDER BY p.updated_at DESC, p.title",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(rusqlite::params![query.trim(), pattern, category_id], map_prompt_row)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}
