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
    let connection = Connection::open(dir.join("promptark.sqlite")).map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT id, parent_id, name, icon, is_system, sort_order
             FROM categories
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
    parent_id: &str,
) -> Result<CategoryRecord, String> {
    let title = name.trim();
    if title.is_empty() {
        return Err("分类名称不能为空".to_string());
    }
    let connection = Connection::open(dir.join("promptark.sqlite")).map_err(|error| error.to_string())?;
    let parent: (Option<String>,) = connection
        .query_row(
            "SELECT parent_id FROM categories WHERE id = ?1",
            [parent_id],
            |row| Ok((row.get(0)?,)),
        )
        .map_err(|_| "大分类不存在".to_string())?;
    if parent.0.is_some() {
        return Err("小分类下不能再创建子分类".to_string());
    }
    let sort_order: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM categories WHERE parent_id = ?1",
            [parent_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let id = format!("cat-user-{}", Uuid::new_v4());
    connection
        .execute(
            "INSERT INTO categories (id, parent_id, name, icon, is_system, sort_order)
             VALUES (?1, ?2, ?3, NULL, 0, ?4)",
            rusqlite::params![id, parent_id, title, sort_order],
        )
        .map_err(|error| error.to_string())?;
    Ok(CategoryRecord {
        id,
        parent_id: Some(parent_id.to_string()),
        name: title.to_string(),
        icon: None,
        is_system: false,
        sort_order,
    })
}
