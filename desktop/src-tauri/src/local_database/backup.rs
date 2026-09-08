use rusqlite::{backup::{Backup, StepResult}, Connection, OpenFlags};
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const LIVE_NAME: &str = "promptark.sqlite";
static FILE_OPERATIONS: Mutex<()> = Mutex::new(());

pub fn backup_library_in_dir(dir: &Path, dest: &Path) -> Result<String, String> {
    let _guard = FILE_OPERATIONS.lock().map_err(|error| error.to_string())?;
    let src = dir.join(LIVE_NAME);
    reject_live_path(&src, dest)?;
    let file = tempfile::NamedTempFile::new_in(destination_parent(dest)?).map_err(|error| error.to_string())?;
    snapshot(&src, file.path())?;
    file.as_file().sync_all().map_err(|error| error.to_string())?;
    file.persist(dest).map_err(|error| error.to_string())?;
    Ok(dest.to_string_lossy().into_owned())
}

pub fn restore_library_in_dir(dir: &Path, src: &Path) -> Result<(), String> {
    let _guard = FILE_OPERATIONS.lock().map_err(|error| error.to_string())?;
    let live = dir.join(LIVE_NAME);
    reject_live_path(&live, src)?;
    validate_library_file(src)?;
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let staged = tempfile::tempdir_in(dir).map_err(|error| error.to_string())?;
    let candidate = staged.path().join(LIVE_NAME);
    snapshot(src, &candidate)?;
    super::initialize_in_dir(staged.path())?;
    validate_library_file(&candidate)?;
    // Reading every supported table/column rejects incompatible schemas before touching live data.
    super::export_sync_changes(staged.path())?;
    validate_references(&candidate)?;
    snapshot(&candidate, &live)?;
    Ok(())
}

pub fn export_library_zip_in_dir(dir: &Path, dest: &Path) -> Result<String, String> {
    let _guard = FILE_OPERATIONS.lock().map_err(|error| error.to_string())?;
    reject_live_path(&dir.join(LIVE_NAME), dest)?;
    let parent = destination_parent(dest)?;
    let staged = tempfile::tempdir_in(parent).map_err(|error| error.to_string())?;
    let sqlite = staged.path().join(LIVE_NAME);
    snapshot(&dir.join(LIVE_NAME), &sqlite)?;
    let json = super::export_library_json_in_dir(staged.path())?;
    let sqlite_bytes = std::fs::read(&sqlite).map_err(|error| error.to_string())?;
    let file = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    write_store_zip(
        file.path(),
        &[
            ("library.json", json.as_bytes()),
            ("promptark.sqlite", &sqlite_bytes),
        ],
    )?;
    file.as_file().sync_all().map_err(|error| error.to_string())?;
    file.persist(dest).map_err(|error| error.to_string())?;
    Ok(dest.to_string_lossy().into_owned())
}

fn destination_parent(dest: &Path) -> Result<&Path, String> {
    let parent = dest.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    Ok(parent)
}

fn reject_live_path(live: &Path, other: &Path) -> Result<(), String> {
    if live.exists() && other.exists() && live.canonicalize().map_err(|e| e.to_string())? == other.canonicalize().map_err(|e| e.to_string())? {
        return Err("备份或恢复路径不能是当前库文件".into());
    }
    Ok(())
}

