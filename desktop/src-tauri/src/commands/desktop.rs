use std::fs;
use std::path::Path;
use tauri::AppHandle;

const AGENT_NAME: &str = "app.promptark.desktop.login.plist";

pub fn launch_agent_plist(program: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>app.promptark.desktop.login</string>
  <key>ProgramArguments</key>
  <array>
    <string>{program}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
"#
    )
}

pub fn apply_launch_agent(dir: &Path, enabled: bool, program: &str) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = dir.join(AGENT_NAME);
    if enabled {
        fs::write(&path, launch_agent_plist(program)).map_err(|error| error.to_string())?;
    } else if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn apply_launch_at_login(enabled: bool) -> Result<(), String> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = enabled;
        return Err("当前系统尚未验证开机启动，不会声称已生效".into());
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").map_err(|error| error.to_string())?;
        let dir = Path::new(&home).join("Library/LaunchAgents");
        let program = std::env::current_exe()
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .into_owned();
        apply_launch_agent(&dir, enabled, &program)
    }
}

#[tauri::command]
pub fn apply_minimize_to_tray(_app: AppHandle, enabled: bool) -> Result<(), String> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = enabled;
        return Err("当前系统尚未验证托盘，不会声称已生效".into());
    }
    #[cfg(target_os = "macos")]
    {
        let _ = enabled;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn writes_and_removes_launch_agent() {
        let dir = tempdir().unwrap();
        apply_launch_agent(dir.path(), true, "/tmp/PromptArk.app").unwrap();
        let path = dir.path().join(AGENT_NAME);
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("app.promptark.desktop.login"));
        assert!(body.contains("/tmp/PromptArk.app"));
        apply_launch_agent(dir.path(), false, "/tmp/PromptArk.app").unwrap();
        assert!(!path.exists());
    }
}
