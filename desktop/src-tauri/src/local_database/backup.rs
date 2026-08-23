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

pub fn export_library_zip_in_dir(dir: &Path, dest: &Path) -> Result<String, String> {
    let json = super::export_library_json_in_dir(dir)?;
    let sqlite = dir.join(LIVE_NAME);
    let sqlite_bytes = if sqlite.exists() {
        std::fs::read(&sqlite).map_err(|error| error.to_string())?
    } else {
        Vec::new()
    };
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
    }
    write_store_zip(
        dest,
        &[
            ("library.json", json.as_bytes()),
            ("promptark.sqlite", &sqlite_bytes),
        ],
    )?;
    Ok(dest.to_string_lossy().into_owned())
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
