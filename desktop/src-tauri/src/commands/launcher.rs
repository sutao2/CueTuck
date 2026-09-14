use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, Size};

pub const LAUNCHER_LABEL: &str = "launcher";
const FOCUS_GRACE: Duration = Duration::from_millis(600);

// Ported from the independent legacy launcher: activation is asynchronous on macOS.
#[cfg(any(target_os = "macos", test))]
fn wait_for_stable_focus(mut ready: impl FnMut() -> bool, mut sleep: impl FnMut(Duration)) -> Result<(), String> {
    let mut stable = 0;
    for observation in 0..20 {
        stable = if ready() { stable + 1 } else { 0 };
        if stable == 4 { return Ok(()); }
        if observation < 19 { sleep(Duration::from_millis(25)); }
    }
    Err("等待原窗口恢复焦点超时".into())
}

#[derive(Default)]
pub struct LauncherFocusGuard {
    shown: Mutex<Option<Instant>>,
    layout: Mutex<Option<&'static str>>,
}

impl LauncherFocusGuard {
    pub fn mark_shown(&self) {
        *self.shown.lock().unwrap() = Some(Instant::now());
    }

    pub fn in_grace_period(&self) -> bool {
        self.shown
            .lock()
            .unwrap()
            .map(|shown| shown.elapsed() < FOCUS_GRACE)
            .unwrap_or(false)
    }

    fn remember_layout(&self, layout: &str) {
        *self.layout.lock().unwrap() = Some(match layout { "fill" => "fill", "expanded" => "expanded", _ => "collapsed" });
    }

