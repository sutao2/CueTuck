use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct DeletedItem {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub deleted_at: String,
}

pub fn list_deleted(dir: &Path, query: &str) -> Result<Vec<DeletedItem>, String> {
    let connection = Connection::open(dir.join("promptark.sqlite")).map_err(|e| e.to_string())?;
    let mut statement = connection.prepare("SELECT id,kind,title,deleted_at FROM (
        SELECT id,'prompt' AS kind,title,deleted_at FROM prompts WHERE deleted_at IS NOT NULL
        UNION ALL SELECT id,'collection',title,deleted_at FROM collections WHERE deleted_at IS NOT NULL
    ) WHERE instr(lower(title),lower(?1)) > 0 ORDER BY CAST(deleted_at AS INTEGER) DESC LIMIT 200").map_err(|e| e.to_string())?;
    let rows = statement.query_map([query.trim()], |row| Ok(DeletedItem {
        id: row.get(0)?, kind: row.get(1)?, title: row.get(2)?, deleted_at: row.get(3)?,
    })).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn restore_deleted(dir: &Path, id: &str, kind: &str) -> Result<(), String> {
    let table = match kind { "prompt" => "prompts", "collection" => "collections", _ => return Err("不支持的恢复类型".into()) };
    let mut connection = Connection::open(dir.join("promptark.sqlite")).map_err(|e| e.to_string())?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let stamp: String = transaction.query_row(&format!("SELECT updated_at FROM {table} WHERE id=?1 AND deleted_at IS NOT NULL"), [id], |r| r.get(0)).map_err(|_| "条目已恢复或不存在")?;
    let now = super::timestamp_ms(&super::now_millis()).max(super::timestamp_ms(&stamp).saturating_add(1)).to_string();
    transaction.execute(&format!("UPDATE {table} SET deleted_at=NULL,updated_at=?1,category_id=CASE WHEN EXISTS(SELECT 1 FROM categories WHERE id={table}.category_id AND deleted_at IS NULL) THEN category_id ELSE NULL END WHERE id=?2"), params![now,id]).map_err(|e| e.to_string())?;
    if kind == "prompt" {
        transaction.execute("UPDATE prompts SET collection_id=NULL,source=CASE WHEN source='collection' THEN 'local' ELSE source END WHERE id=?1 AND NOT EXISTS(SELECT 1 FROM collections WHERE id=prompts.collection_id AND deleted_at IS NULL)", [id]).map_err(|e| e.to_string())?;
    }
    transaction.commit().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_database as db;
    #[test]
    fn restores_deleted_prompt_with_files_and_does_not_overwrite_live_records() {
        let dir = tempfile::tempdir().unwrap(); db::initialize_in_dir(dir.path()).unwrap();
        let prompt = db::create_prompt_in_dir(dir.path(), "保留文件", "原文", None).unwrap();
        let connection = Connection::open(dir.path().join("promptark.sqlite")).unwrap();
        let asset = db::assets::Asset { id: uuid::Uuid::new_v4().to_string(), name: "note.txt".into(), mime: "text/plain".into(), data: "aGVsbG8=".into() };
        db::assets::replace(&connection, &prompt.id, &[asset]).unwrap();
        db::delete_prompt_in_dir(dir.path(), &prompt.id).unwrap();
        assert_eq!(list_deleted(dir.path(), "文件").unwrap().len(), 1);
        restore_deleted(dir.path(), &prompt.id, "prompt").unwrap();
        assert!(list_deleted(dir.path(), "").unwrap().is_empty());
        assert_eq!(db::assets::list(dir.path(), &prompt.id).unwrap()[0].data, "aGVsbG8=");
        assert!(restore_deleted(dir.path(), &prompt.id, "prompt").is_err());
        assert_eq!(db::list_prompts_in_dir(dir.path(), "", None).unwrap()[0].content, "原文");
    }
    #[test]
    fn restoring_a_collection_does_not_move_its_former_members() {
        let dir = tempfile::tempdir().unwrap(); db::initialize_in_dir(dir.path()).unwrap();
        let collection = db::create_collection_in_dir(dir.path(), "合集", None, "none", None).unwrap();
        let prompt = db::create_prompt_in_dir(dir.path(), "成员", "正文", None).unwrap();
        db::add_prompt_to_collection_in_dir(dir.path(), &prompt.id, &collection.id).unwrap();
        db::delete_collection_in_dir(dir.path(), &collection.id).unwrap();
        restore_deleted(dir.path(), &collection.id, "collection").unwrap();
        assert_eq!(db::collection_member_count(dir.path(), &collection.id).unwrap(), 0);
        assert_eq!(db::list_prompts_in_dir(dir.path(), "", None).unwrap().len(), 1);
        assert!(restore_deleted(dir.path(), &prompt.id, "invalid").is_err());
    }
}
