mod device;

use ring::{
    pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

const LOCKED: &str = "请先解锁应用";
const DEVICE_FAILED: &str = "设备验证未完成，可使用解锁口令";
const ROUNDS: NonZeroU32 = NonZeroU32::new(600_000).unwrap();

#[derive(Serialize, Deserialize, PartialEq)]
struct Config {
    revision: u64,
    salt: [u8; 16],
    verifier: [u8; 32],
    device: bool,
}

struct Session {
    config: Option<Config>,
    locked: bool,
    generation: u64,
    failures: u32,
    retry_at: Option<Instant>,
}

pub struct AppLock {
    path: PathBuf,
    session: Mutex<Session>,
}

#[derive(Serialize)]
pub struct Status {
    enabled: bool,
    locked: bool,
    device: bool,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Enable {
        password: String,
        #[serde(default)]
        device: bool,
    },
    Unlock {
        password: String,
    },
    Lock,
    Change {
        current: String,
        password: String,
    },
    Disable {
        current: String,
    },
    SetDevice {
        current: String,
        enabled: bool,
    },
    UnlockDevice,
}

impl Session {
    fn status(&self) -> Status {
        Status {
            enabled: self.config.is_some(),
            locked: self.locked,
            device: self.config.as_ref().is_some_and(|c| c.device),
        }
    }

    fn verify(&mut self, password: &str) -> Result<(), String> {
        let cfg = self.config.as_ref().ok_or("尚未设置应用锁")?;
        if self.retry_at.is_some_and(|t| Instant::now() < t) {
            return Err("尝试次数过多，请稍后再试".into());
        }
        if pbkdf2::verify(
            pbkdf2::PBKDF2_HMAC_SHA256,
            ROUNDS,
            &cfg.salt,
            password.as_bytes(),
            &cfg.verifier,
        )
        .is_err()
        {
            self.failures += 1;
            if self.failures >= 5 {
                self.retry_at = Some(Instant::now() + Duration::from_secs(30));
            }
            return Err("解锁口令不正确".into());
        }
        self.failures = 0;
        self.retry_at = None;
        Ok(())
    }
}

impl AppLock {
    pub fn load(dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&dir)?;
        let path = dir.join("app-lock.json");
        Ok(Self {
            path,
            session: Mutex::new(Session {
                config: None,
                locked: false,
                generation: 0,
                failures: 0,
                retry_at: None,
            }),
        })
    }

    fn read_config(path: &Path) -> io::Result<Option<Config>> {
        match fs::read(path) {
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn session(&self) -> Result<MutexGuard<'_, Session>, String> {
        let mut state = self.session.lock().map_err(|_| LOCKED)?;
        // A separately launched desktop instance can enable, change or lock the same library.
        let saved = Self::read_config(&self.path).map_err(|_| "无法读取应用锁状态")?;
        if saved != state.config {
            state.locked = saved.is_some();
            state.config = saved;
            state.generation += 1;
        }
        Ok(state)
    }

    pub fn locked(&self) -> bool {
        self.session().map(|s| s.locked).unwrap_or(true)
    }

    fn store(&self, cfg: &Config) -> Result<(), String> {
        let write = || -> io::Result<()> {
            let mut file = tempfile::NamedTempFile::new_in(self.path.parent().unwrap())?;
            serde_json::to_writer(file.as_file_mut(), cfg)?;
            file.as_file().sync_all()?;
            file.persist(&self.path)?;
            Ok(())
        };
        write().map_err(|_| "无法保存应用锁设置".to_string())
    }

    fn apply(&self, request: Request) -> Result<(), String> {
        let guard = fs::File::options()
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.with_extension("guard"))
            .map_err(|_| "无法保存应用锁设置")?;
        guard.lock().map_err(|_| "无法保存应用锁设置")?;
        let mut s = self.session()?;
        if s.locked && !matches!(request, Request::Lock | Request::Unlock { .. }) {
            return Err(LOCKED.into());
        }
        let device = match &request {
            Request::Enable { device, .. } => *device,
            Request::Change { current, .. } => {
                s.verify(current)?;
                s.status().device
            }
            _ => s.status().device,
        };
        match request {
            Request::Lock => {
                if let Some(cfg) = &s.config {
                    let cfg = Config {
                        revision: cfg.revision.wrapping_add(1),
                        ..*cfg
                    };
                    self.store(&cfg)?;
                    s.config = Some(cfg);
                    s.locked = true;
                }
            }
            Request::Unlock { password } => {
                s.verify(&password)?;
                s.locked = false;
            }
            Request::UnlockDevice => return Err(DEVICE_FAILED.into()),
            Request::Enable { .. } if s.config.is_some() => return Err("应用锁已启用".into()),
            Request::Enable { password, .. }
            | Request::Change {
                current: _,
                password,
            } => {
                if password.chars().count() < 8 {
                    return Err("解锁口令至少需要 8 个字符".into());
                }
                let mut cfg = Config {
                    revision: 0,
                    salt: [0; 16],
                    verifier: [0; 32],
                    device,
                };
                SystemRandom::new()
                    .fill(&mut cfg.salt)
                    .map_err(|_| "无法生成口令盐值")?;
                pbkdf2::derive(
                    pbkdf2::PBKDF2_HMAC_SHA256,
                    ROUNDS,
                    &cfg.salt,
                    password.as_bytes(),
                    &mut cfg.verifier,
                );
                self.store(&cfg)?;
                s.config = Some(cfg);
            }
            Request::Disable { current } => {
                s.verify(&current)?;
                fs::remove_file(&self.path).map_err(|_| "无法保存应用锁设置")?;
                s.config = None;
            }
            Request::SetDevice { current, enabled } => {
                s.verify(&current)?;
                let cfg = s.config.as_ref().unwrap();
                let cfg = Config {
                    device: enabled,
                    ..*cfg
                };
                self.store(&cfg)?;
                s.config = Some(cfg);
            }
        }
        s.generation += 1;
        Ok(())
    }

    fn device_ticket(&self) -> Result<u64, String> {
        let s = self.session()?;
        if !s.locked || !s.status().device {
            return Err(DEVICE_FAILED.into());
        }
        Ok(s.generation)
    }

    fn finish_device(&self, ticket: u64, verified: bool) -> Result<(), String> {
        let mut s = self.session()?;
        if !verified || s.generation != ticket || !s.locked || !s.status().device {
            return Err(DEVICE_FAILED.into());
        }
        s.locked = false;
        s.generation += 1;
        s.failures = 0;
        s.retry_at = None;
        Ok(())
    }
}

