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

#[cfg(any(test, target_os = "windows"))]
pub fn windows_run_value(program: &str) -> String {
    format!("\"{}\"", program.replace('"', ""))
}

#[cfg(any(test, target_os = "windows"))]
pub fn apply_windows_startup_record(dir: &Path, enabled: bool, program: &str) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = dir.join("PromptArk.run");
    if enabled {
        let mut bytes = vec![0xFF, 0xFE];
        for unit in windows_run_value(program).encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        fs::write(&path, bytes).map_err(|error| error.to_string())?;
    } else if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_windows_run_key(enabled: bool, program: &str) -> Result<(), String> {
    let value = windows_run_value(program).replace('\'', "''");
    let script = if enabled {
        format!(
            "Set-ItemProperty -LiteralPath 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'PromptArk' -Type String -Value '{value}'"
        )
    } else {
        "if (Get-ItemProperty -LiteralPath 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'PromptArk' -ErrorAction SilentlyContinue) { Remove-ItemProperty -LiteralPath 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'PromptArk' -ErrorAction SilentlyContinue }".into()
    };
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("当前系统尚未验证开机启动，不会声称已生效".into())
    }
}

#[cfg(any(test, target_os = "linux"))]
pub fn apply_linux_autostart(dir: &Path, enabled: bool, program: &str) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = dir.join("promptark.desktop");
    if enabled {
        let exec = if program.chars().any(char::is_whitespace) {
            format!("\"{}\"", program.replace('"', ""))
        } else {
            program.to_string()
        };
        let body = format!(
            "[Desktop Entry]\nType=Application\nName=PromptArk\nExec={exec}\nX-GNOME-Autostart-enabled=true\n"
        );
        fs::write(&path, body).map_err(|error| error.to_string())?;
    } else if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn apply_launch_at_login(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").map_err(|error| error.to_string())?;
        let dir = Path::new(&home).join("Library/LaunchAgents");
        let program = std::env::current_exe()
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .into_owned();
        return apply_launch_agent(&dir, enabled, &program);
    }
    #[cfg(target_os = "windows")]
    {
        let program = std::env::current_exe()
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .into_owned();
        return apply_windows_run_key(enabled, &program);
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").map_err(|error| error.to_string())?;
        let config = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
        let dir = Path::new(&config).join("autostart");
        let program = std::env::current_exe()
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .into_owned();
        return apply_linux_autostart(&dir, enabled, &program);
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = enabled;
        Err("当前系统尚未验证开机启动，不会声称已生效".into())
    }
}

#[tauri::command]
pub fn apply_minimize_to_tray(_app: AppHandle, enabled: bool) -> Result<(), String> {
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = enabled;
        return Err("当前系统尚未验证托盘，不会声称已生效".into());
    }
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
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

    #[test]
    fn writes_and_removes_windows_startup_record() {
        let dir = tempdir().unwrap();
        let program = r"C:\Users\测试\PromptArk.exe";
        apply_windows_startup_record(dir.path(), true, program).unwrap();
        let path = dir.path().join("PromptArk.run");
        let bytes = fs::read(&path).unwrap();
        assert_eq!(&bytes[..2], [0xFF, 0xFE]);
        let units: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        let body = String::from_utf16(&units).unwrap();
        assert_eq!(body, windows_run_value(program));
        assert!(body.contains("测试"));
        assert!(!body.contains("NSIS 已验证"));
        apply_windows_startup_record(dir.path(), false, program).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn writes_and_removes_linux_autostart_entry() {
        let dir = tempdir().unwrap();
        let program = "/home/测试/promptark";
        apply_linux_autostart(dir.path(), true, program).unwrap();
        let path = dir.path().join("promptark.desktop");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains(program));
        assert!(body.contains("Type=Application"));
        apply_linux_autostart(dir.path(), false, program).unwrap();
        assert!(!path.exists());
    }
}