    fn current_layout(&self) -> &'static str {
        self.layout.lock().unwrap().unwrap_or("collapsed")
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
                wait_for_stable_focus(|| window.is_focused().unwrap_or(false), std::thread::sleep)
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
                wait_for_stable_focus(|| {
                    application.isActive() && NSWorkspace::sharedWorkspace()
                        .frontmostApplication().map(|frontmost| frontmost.processIdentifier()) == Some(pid)
                }, std::thread::sleep)
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

// Temporary restoration after paste/selection must not clear the current draft.
#[tauri::command]
pub fn resume_launcher(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window(LAUNCHER_LABEL).ok_or("启动器窗口不存在")?;
    if let Some(guard) = app.try_state::<LauncherFocusGuard>() { guard.mark_shown(); }
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

fn launcher_size(size: &str) -> (f64, f64) {
    match size {
        "standard" => (680.0, 500.0),
        "large" => (760.0, 560.0),
        _ => (620.0, 420.0),
    }
}

fn read_launcher_preferences(app: &AppHandle) -> Result<serde_json::Value, String> {
    let dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
    read_launcher_preferences_in_dir(&dir)
}

fn read_launcher_preferences_in_dir(dir: &std::path::Path) -> Result<serde_json::Value, String> {
    let raw = crate::local_database::get_optional_setting_in_dir(dir, "launcher_preferences")?.unwrap_or_default();
    Ok(serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null))
}

pub fn launcher_logical_height(layout: &str) -> f64 {
    match layout {
        "collapsed" => 64.0,
        _ => 420.0,
    }
}

fn resize_launcher_window(app: &AppHandle, layout: &str) -> Result<(), String> {
    let window = app
        .get_webview_window(LAUNCHER_LABEL)
        .ok_or_else(|| "启动器窗口不存在".to_string())?;
    let position = window.outer_position().map_err(|error| error.to_string())?;
    let preferences = read_launcher_preferences(app)?;
    let (mut width, mut height) = launcher_size(preferences["size"].as_str().unwrap_or("compact"));
    if layout == "collapsed" { height = launcher_logical_height(layout); }
    if let Some(monitor) = window.current_monitor().map_err(|error| error.to_string())? {
        (width, height) = fit_launcher_size((width, height), monitor.work_area(), monitor.scale_factor());
    }
    window
        .set_size(Size::Logical(LogicalSize::new(
            width,
            height,
        )))
        .map_err(|error| error.to_string())?;
    // Keep the search bar anchored, including after dragging and on macOS resize.
    window
        .set_position(position)
        .map_err(|error| error.to_string())?;
    if let Some(guard) = app.try_state::<LauncherFocusGuard>() { guard.remember_layout(layout); }
    Ok(())
}

fn fit_launcher_size(size: (f64, f64), area: &tauri::PhysicalRect<i32, u32>, scale: f64) -> (f64, f64) {
    (size.0.min(area.size.width as f64 / scale), size.1.min(area.size.height as f64 / scale))
}

fn launcher_show_position(
    area: &tauri::PhysicalRect<i32, u32>,
    scale: f64,
    preferences: &serde_json::Value,
) -> tauri::PhysicalPosition<i32> {
    let (width, height) = fit_launcher_size(launcher_size(preferences["size"].as_str().unwrap_or("compact")), area, scale);
    let divisor = if preferences["position"].as_str() == Some("center") { 2.0 } else { 4.0 };
    tauri::PhysicalPosition::new(
        area.position.x + ((area.size.width as f64 - width * scale).max(0.0) / 2.0).round() as i32,
        area.position.y
            + ((area.size.height as f64 - height * scale).max(0.0)
                / divisor)
                .round() as i32,
    )
}

fn show_launcher_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(LAUNCHER_LABEL)
        .ok_or_else(|| "启动器窗口不存在".to_string())?;
    if window.is_visible().map_err(|error| error.to_string())? {
        return resume_launcher(app.clone());
    }
    #[cfg(target_os = "macos")]
    if let Some(previous) = app.try_state::<PreviousApplication>() {
        previous.remember_frontmost(app);
    }
    if let Some(guard) = app.try_state::<LauncherFocusGuard>() {
        guard.mark_shown();
    }
    let layout = app.try_state::<LauncherFocusGuard>().map(|guard| guard.current_layout()).unwrap_or("collapsed");
    resize_launcher_window(app, layout)?;
    let preferences = read_launcher_preferences(app)?;
    if let Some(monitor) = window.current_monitor().map_err(|error| error.to_string())? {
        window
            .set_position(launcher_show_position(monitor.work_area(), monitor.scale_factor(), &preferences))
            .map_err(|error| error.to_string())?;
    } else {
        window.center().map_err(|error| error.to_string())?;
    }
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    window.emit("launcher-shown", ()).map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn show_launcher(app: AppHandle) -> Result<(), String> {
    show_launcher_window(&app)
}

#[tauri::command]
pub async fn hide_launcher(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let return_focus = app.get_webview_window(LAUNCHER_LABEL)
        .is_some_and(|window| window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false));
    hide_launcher_window(&app)?;
    if let Some(guard) = app.try_state::<LauncherFocusGuard>() { guard.remember_layout("collapsed"); }
    app.emit_to(LAUNCHER_LABEL, "launcher-hidden", ()).map_err(|error| error.to_string())?;
    // A dismiss is not a paste: restore only if we still owned focus, never reopen on failure.
    #[cfg(target_os = "macos")]
    if return_focus {
        if let Some(previous) = app.try_state::<PreviousApplication>() {
            if let Err(error) = previous.restore_previous(&app) {
                eprintln!("Launcher dismissed without restoring target: {error}");
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_launcher_if_idle(app: AppHandle) -> Result<bool, String> {
    if app.get_webview_window(LAUNCHER_LABEL).is_some_and(|window| window.is_focused().unwrap_or(false)) {
        return Ok(false);
    }
    if let Some(guard) = app.try_state::<LauncherFocusGuard>() {
        if guard.in_grace_period() {
            return Ok(false);
        }
    }
    // Passive blur suspends the draft; it must not restore focus or reset the layout.
    hide_launcher_window(&app)?;
    app.emit_to(LAUNCHER_LABEL, "launcher-hidden", "blur").map_err(|error| error.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn resize_launcher(app: AppHandle, layout: String) -> Result<(), String> {
    resize_launcher_window(&app, &layout)
}

#[tauri::command]
pub async fn toggle_launcher(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(LAUNCHER_LABEL)
        .ok_or_else(|| "启动器窗口不存在".to_string())?;
    if window.is_visible().map_err(|error| error.to_string())? {
        hide_launcher(app).await
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
    let result = paste_recent(&app).await;
    if let Err(message) = &result {
        resume_launcher(app.clone())?;
        app.emit_to(LAUNCHER_LABEL, "launcher-feedback", message).map_err(|error| error.to_string())?;
    }
    result
}

async fn paste_recent(app: &AppHandle) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
    let text = crate::local_database::get_setting_in_dir(&dir, "last_rendered_prompt")?;
    if text.trim().is_empty() {
        return Err("没有最近使用的提示词".into());
    }
    #[cfg(target_os = "macos")]
    if !app.get_webview_window(LAUNCHER_LABEL).is_some_and(|window| window.is_visible().unwrap_or(false)) {
        app.state::<PreviousApplication>().remember_frontmost(app);
    }
    copy_text_to_clipboard(&text)?;
    crate::commands::paste::paste_to_active_app(app.clone()).await
        .map_err(|error| format!("已复制，未能粘贴：{error}"))
}

#[tauri::command]
pub async fn copy_launcher_text(text: String) -> Result<(), String> {
    if text.trim().is_empty() { return Err("提示词内容为空".into()); }
    copy_text_to_clipboard(&text)
}

fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        use std::process::Stdio;
        let mut child = clipboard_command("/usr/bin/pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|error| error.to_string())?;
        child
            .stdin
            .as_mut()
            .ok_or_else(|| "无法写入剪贴板".to_string())?
            .write_all(text.as_bytes())
            .map_err(|error| error.to_string())?;
        let status = child.wait().map_err(|error| error.to_string())?;
        if !status.success() { return Err(format!("系统剪贴板写入失败：{status}")); }
        let pasted = clipboard_command("/usr/bin/pbpaste")
            .args(["-Prefer", "txt"])
            .output().map_err(|error| error.to_string())?;
        if !pasted.status.success() { return Err("无法确认系统剪贴板内容，请重试".into()); }
        verify_clipboard_text(text, &pasted.stdout)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = text;
        Err("当前系统尚未验证粘贴最近使用".into())
    }
}

#[cfg(target_os = "macos")]
fn clipboard_command(program: &str) -> std::process::Command {
    let mut command = std::process::Command::new(program);
    // GUI-launched applications need not inherit Terminal's UTF-8 locale.
    command.env("LANG", "en_US.UTF-8").env("LC_ALL", "en_US.UTF-8");
    command
}

#[cfg(any(target_os = "macos", test))]
fn verify_clipboard_text(expected: &str, actual: &[u8]) -> Result<(), String> {
    if actual == expected.as_bytes() { Ok(()) }
    else { Err("系统剪贴板内容未正确写入，请重试".into()) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn clipboard_confirmation_rejects_empty_truncated_or_changed_unicode() {
        let text = "中文提示词 🦜\n第二行 café";
        assert!(super::verify_clipboard_text(text, text.as_bytes()).is_ok());
        for bytes in [b"".as_slice(), "中文提示词".as_bytes(), b"old clipboard".as_slice()] {
            assert!(super::verify_clipboard_text(text, bytes).is_err());
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn clipboard_commands_override_gui_locale_for_copy_and_confirmation() {
        for program in ["/usr/bin/pbcopy", "/usr/bin/pbpaste"] {
            let command = super::clipboard_command(program);
            let env: std::collections::HashMap<_, _> = command.get_envs().collect();
            assert_eq!(env[std::ffi::OsStr::new("LC_ALL")], Some(std::ffi::OsStr::new("en_US.UTF-8")));
            assert_eq!(env[std::ffi::OsStr::new("LANG")], Some(std::ffi::OsStr::new("en_US.UTF-8")));
        }
    }

    #[test]
    fn remembers_fill_layout_until_explicit_reset() {
        let guard=super::LauncherFocusGuard::default();
        assert_eq!(guard.current_layout(),"collapsed");
        guard.remember_layout("fill");guard.mark_shown();
        assert_eq!(guard.current_layout(),"fill");
        guard.remember_layout("collapsed");assert_eq!(guard.current_layout(),"collapsed");
    }
    use super::{LauncherFocusGuard, LAUNCHER_LABEL};

    #[test]
    fn missing_launcher_preferences_use_defaults_without_writing_settings() {
        let dir = tempfile::tempdir().unwrap();
        crate::local_database::LocalDatabase::default().initialize(dir.path()).unwrap();
        let preferences = super::read_launcher_preferences_in_dir(dir.path()).unwrap();
        assert!(preferences.is_null());
        assert_eq!(super::launcher_size(preferences["size"].as_str().unwrap_or("compact")), (620.0, 420.0));
        assert!(crate::local_database::get_setting_in_dir(dir.path(), "launcher_preferences").is_err());
    }

    #[test]
    fn launcher_preferences_preserve_saved_values_and_database_errors() {
        let dir = tempfile::tempdir().unwrap();
        assert!(super::read_launcher_preferences_in_dir(dir.path()).is_err());
        crate::local_database::LocalDatabase::default().initialize(dir.path()).unwrap();
        crate::local_database::set_setting_in_dir(dir.path(), "launcher_preferences", r#"{"size":"large","position":"center"}"#).unwrap();
        assert_eq!(super::read_launcher_preferences_in_dir(dir.path()).unwrap()["size"], "large");
        crate::local_database::set_setting_in_dir(dir.path(), "launcher_preferences", "invalid json").unwrap();
        assert!(super::read_launcher_preferences_in_dir(dir.path()).unwrap().is_null());
    }

    #[test]
    fn launcher_label_is_stable() {
        assert_eq!(LAUNCHER_LABEL, "launcher");
    }

    #[test]
    fn palette_heights_keep_search_and_fill_compact() {
        assert_eq!(super::launcher_logical_height("collapsed"), 64.0);
        assert_eq!(super::launcher_logical_height("expanded"), 420.0);
        assert_eq!(super::launcher_logical_height("fill"), 420.0);
        assert_eq!(super::launcher_logical_height("fill"), super::launcher_logical_height("expanded"));
    }

    #[test]
    fn focus_grace_is_600ms() {
        let guard = LauncherFocusGuard::default();
        assert!(!guard.in_grace_period());
        guard.mark_shown();
        assert!(guard.in_grace_period());
    }

    #[test]
    fn focus_must_be_stable_before_pasting() {
        let mut observations = [true, true, false, true, true, true, true].into_iter();
        let mut sleeps = 0;
        assert!(super::wait_for_stable_focus(|| observations.next().unwrap(), |_| sleeps += 1).is_ok());
        assert_eq!(sleeps, 6);
    }

    #[test]
    fn unready_target_times_out_without_proceeding() {
        let mut sleeps = 0;
        assert!(super::wait_for_stable_focus(|| false, |_| sleeps += 1).is_err());
        assert_eq!(sleeps, 19);
    }

    #[test]
    fn palette_opens_above_center_on_scaled_and_offset_monitors() {
        for (x, y, scale) in [(0, 0, 1.0), (0, 48, 2.0), (-2880, -1800, 2.0)] {
            let area = tauri::PhysicalRect {
                position: tauri::PhysicalPosition::new(x, y),
                size: tauri::PhysicalSize::new((1440.0 * scale) as u32, (980.0 * scale) as u32),
            };
            assert_eq!(
                super::launcher_show_position(&area, scale, &serde_json::Value::Null),
                tauri::PhysicalPosition::new(x + (410.0 * scale) as i32, y + (140.0 * scale) as i32),
            );
        }
    }

    #[test]
    fn palette_reserves_expanded_space_on_small_screens() {
        let area = tauri::PhysicalRect {
            position: tauri::PhysicalPosition::new(0, 24),
            size: tauri::PhysicalSize::new(1280, 696),
        };
        let position = super::launcher_show_position(&area, 1.0, &serde_json::Value::Null);
        assert_eq!(position.y, 93);
        assert!(position.y + 420 <= 720);
    }

    #[test]
    fn preferences_select_sizes_centering_and_fit_work_area() {
        assert_eq!(super::launcher_size("standard"), (680.0, 500.0));
        assert_eq!(super::launcher_size("large"), (760.0, 560.0));
        assert_eq!(super::launcher_size("invalid"), (620.0, 420.0));
        let area = tauri::PhysicalRect { position: tauri::PhysicalPosition::new(-2880, 48), size: tauri::PhysicalSize::new(2880, 1960) };
        let prefs = serde_json::json!({"size":"large", "position":"center"});
        assert_eq!(super::launcher_show_position(&area, 2.0, &prefs), tauri::PhysicalPosition::new(-2200, 468));
        let small = tauri::PhysicalRect { position: tauri::PhysicalPosition::new(0, 24), size: tauri::PhysicalSize::new(700, 520) };
        assert_eq!(super::fit_launcher_size(super::launcher_size("large"), &small, 1.0), (700.0, 520.0));
        assert_eq!(super::launcher_show_position(&small, 1.0, &prefs), small.position);
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

#[tauri::command]
pub fn open_launcher_destination(app: AppHandle, destination: String, id: Option<String>) -> Result<(), String> {
    if !["ai-settings", "square-detail"].contains(&destination.as_str()) || id.as_ref().is_some_and(|v|v.len()>200) { return Err("无效启动器目标".into()); }
    let window=app.get_webview_window("main").ok_or("主窗口不存在")?;
    window.emit("launcher-navigate",serde_json::json!({"destination":destination,"id":id})).map_err(|_|"页面导航失败")?;
    window.show().map_err(|e|e.to_string())?;
    window.set_focus().map_err(|e|e.to_string())?;
    Ok(())
}