pub fn dispatch<R: tauri::Runtime>(
    invoke: tauri::ipc::Invoke<R>,
    handle: impl FnOnce(tauri::ipc::Invoke<R>) -> bool,
) -> bool {
    let cmd = invoke.message.command();
    let webview = invoke.message.webview();
    if matches!(
        cmd,
        "app_lock_status" | "app_lock_action" | "app_lock_device_available"
    ) || webview
        .try_state::<AppLock>()
        .is_some_and(|lock| !lock.locked())
    {
        return handle(invoke);
    }
    if matches!(
        cmd,
        "toggle_launcher" | "show_launcher" | "open_new_prompt" | "paste_recent_prompt"
    ) {
        if let Some(main) = webview.get_webview_window("main") {
            let _ = main.show();
            let _ = main.unminimize();
            let _ = main.set_focus();
        }
        invoke.resolver.resolve(());
    } else {
        invoke.resolver.reject(LOCKED);
    }
    true
}

#[tauri::command]
pub fn app_lock_status(lock: State<'_, AppLock>) -> Result<Status, String> {
    Ok(lock.session()?.status())
}

#[tauri::command]
pub async fn app_lock_device_available(window: WebviewWindow) -> bool {
    window.label() == "main"
        && tauri::async_runtime::spawn_blocking(move || device::verify(&window, false))
            .await
            .unwrap_or(false)
}

pub fn notify(app: &AppHandle) {
    if app.state::<AppLock>().locked() {
        if let Some(launcher) = app.get_webview_window("launcher") {
            let _ = launcher.hide();
        }
    }
    let _ = app.emit("app-lock-changed", ());
}

#[tauri::command]
pub async fn app_lock_action(window: WebviewWindow, request: Request) -> Result<(), String> {
    if window.label() != "main" {
        return Err(LOCKED.into());
    }
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let lock = app.state::<AppLock>();
        match request {
            Request::UnlockDevice => {
                let ticket = lock.device_ticket()?;
                lock.finish_device(ticket, device::verify(&window, true))?;
            }
            Request::Enable { device: true, .. } | Request::SetDevice { enabled: true, .. }
                if !device::verify(&window, false) =>
            {
                return Err(DEVICE_FAILED.into())
            }
            other => lock.apply(other)?,
        }
        notify(&app);
        Ok(())
    })
    .await
    .map_err(|_| LOCKED)?
}

#[cfg(test)]
mod tests {
    use super::*;
    const PASS: &str = "local passphrase";

