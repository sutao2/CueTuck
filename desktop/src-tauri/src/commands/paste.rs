use tauri::{AppHandle, Manager};

trait SelectedTextReader {
    fn read_selected_text(&self) -> Result<String, String>;
}

fn read_selected_text<R: SelectedTextReader>(reader: &R) -> Result<String, String> {
    reader.read_selected_text()
}

#[tauri::command]
pub fn capture_selected_text() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return read_selected_text(&MacAccessibilitySelectedTextReader);
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("当前平台暂不支持读取选中文本".into())
    }
}

#[tauri::command]
pub async fn paste_to_active_app(app: AppHandle) -> Result<(), String> {
    crate::commands::launcher::hide_launcher_window(&app)?;
    #[cfg(target_os = "macos")]
    {
        if let Some(previous) = app.try_state::<crate::commands::launcher::PreviousApplication>() {
            previous.restore_previous(&app)?;
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::thread::sleep(std::time::Duration::from_millis(180));
    }
    send_paste_keystroke()
}

#[cfg(target_os = "macos")]
fn send_paste_keystroke() -> Result<(), String> {
    if !accessibility_trusted() {
        return Err("缺少 macOS 辅助功能权限，无法自动粘贴".into());
    }
    post_macos_cmd_v()
}

#[cfg(not(target_os = "macos"))]
fn send_paste_keystroke() -> Result<(), String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|error| format!("初始化输入模拟失败: {error}"))?;
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|error| error.to_string())?;
    let typed = enigo.key(Key::Unicode('v'), Direction::Click);
    let released = enigo.key(Key::Control, Direction::Release);
    typed.map_err(|error| error.to_string())?;
    released.map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn accessibility_trusted() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> u8;
    }
    unsafe { AXIsProcessTrusted() != 0 }
}

