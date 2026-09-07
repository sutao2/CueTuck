use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRecord {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub is_system: bool,
    pub sort_order: i64,
}

pub fn list_categories_in_dir(dir: &Path) -> Result<Vec<CategoryRecord>, String> {
    let connection =
        Connection::open(dir.join("promptark.sqlite")).map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT id, parent_id, name, icon, is_system, sort_order
             FROM categories WHERE deleted_at IS NULL
             ORDER BY CASE WHEN parent_id IS NULL THEN 0 ELSE 1 END, sort_order, name",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(CategoryRecord {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                icon: row.get(3)?,
                is_system: row.get::<_, i64>(4)? == 1,
                sort_order: row.get(5)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}

pub fn create_category_in_dir(
    dir: &Path,
    name: &str,
    parent_id: Option<&str>,
) -> Result<CategoryRecord, String> {
    let title = name.trim();
    if title.is_empty() {
        return Err("分类名称不能为空".to_string());
    }
    let mut connection =
        Connection::open(dir.join("promptark.sqlite")).map_err(|error| error.to_string())?;
    let connection = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    if let Some(parent_id) = parent_id {
        let parent: (Option<String>,) = connection
            .query_row(
                "SELECT parent_id FROM categories WHERE id = ?1 AND deleted_at IS NULL",
                [parent_id],
                |row| Ok((row.get(0)?,)),
            )
            .map_err(|_| "大分类不存在".to_string())?;
        if parent.0.is_some() {
            return Err("小分类下不能再创建子分类".to_string());
        }
    }
    let duplicate: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM categories WHERE parent_id IS ?1 AND trim(name)=?2 AND deleted_at IS NULL)",
        rusqlite::params![parent_id, title], |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if duplicate {
        return Err("同一级已有同名分类".into());
    }
    let sort_order: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM categories WHERE parent_id IS ?1",
            [parent_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let id = format!("cat-user-{}", Uuid::new_v4());
    connection
        .execute(
            "INSERT INTO categories (id, parent_id, name, icon, is_system, sort_order, updated_at)
             VALUES (?1, ?2, ?3, NULL, 0, ?4, ?5)",
            rusqlite::params![id, parent_id, title, sort_order, super::now_millis()],
        )
        .map_err(|error| error.to_string())?;
    connection.commit().map_err(|e| e.to_string())?;
    Ok(CategoryRecord {
        id,
        parent_id: parent_id.map(str::to_string),
        name: title.to_string(),
        icon: None,
        is_system: false,
        sort_order,
    })
}

pub fn delete_category_in_dir(dir: &Path, id: &str) -> Result<(), String> {
    let mut connection =
        Connection::open(dir.join("promptark.sqlite")).map_err(|e| e.to_string())?;
    let transaction = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    let system: bool = transaction
        .query_row(
            "SELECT is_system FROM categories WHERE id=?1 AND deleted_at IS NULL",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| "分类不存在".to_string())?;
    if system {
        return Err("系统分类不能删除".into());
    }
    let has_children: bool = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE parent_id=?1 AND deleted_at IS NULL)",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if has_children {
        return Err("请先删除该大分类下的小分类".into());
    }
    let timestamp = super::now_millis();
    transaction
        .execute(
            "UPDATE categories SET deleted_at=?2, updated_at=?2 WHERE id=?1",
            rusqlite::params![id, timestamp],
        )
        .map_err(|e| e.to_string())?;
    for table in ["prompts", "collections"] {
        transaction
            .execute(
                &format!("UPDATE {table} SET category_id=NULL, updated_at=?2 WHERE category_id=?1"),
                rusqlite::params![id, timestamp],
            )
            .map_err(|e| e.to_string())?;
    }
    transaction.commit().map_err(|e| e.to_string())
}