    #[test]
    fn locked_ipc_denies_business_commands_and_redirects_shortcuts() {
        use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
        let dir = tempfile::tempdir().unwrap();
        let lock = AppLock::load(dir.path().into()).unwrap();
        lock.apply(Request::Enable {
            password: PASS.into(),
            device: false,
        })
        .unwrap();
        let app = mock_builder()
            .manage(lock)
            .invoke_handler(|invoke| {
                dispatch(invoke, |allowed| {
                    allowed.resolver.resolve("dispatched");
                    true
                })
            })
            .build(mock_context(noop_assets()))
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let launcher = tauri::WebviewWindowBuilder::new(&app, "launcher", Default::default())
            .build()
            .unwrap();
        let call = |window, cmd: &str| {
            get_ipc_response(
                window,
                tauri::webview::InvokeRequest {
                    cmd: cmd.into(),
                    callback: tauri::ipc::CallbackFn(0),
                    error: tauri::ipc::CallbackFn(1),
                    url: if cfg!(windows) {
                        "http://tauri.localhost"
                    } else {
                        "tauri://localhost"
                    }
                    .parse()
                    .unwrap(),
                    body: Default::default(),
                    headers: Default::default(),
                    invoke_key: INVOKE_KEY.into(),
                },
            )
            .map(|r| r.deserialize::<serde_json::Value>().unwrap())
        };
        assert_eq!(call(&launcher, "quick_clip").unwrap(), "dispatched");
        app.state::<AppLock>().apply(Request::Lock).unwrap();
        for window in [&launcher, &main] {
            for cmd in [
                "quick_clip",
                "list_local_prompts",
                "skills_command",
                "copy_launcher_text",
            ] {
                assert_eq!(call(window, cmd).unwrap_err(), LOCKED);
            }
            assert_eq!(call(window, "app_lock_status").unwrap(), "dispatched");
            assert!(call(window, "paste_recent_prompt").unwrap().is_null());
            assert!(call(window, "toggle_launcher").unwrap().is_null());
        }
    }

    #[test]
    fn persists_only_a_salted_verifier_and_requires_the_current_password() {
        let dir = tempfile::tempdir().unwrap();
        let lock = AppLock::load(dir.path().into()).unwrap();
        let second = AppLock::load(dir.path().into()).unwrap();
        assert!(!lock.locked());
        assert!(lock
            .apply(Request::Enable {
                device: false,
                password: "short".into()
            })
            .is_err());
        lock.apply(Request::Enable {
            device: false,
            password: PASS.into(),
        })
        .unwrap();
        assert!(second.locked());
        assert!(!fs::read_to_string(&lock.path).unwrap().contains(PASS));
        let original = fs::read(&lock.path).unwrap();
        assert!(lock
            .apply(Request::Enable {
                device: false,
                password: PASS.into()
            })
            .is_err());
        assert!(lock
            .apply(Request::Change {
                current: "incorrect".into(),
                password: PASS.into()
            })
            .is_err());
        assert_eq!(fs::read(&lock.path).unwrap(), original);
        lock.apply(Request::Change {
            current: PASS.into(),
            password: PASS.into(),
        })
        .unwrap();
        assert_ne!(fs::read(&lock.path).unwrap(), original);
        let restarted = AppLock::load(dir.path().into()).unwrap();
        assert!(restarted.locked());
        assert!(restarted
            .apply(Request::Disable {
                current: PASS.into()
            })
            .is_err());
        assert!(restarted
            .apply(Request::Unlock {
                password: "incorrect".into()
            })
            .is_err());
        assert!(restarted.locked());
        restarted
            .apply(Request::Unlock {
                password: PASS.into(),
            })
            .unwrap();
        assert!(!restarted.locked());
        lock.apply(Request::Lock).unwrap();
        assert!(restarted.locked());
        restarted
            .apply(Request::Unlock {
                password: PASS.into(),
            })
            .unwrap();
        assert!(restarted
            .apply(Request::Disable {
                current: "incorrect".into()
            })
            .is_err());
        restarted
            .apply(Request::Disable {
                current: PASS.into(),
            })
            .unwrap();
        assert!(!lock.locked());
        assert!(!AppLock::load(dir.path().into()).unwrap().locked());
    }

    #[test]
    fn stale_or_cancelled_device_approval_cannot_unlock() {
        let dir = tempfile::tempdir().unwrap();
        let lock = AppLock::load(dir.path().into()).unwrap();
        lock.apply(Request::Enable {
            device: false,
            password: PASS.into(),
        })
        .unwrap();
        assert!(lock.device_ticket().is_err());
        lock.apply(Request::SetDevice {
            current: PASS.into(),
            enabled: true,
        })
        .unwrap();
        lock.apply(Request::Lock).unwrap();
        let first = lock.device_ticket().unwrap();
        assert!(lock.finish_device(first, false).is_err());
        assert!(lock.locked());
        lock.apply(Request::Lock).unwrap();
        assert!(lock.finish_device(first, true).is_err());
        assert!(lock.locked());
        lock.finish_device(lock.device_ticket().unwrap(), true)
            .unwrap();
        assert!(!lock.locked());
        fs::write(&lock.path, b"broken").unwrap();
        assert!(lock.locked());
        assert!(AppLock::load(dir.path().into()).unwrap().locked());
    }
}
