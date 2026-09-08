use std::path::Path;
use tauri::{AppHandle, Manager};

fn validate_program(path: &str) -> Result<String, String> {
    let path = Path::new(path);
    if !path.is_absolute() { return Err("请选择 MCP 程序的绝对路径，不支持 ~ 或相对路径".into()); }
    let canonical = path.canonicalize().map_err(|_| "MCP 程序不存在，请先独立构建")?;
    let metadata = canonical.metadata().map_err(|_| "无法读取 MCP 程序")?;
    if !metadata.is_file() { return Err("MCP 路径必须是可执行文件，不是目录".into()); }
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 { return Err("所选文件没有执行权限".into()); }
    }
    canonical.to_str().map(str::to_owned).ok_or_else(|| "文件路径不是有效 UTF-8".into())
}

#[tauri::command]
pub fn mcp_connection_info(app: AppHandle, executable: String) -> Result<serde_json::Value,String> {
    let command = validate_program(&executable)?;
    let dir = app.path().app_data_dir().map_err(|_| "无法读取本机库目录")?;
    if !dir.join("promptark.sqlite").is_file() { return Err("本机库尚未创建，请先打开本地提示词页".into()); }
    Ok(serde_json::json!({"command":command,"library_dir":dir.to_str().ok_or("库路径不是有效 UTF-8")?}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_paths_without_executing_files() {
        assert!(validate_program("relative").is_err());
        let dir = tempfile::tempdir().unwrap();
        assert!(validate_program(dir.path().to_str().unwrap()).is_err());
        let file = dir.path().join("mcp test");
        assert!(validate_program(file.to_str().unwrap()).is_err());
        std::fs::write(&file,b"not executed").unwrap();
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file,std::fs::Permissions::from_mode(0o600)).unwrap();
            assert!(validate_program(file.to_str().unwrap()).is_err());
            std::fs::set_permissions(&file,std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        assert_eq!(validate_program(file.to_str().unwrap()).unwrap(),file.canonicalize().unwrap().to_str().unwrap());
    }
}