fn snapshot(src: &Path, dest: &Path) -> Result<(), String> {
    let source = Connection::open_with_flags(src, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|error| error.to_string())?;
    let mut target = Connection::open(dest).map_err(|error| error.to_string())?;
    target.busy_timeout(Duration::from_millis(100)).map_err(|error| error.to_string())?;
    let backup = Backup::new(&source, &mut target).map_err(|error| error.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match backup.step(256).map_err(|error| error.to_string())? {
            StepResult::Done => return Ok(()),
            _ if Instant::now() >= deadline => return Err("数据库忙，备份或恢复未完成，请稍后重试".into()),
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

fn validate_references(path: &Path) -> Result<(), String> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|error| error.to_string())?;
    for query in [
        "SELECT COUNT(*) FROM prompts p WHERE p.deleted_at IS NULL AND (p.id IS NULL OR trim(p.title) = '' OR p.content IS NULL OR (p.collection_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM collections c WHERE c.id = p.collection_id AND c.deleted_at IS NULL)) OR (p.category_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM categories c WHERE c.id = p.category_id)))",
        "SELECT COUNT(*) FROM collections c WHERE c.deleted_at IS NULL AND (c.id IS NULL OR trim(c.title) = '' OR (c.category_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM categories cat WHERE cat.id = c.category_id)))",
    ] {
        let invalid: i64 = connection.query_row(query, [], |row| row.get(0)).map_err(|error| error.to_string())?;
        if invalid != 0 { return Err("备份包含无效记录或成员引用".into()); }
    }
    let mut statement = connection.prepare("SELECT DISTINCT prompt_id FROM prompt_assets").map_err(|e| e.to_string())?;
    let owners = statement.query_map([], |row| row.get::<_, String>(0)).map_err(|e| e.to_string())?;
    for owner in owners {
        let owner = owner.map_err(|e| e.to_string())?;
        let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM prompts WHERE id=?1)", [&owner], |row| row.get(0)).map_err(|e| e.to_string())?;
        if !exists { return Err("附件归属无效".into()); }
        super::assets::validate(&super::assets::read(&connection, &owner)?)?;
    }
    Ok(())
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn write_store_zip(dest: &Path, files: &[(&str, &[u8])]) -> Result<(), String> {
    let mut output = Vec::new();
    let mut directory = Vec::new();
    for (name, data) in files {
        let name_bytes = name.as_bytes();
        let crc = crc32(data);
        let offset = output.len() as u32;
        output.extend_from_slice(&0x04034b50u32.to_le_bytes());
        output.extend_from_slice(&20u16.to_le_bytes());
        output.extend_from_slice(&0u16.to_le_bytes());
        output.extend_from_slice(&0u16.to_le_bytes());
        output.extend_from_slice(&0u16.to_le_bytes());
        output.extend_from_slice(&0u16.to_le_bytes());
        output.extend_from_slice(&crc.to_le_bytes());
        output.extend_from_slice(&(data.len() as u32).to_le_bytes());
        output.extend_from_slice(&(data.len() as u32).to_le_bytes());
        output.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        output.extend_from_slice(&0u16.to_le_bytes());
        output.extend_from_slice(name_bytes);
        output.extend_from_slice(data);
        directory.extend_from_slice(&0x02014b50u32.to_le_bytes());
        directory.extend_from_slice(&20u16.to_le_bytes());
        directory.extend_from_slice(&20u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&crc.to_le_bytes());
        directory.extend_from_slice(&(data.len() as u32).to_le_bytes());
        directory.extend_from_slice(&(data.len() as u32).to_le_bytes());
        directory.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&0u32.to_le_bytes());
        directory.extend_from_slice(&offset.to_le_bytes());
        directory.extend_from_slice(name_bytes);
    }
    let dir_offset = output.len() as u32;
    output.extend_from_slice(&directory);
    output.extend_from_slice(&0x06054b50u32.to_le_bytes());
    output.extend_from_slice(&0u16.to_le_bytes());
    output.extend_from_slice(&0u16.to_le_bytes());
    output.extend_from_slice(&(files.len() as u16).to_le_bytes());
    output.extend_from_slice(&(files.len() as u16).to_le_bytes());
    output.extend_from_slice(&(directory.len() as u32).to_le_bytes());
    output.extend_from_slice(&dir_offset.to_le_bytes());
    output.extend_from_slice(&0u16.to_le_bytes());
    std::fs::write(dest, output).map_err(|error| error.to_string())
}

