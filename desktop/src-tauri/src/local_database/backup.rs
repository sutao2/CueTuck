use rusqlite::Connection;
use std::path::Path;

const LIVE_NAME: &str = "promptark.sqlite";
const BAK_NAME: &str = "promptark.sqlite.restore-bak";

pub fn backup_library_in_dir(dir: &Path, dest: &Path) -> Result<String, String> {
    let src = dir.join(LIVE_NAME);
    if !src.exists() {
        return Err("本地库不存在".to_string());
    }
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
    }
    std::fs::copy(&src, dest).map_err(|error| error.to_string())?;
    Ok(dest.to_string_lossy().into_owned())
}

pub fn restore_library_in_dir(dir: &Path, src: &Path) -> Result<(), String> {
    validate_library_file(src)?;
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let live = dir.join(LIVE_NAME);
    let bak = dir.join(BAK_NAME);
    if live.exists() {
        std::fs::copy(&live, &bak).map_err(|error| error.to_string())?;
    }
    if let Err(error) = std::fs::copy(src, &live) {
        rollback_live(&live, &bak);
        return Err(error.to_string());
    }
    if let Err(error) = validate_library_file(&live) {
        rollback_live(&live, &bak);
        return Err(error);
    }
    let _ = std::fs::remove_file(&bak);
    Ok(())
}

fn validate_library_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("备份文件不存在".to_string());
    }
    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'prompts'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if count < 1 {
        return Err("备份不是有效的本地库".to_string());
    }
    Ok(())
}

fn rollback_live(live: &Path, bak: &Path) {
    if bak.exists() {
        let _ = std::fs::copy(bak, live);
    }
}
