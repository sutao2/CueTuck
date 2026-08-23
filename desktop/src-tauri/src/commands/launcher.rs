use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, Size};

pub const LAUNCHER_LABEL: &str = "launcher";
const FOCUS_GRACE: Duration = Duration::from_millis(600);

#[derive(Default)]
pub struct LauncherFocusGuard(Mutex<Option<Instant>>);

impl LauncherFocusGuard {
    pub fn mark_shown(&self) {
        *self.0.lock().unwrap() = Some(Instant::now());
    }

    pub fn in_grace_period(&self) -> bool {
        self.0
            .lock()
            .unwrap()
            .map(|shown| shown.elapsed() < FOCUS_GRACE)
            .unwrap_or(false)
    }
}

#[cfg(target_os = "macos")]
#[derive(Clone, Debug, Eq, PartialEq)]
enum PreviousTarget {
    OwnWindow(String),
    ExternalApplication(i32),
}

#[cfg(target_os = "macos")]
#[derive(Default)]
pub struct PreviousApplication(Mutex<Option<PreviousTarget>>);

#[cfg(target_os = "macos")]
fn classify_previous_target(
    current_pid: i32,
    frontmost_pid: Option<i32>,
    focused_own_window_label: Option<&str>,
) -> Option<PreviousTarget> {
    match frontmost_pid.filter(|pid| *pid > 0) {
        Some(pid) if pid == current_pid => {
            focused_own_window_label.map(|label| PreviousTarget::OwnWindow(label.to_string()))
        }
        Some(pid) if pid != current_pid => Some(PreviousTarget::ExternalApplication(pid)),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
impl PreviousApplication {
    pub fn remember_frontmost(&self, app: &AppHandle) {
        use objc2_app_kit::{NSRunningApplication, NSWorkspace};

        let current_pid = NSRunningApplication::currentApplication().processIdentifier();
        let frontmost_pid = NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .map(|application| application.processIdentifier());
        let focused_own_window_label = app.webview_windows().into_iter().find_map(|(label, window)| {
            (label != LAUNCHER_LABEL && window.is_focused().unwrap_or(false)).then_some(label)
        });
        *self.0.lock().unwrap() = classify_previous_target(
            current_pid,
            frontmost_pid,
            focused_own_window_label.as_deref(),
        );
    }

    pub fn restore_previous(&self, app: &AppHandle) -> Result<(), String> {
        use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};

        let target = self
            .0
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "未记录启动器打开前的活动应用".to_string())?;
        match target {
            PreviousTarget::OwnWindow(label) => {
                let window = app
                    .get_webview_window(&label)
                    .ok_or_else(|| format!("窗口 {label} 不可用"))?;
                window.show().map_err(|error| error.to_string())?;
                window.set_focus().map_err(|error| error.to_string())?;
                Ok(())
            }
            PreviousTarget::ExternalApplication(pid) => {
                let application = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
                    .ok_or_else(|| "启动器打开前的应用已经退出".to_string())?;
                if application.isTerminated() {
                    return Err("启动器打开前的应用已经退出".into());
                }
                let _ = application.unhide();
                if !application.activateWithOptions(NSApplicationActivationOptions::empty()) {
                    return Err("无法恢复启动器打开前的应用".into());
                }
                let _ = NSWorkspace::sharedWorkspace();
                Ok(())
            }
        }
    }
}

pub fn hide_launcher_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(LAUNCHER_LABEL)
        .ok_or_else(|| "启动器窗口不存在".to_string())?;
    window.hide().map_err(|error| error.to_string())
}

pub fn launcher_logical_height(layout: &str) -> f64 {
    match layout {
        "collapsed" => 80.0,
        "fill" => 520.0,
        _ => 500.0,
    }
}