fn validate_library_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("备份文件不存在".to_string());
    }
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|error| error.to_string())?;
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0)).map_err(|error| error.to_string())?;
    if integrity != "ok" {
        return Err("备份不是有效的本地库".to_string());
    }
    connection.prepare("SELECT id, title, content FROM prompts").map_err(|_| "备份缺少提示词必要字段".to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_database::{initialize_in_dir, create_prompt_in_dir, list_prompts_in_dir};

    #[test]
    fn backup_and_restore_include_wal_and_work_with_an_open_connection() {
        let dir = tempfile::tempdir().unwrap();
        initialize_in_dir(dir.path()).unwrap();
        let live = dir.path().join(LIVE_NAME);
        let writer = Connection::open(&live).unwrap();
        writer.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0;").unwrap();
        create_prompt_in_dir(dir.path(), "WAL新提交", "最新正文", None).unwrap();
        writer.execute("UPDATE prompts SET content = 'WAL最新正文'", []).unwrap();
        assert!(dir.path().join("promptark.sqlite-wal").metadata().unwrap().len() > 0);
        let dest = dir.path().join("backup.sqlite");
        backup_library_in_dir(dir.path(), &dest).unwrap();
        create_prompt_in_dir(dir.path(), "备份后", "不应恢复", None).unwrap();
        restore_library_in_dir(dir.path(), &dest).unwrap();
        let count: i64 = writer.query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
        assert_eq!(list_prompts_in_dir(dir.path(), "", None).unwrap()[0].content, "WAL最新正文");
        assert!(backup_library_in_dir(dir.path(), &live).is_err());
        assert!(restore_library_in_dir(dir.path(), &live).is_err());
    }

    #[test]
    fn restores_legacy_columns_on_a_copy_and_rejects_bad_schema_and_references() {
        let dir = tempfile::tempdir().unwrap();
        initialize_in_dir(dir.path()).unwrap();
        let legacy = dir.path().join("legacy.sqlite");
        let old = Connection::open(&legacy).unwrap();
        old.execute_batch("CREATE TABLE prompts (id TEXT PRIMARY KEY, title TEXT NOT NULL, content TEXT); INSERT INTO prompts VALUES ('old', '旧库', '旧正文');").unwrap();
        drop(old);
        let original = std::fs::read(&legacy).unwrap();
        restore_library_in_dir(dir.path(), &legacy).unwrap();
        assert_eq!(std::fs::read(&legacy).unwrap(), original);
        assert_eq!(list_prompts_in_dir(dir.path(), "", None).unwrap()[0].title, "旧库");
        let invalid = dir.path().join("invalid.sqlite");
        let bad = Connection::open(&invalid).unwrap();
        bad.execute_batch("CREATE TABLE prompts (unrelated TEXT)").unwrap();
        drop(bad);
        assert!(restore_library_in_dir(dir.path(), &invalid).is_err());
        let broken = tempfile::tempdir().unwrap();
        initialize_in_dir(broken.path()).unwrap();
        create_prompt_in_dir(broken.path(), "悬空归属", "正文", Some("missing")).unwrap();
        assert!(restore_library_in_dir(dir.path(), &broken.path().join(LIVE_NAME)).is_err());
        assert_eq!(list_prompts_in_dir(dir.path(), "", None).unwrap()[0].title, "旧库");
    }

    #[test]
    fn locked_restore_rolls_back_and_failed_backup_preserves_existing_target() {
        let dir = tempfile::tempdir().unwrap();
        initialize_in_dir(dir.path()).unwrap();
        create_prompt_in_dir(dir.path(), "原始", "正文", None).unwrap();
        let dest = dir.path().join("backup.sqlite");
        backup_library_in_dir(dir.path(), &dest).unwrap();
        create_prompt_in_dir(dir.path(), "新增", "正文", None).unwrap();
        let writer = Connection::open(dir.path().join(LIVE_NAME)).unwrap();
        writer.execute_batch("BEGIN IMMEDIATE").unwrap();
        assert!(restore_library_in_dir(dir.path(), &dest).is_err());
        writer.execute_batch("ROLLBACK").unwrap();
        assert_eq!(list_prompts_in_dir(dir.path(), "", None).unwrap().len(), 2);
        let original = std::fs::read(&dest).unwrap();
        let missing = tempfile::tempdir().unwrap();
        assert!(backup_library_in_dir(missing.path(), &dest).is_err());
        assert_eq!(std::fs::read(&dest).unwrap(), original);
    }

    #[test]
    fn zip_contains_matching_json_and_sqlite_with_wal_data() {
        let dir = tempfile::tempdir().unwrap();
        initialize_in_dir(dir.path()).unwrap();
        let writer = Connection::open(dir.path().join(LIVE_NAME)).unwrap();
        writer.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0").unwrap();
        create_prompt_in_dir(dir.path(), "ZIP新提交", "WAL正文", None).unwrap();
        writer.execute("UPDATE prompts SET content = 'WAL中尚未检查点的正文'", []).unwrap();
        assert!(dir.path().join("promptark.sqlite-wal").metadata().unwrap().len() > 0);
        let dest = dir.path().join("backup.zip");
        export_library_zip_in_dir(dir.path(), &dest).unwrap();
        let data = std::fs::read(dest).unwrap();
        let mut offset = 0;
        let extracted = tempfile::tempdir().unwrap();
        let mut json = serde_json::Value::Null;
        // The ZIP writer uses stored entries; inspect their payloads, not merely file existence.
        for _ in 0..2 {
            let length = u32::from_le_bytes(data[offset + 18..offset + 22].try_into().unwrap()) as usize;
            let name_len = u16::from_le_bytes(data[offset + 26..offset + 28].try_into().unwrap()) as usize;
            let name = std::str::from_utf8(&data[offset + 30..offset + 30 + name_len]).unwrap();
            let start = offset + 30 + name_len;
            let bytes = &data[start..start + length];
            if name == "library.json" { json = serde_json::from_slice(bytes).unwrap(); }
            if name == LIVE_NAME { std::fs::write(extracted.path().join(LIVE_NAME), bytes).unwrap(); }
            offset = start + length;
        }
        let rows = list_prompts_in_dir(extracted.path(), "", None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].content, "WAL中尚未检查点的正文");
        assert_eq!(json["prompts"][0]["id"].as_str(), Some(rows[0].id.as_str()));
        assert_eq!(json["prompts"][0]["content"], "WAL中尚未检查点的正文");
    }
}