#[cfg(target_os = "macos")]
fn post_macos_cmd_v() -> Result<(), String> {
    use std::ffi::c_void;

    const K_CG_HID_EVENT_TAP: u32 = 0;
    const K_CG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE: u32 = 1;
    const K_CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x0010_0000;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceCreate(state_id: u32) -> *mut c_void;
        fn CGEventCreateKeyboardEvent(
            source: *mut c_void,
            virtual_key: u16,
            key_down: bool,
        ) -> *mut c_void;
        fn CGEventSetFlags(event: *mut c_void, flags: u64);
        fn CGEventPost(tap: u32, event: *mut c_void);
    }
    extern "C" {
        fn CFRelease(cf: *const c_void);
    }

    unsafe {
        let source = CGEventSourceCreate(K_CG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE);
        if source.is_null() {
            return Err("创建 macOS 输入事件源失败".into());
        }
        let key_down = CGEventCreateKeyboardEvent(source, 9, true);
        let key_up = CGEventCreateKeyboardEvent(source, 9, false);
        if key_down.is_null() || key_up.is_null() {
            if !key_down.is_null() {
                CFRelease(key_down);
            }
            if !key_up.is_null() {
                CFRelease(key_up);
            }
            CFRelease(source);
            return Err("创建 macOS 粘贴按键事件失败".into());
        }
        CGEventSetFlags(key_down, K_CG_EVENT_FLAG_MASK_COMMAND);
        CGEventSetFlags(key_up, K_CG_EVENT_FLAG_MASK_COMMAND);
        CGEventPost(K_CG_HID_EVENT_TAP, key_down);
        CGEventPost(K_CG_HID_EVENT_TAP, key_up);
        CFRelease(key_down);
        CFRelease(key_up);
        CFRelease(source);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
struct MacAccessibilitySelectedTextReader;

#[cfg(target_os = "macos")]
impl SelectedTextReader for MacAccessibilitySelectedTextReader {
    fn read_selected_text(&self) -> Result<String, String> {
        if !accessibility_trusted() {
            return Err("缺少 macOS 辅助功能权限，无法读取选中文本".into());
        }
        macos_accessibility_selected_text()
    }
}

#[cfg(target_os = "macos")]
fn macos_accessibility_selected_text() -> Result<String, String> {
    use std::ffi::{c_char, c_void, CStr, CString};

    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateSystemWide() -> *const c_void;
        fn AXUIElementCopyAttributeValue(
            element: *const c_void,
            attribute: *const c_void,
            value: *mut *const c_void,
        ) -> i32;
        fn AXUIElementSetMessagingTimeout(element: *const c_void, timeout_in_seconds: f32) -> i32;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(
            allocator: *const c_void,
            value: *const c_char,
            encoding: u32,
        ) -> *const c_void;
        fn CFGetTypeID(value: *const c_void) -> usize;
        fn CFStringGetTypeID() -> usize;
        fn CFStringGetLength(value: *const c_void) -> isize;
        fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
        fn CFStringGetCString(
            value: *const c_void,
            buffer: *mut c_char,
            buffer_size: isize,
            encoding: u32,
        ) -> u8;
        fn CFRelease(value: *const c_void);
    }

    struct OwnedCf(*const c_void);
    impl Drop for OwnedCf {
        fn drop(&mut self) {
            unsafe { CFRelease(self.0) };
        }
    }

    unsafe {
        let name = CString::new("AXFocusedUIElement").unwrap();
        let focused_attr = CFStringCreateWithCString(
            std::ptr::null(),
            name.as_ptr(),
            K_CF_STRING_ENCODING_UTF8,
        );
        let selected_name = CString::new("AXSelectedText").unwrap();
        let selected_attr = CFStringCreateWithCString(
            std::ptr::null(),
            selected_name.as_ptr(),
            K_CF_STRING_ENCODING_UTF8,
        );
        let system = AXUIElementCreateSystemWide();
        if system.is_null() {
            return Err("创建系统 Accessibility 元素失败".into());
        }
        let _ = AXUIElementSetMessagingTimeout(system, 0.35);
        let mut focused = std::ptr::null();
        if AXUIElementCopyAttributeValue(system, focused_attr, &mut focused) != 0 {
            CFRelease(focused_attr);
            CFRelease(selected_attr);
            CFRelease(system);
            return Err("读取焦点元素失败".into());
        }
        let mut selected = std::ptr::null();
        let error = AXUIElementCopyAttributeValue(focused, selected_attr, &mut selected);
        CFRelease(focused_attr);
        CFRelease(selected_attr);
        CFRelease(focused);
        CFRelease(system);
        if error != 0 {
            return Err(format!("读取选中文本失败: {error}"));
        }
        let _owned = OwnedCf(selected);
        if CFGetTypeID(selected) != CFStringGetTypeID() {
            return Err("选中文本不是字符串".into());
        }
        let length = CFStringGetLength(selected);
        let capacity = CFStringGetMaximumSizeForEncoding(length, K_CF_STRING_ENCODING_UTF8) + 1;
        let mut buffer = vec![0 as c_char; capacity.max(1) as usize];
        if CFStringGetCString(
            selected,
            buffer.as_mut_ptr(),
            buffer.len() as isize,
            K_CF_STRING_ENCODING_UTF8,
        ) == 0
        {
            return Err("转换选中文本失败".into());
        }
        Ok(CStr::from_ptr(buffer.as_ptr())
            .to_string_lossy()
            .into_owned())
    }
}

#[cfg(test)]
mod selected_text_tests {
    use super::{read_selected_text, SelectedTextReader};

    struct FakeReader;
    impl SelectedTextReader for FakeReader {
        fn read_selected_text(&self) -> Result<String, String> {
            Ok("  selected text  ".into())
        }
    }

    #[test]
    fn selected_text_reader_preserves_whitespace() {
        assert_eq!(
            read_selected_text(&FakeReader).unwrap(),
            "  selected text  "
        );
    }
}