fn resize_launcher_window(app: &AppHandle, layout: &str) -> Result<(), String> {
    let window = app
        .get_webview_window(LAUNCHER_LABEL)
        .ok_or_else(|| "启动器窗口不存在".to_string())?;
    window
        .set_size(Size::Logical(LogicalSize::new(
            680.0,
            launcher_logical_height(layout),
        )))
        .map_err(|error| error.to_string())?;
    let _ = window.center();
    Ok(())
}

fn show_launcher_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(LAUNCHER_LABEL)
        .ok_or_else(|| "启动器窗口不存在".to_string())?;
    #[cfg(target_os = "macos")]
    if let Some(previous) = app.try_state::<PreviousApplication>() {
        previous.remember_frontmost(app);
    }
    if let Some(guard) = app.try_state::<LauncherFocusGuard>() {
        guard.mark_shown();
    }
    resize_launcher_window(app, "collapsed")?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn show_launcher(app: AppHandle) -> Result<(), String> {
    show_launcher_window(&app)
}

#[tauri::command]
pub fn hide_launcher(app: AppHandle) -> Result<(), String> {
    hide_launcher_window(&app)
}

#[tauri::command]
pub fn hide_launcher_if_idle(app: AppHandle) -> Result<bool, String> {
    if let Some(guard) = app.try_state::<LauncherFocusGuard>() {
        if guard.in_grace_period() {
            return Ok(false);
        }
    }
    hide_launcher_window(&app)?;
    Ok(true)
}

#[tauri::command]
pub fn resize_launcher(app: AppHandle, layout: String) -> Result<(), String> {
    resize_launcher_window(&app, &layout)
}

#[tauri::command]
pub fn toggle_launcher(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(LAUNCHER_LABEL)
        .ok_or_else(|| "启动器窗口不存在".to_string())?;
    if window.is_visible().map_err(|error| error.to_string())? {
        hide_launcher_window(&app)
    } else {
        show_launcher_window(&app)
    }
}

#[tauri::command]
pub fn open_new_prompt(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    window
        .emit("open-new-prompt", ())
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn paste_recent_prompt(app: AppHandle) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
    let text = crate::local_database::get_setting_in_dir(&dir, "last_rendered_prompt").unwrap_or_default();
    if text.trim().is_empty() {
        return Err("没有最近使用的提示词".into());
    }
    copy_text_to_clipboard(&text)?;
    crate::commands::paste::paste_to_active_app(app).await
}

fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|error| error.to_string())?;
        child
            .stdin
            .as_mut()
            .ok_or_else(|| "无法写入剪贴板".to_string())?
            .write_all(text.as_bytes())
            .map_err(|error| error.to_string())?;
        child.wait().map_err(|error| error.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = text;
        Err("当前系统尚未验证粘贴最近使用".into())
    }
}

#[cfg(test)]
mod tests {
    use super::{LauncherFocusGuard, LAUNCHER_LABEL};

    #[test]
    fn launcher_label_is_stable() {
        assert_eq!(LAUNCHER_LABEL, "launcher");
    }

    #[test]
    fn palette_heights_match_old_window() {
        assert_eq!(super::launcher_logical_height("collapsed"), 80.0);
        assert_eq!(super::launcher_logical_height("expanded"), 500.0);
        assert_eq!(super::launcher_logical_height("fill"), 520.0);
    }

    #[test]
    fn focus_grace_is_600ms() {
        let guard = LauncherFocusGuard::default();
        assert!(!guard.in_grace_period());
        guard.mark_shown();
        assert!(guard.in_grace_period());
    }
}

#[cfg(all(test, target_os = "macos"))]
mod previous_target_tests {
    use super::{classify_previous_target, PreviousTarget};

    #[test]
    fn current_pid_with_focused_own_window_records_its_label() {
        assert_eq!(
            classify_previous_target(42, Some(42), Some("main")),
            Some(PreviousTarget::OwnWindow("main".into()))
        );
    }

    #[test]
    fn external_pid_records_external_application() {
        assert_eq!(
            classify_previous_target(42, Some(84), None),
            Some(PreviousTarget::ExternalApplication(84))
        );
    }
}
