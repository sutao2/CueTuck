use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::Path;

#[derive(Serialize)]
pub struct VersionSummary {
    pub id: String,
    pub title: String,
    pub saved_at: String,
}

pub(crate) fn preserve(connection: &Connection, id: &str, incoming: &Value, deleted: bool) -> Result<(), String> {
    let old = connection.query_row("SELECT title,content,category_id,model FROM prompts WHERE id=?1 AND deleted_at IS NULL", [id], |row| Ok(json!({
        "title": row.get::<_, String>(0)?, "content": row.get::<_, String>(1)?,
        "category_id": row.get::<_, Option<String>>(2)?, "model": row.get::<_, Option<String>>(3)?,
    }))).optional().map_err(|e| e.to_string())?;
    let Some(mut payload) = old else { return Ok(()); };
    if !deleted && ["title", "content"].iter().all(|key| incoming.get(*key).is_none_or(|value| value == &payload[*key])) { return Ok(()); }
    payload["assets"] = serde_json::to_value(super::assets::read(connection, id)?).map_err(|e| e.to_string())?;
    connection.execute("INSERT INTO local_prompt_history(id,title,saved_at,payload_json) VALUES(?1,?2,?3,?4)",
        params![uuid::Uuid::new_v4().to_string(), payload["title"].as_str(), super::now_millis(), payload.to_string()]).map_err(|e| e.to_string())?;
    connection.execute("DELETE FROM local_prompt_history WHERE id NOT IN (SELECT id FROM local_prompt_history ORDER BY CAST(saved_at AS INTEGER) DESC,rowid DESC LIMIT 50)", []).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list(dir: &Path) -> Result<Vec<VersionSummary>, String> {
    let connection = Connection::open(dir.join("promptark.sqlite")).map_err(|e| e.to_string())?;
    let mut statement = connection.prepare("SELECT id,title,saved_at FROM local_prompt_history ORDER BY CAST(saved_at AS INTEGER) DESC,rowid DESC LIMIT 50").map_err(|e| e.to_string())?;
    let rows = statement.query_map([], |row| Ok(VersionSummary { id: row.get(0)?, title: row.get(1)?, saved_at: row.get(2)? }))
        .map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

fn read(connection: &Connection, id: &str) -> Result<Value, String> {
    let text: String = connection.query_row("SELECT payload_json FROM local_prompt_history WHERE id=?1", [id], |row| row.get(0)).map_err(|_| "版本不存在或已超出保留范围")?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn detail(dir: &Path, id: &str) -> Result<Value, String> {
    let connection = Connection::open(dir.join("promptark.sqlite")).map_err(|e| e.to_string())?;
    read(&connection, id)
}

pub fn restore_copy(dir: &Path, version_id: &str) -> Result<String, String> {
    let mut connection = Connection::open(dir.join("promptark.sqlite")).map_err(|e| e.to_string())?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let payload = read(&transaction, version_id)?;
    let title = payload["title"].as_str().filter(|s| !s.trim().is_empty()).ok_or("版本标题无效")?;
    let content = payload["content"].as_str().ok_or("版本正文无效")?;
    let assets: Vec<super::assets::Asset> = serde_json::from_value(payload["assets"].clone()).map_err(|_| "版本附件无效")?;
    let category = payload["category_id"].as_str();
    let category_exists: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM categories WHERE id=?1 AND deleted_at IS NULL)", [category], |row| row.get(0)).map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = super::now_millis();
    transaction.execute("INSERT INTO prompts(id,title,content,category_id,model,source,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,'local',?6,?6)",
        params![id, format!("{title}（同步前版本）"), content, if category_exists { category } else { None }, payload["model"].as_str(), now]).map_err(|e| e.to_string())?;
    super::assets::replace(&transaction, &id, &assets)?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_database as db;
    #[test]
    fn overwritten_versions_and_files_restore_as_copies_without_changing_sync_winner() {
        let dir = tempfile::tempdir().unwrap(); db::initialize_in_dir(dir.path()).unwrap();
        let prompt = db::create_prompt_in_dir(dir.path(), "标题", "本机正文", None).unwrap();
        let connection = Connection::open(dir.path().join("promptark.sqlite")).unwrap();
        db::assets::replace(&connection, &prompt.id, &[db::assets::Asset { id: uuid::Uuid::new_v4().to_string(), name: "a.txt".into(), mime: "text/plain".into(), data: "aGVsbG8=".into() }]).unwrap();
        let mut incoming = db::export_sync_changes(dir.path()).unwrap().into_iter().find(|r| r.id == prompt.id).unwrap();
        incoming.updated_at = (incoming.updated_at.parse::<u64>().unwrap() + 1000).to_string(); incoming.payload["content"] = json!("远端正文");
        db::apply_sync_changes(dir.path(), &[incoming.clone()], true).unwrap();
        assert!(list(dir.path()).unwrap().is_empty());
        let invalid = db::SyncChange { id: "invalid".into(), kind: "prompt".into(), updated_at: incoming.updated_at.clone(), deleted_at: None, payload: json!({"title":"bad","content":"bad","category_id":"missing"}) };
        assert!(db::apply_sync_changes(dir.path(), &[incoming.clone(),invalid], false).is_err());
        assert!(list(dir.path()).unwrap().is_empty());
        db::apply_sync_changes(dir.path(), &[incoming.clone()], false).unwrap();
        db::apply_sync_changes(dir.path(), &[incoming], false).unwrap();
        let versions = list(dir.path()).unwrap(); assert_eq!(versions.len(), 1);
        let copy = restore_copy(dir.path(), &versions[0].id).unwrap();
        let rows = db::list_prompts_in_dir(dir.path(), "", None).unwrap();
        assert_eq!(rows.iter().find(|p| p.id == prompt.id).unwrap().content, "远端正文");
        assert_eq!(rows.iter().find(|p| p.id == copy).unwrap().content, "本机正文");
        assert_eq!(db::assets::list(dir.path(), &copy).unwrap()[0].data, "aGVsbG8=");
        assert!(db::export_sync_changes(dir.path()).unwrap().iter().all(|r| r.kind != "history"));
    }
}
