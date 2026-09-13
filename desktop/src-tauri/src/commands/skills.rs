use crate::skills::{self, files, install, local, remote, Root, Source};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Default)]
pub struct SkillsState {
    pub gate: tokio::sync::Mutex<()>,
    pub active: Mutex<Option<(String, Arc<AtomicBool>)>>,
}
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Request {
    Snapshot,
    Catalog {
        input: String,
    },
    Descriptions {
        source: Source,
        entries: Vec<remote::Entry>,
    },
    SaveSource {
        input: String,
        remove: bool,
    },
    PrepareLocal {
        path: PathBuf,
    },
    PrepareRemote {
        source: Source,
    },
    ReadPrepared {
        id: String,
        file: String,
    },
    Discard {
        id: String,
    },
    Preflight {
        id: String,
        root_ids: Vec<String>,
    },
    Install {
        id: String,
        selections: Vec<install::Selection>,
    },
    CheckUpdate {
        path: PathBuf,
    },
    Remove {
        path: PathBuf,
        root_id: String,
        expected_digest: String,
    },
    Restore {
        id: String,
    },
    Detail {
        path: PathBuf,
    },
    ReadFile {
        path: PathBuf,
        file: String,
    },
    SaveRoot {
        root: Root,
    },
    AddProject {
        path: PathBuf,
    },
    ForgetRoot {
        id: String,
    },
    OpenDirectory {
        path: PathBuf,
    },
}
fn data_dir(app: &AppHandle) -> skills::Result<PathBuf> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("skills"))
}
fn defaults(app: &AppHandle) -> skills::Result<Vec<Root>> {
    Ok(local::default_roots(
        &app.path().home_dir().map_err(|e| e.to_string())?,
        |key| std::env::var(key).ok(),
    ))
}
fn value(data: impl Serialize) -> skills::Result<serde_json::Value> {
    serde_json::to_value(data).map_err(|e| e.to_string())
}
fn record<T>(
    dir: &std::path::Path,
    registry: &mut skills::Registry,
    target: PathBuf,
    result: &skills::Result<T>,
    message: &str,
) -> skills::Result<()> {
    registry.operations.push(skills::OperationResult {
        target,
        status: if result.is_ok() { "success" } else { "failed" }.into(),
        message: result
            .as_ref()
            .err()
            .cloned()
            .unwrap_or_else(|| message.into()),
        at: skills::now(),
    });
    if registry.operations.len() > 100 {
        registry.operations.drain(..registry.operations.len() - 100);
    }
    files::save_registry(dir, registry)
}
fn run_local(
    dir: PathBuf,
    defaults: Vec<Root>,
    request: Request,
) -> skills::Result<serde_json::Value> {
    let mut registry = files::load_registry(&dir)?;
    install::recover(&dir, &mut registry)?;
    match request {
        Request::PrepareLocal { path } => value(install::prepare_local(&dir, &path)?),
        Request::ReadPrepared { id, file } => {
            install::prepared(&dir, &id)?;
            let path = install::prepared_dir(&dir, &id)?
                .join("package")
                .join(files::relative(&file)?);
            value(
                String::from_utf8(files::read_bounded(&path, 1024 * 1024)?)
                    .map_err(|_| "这是二进制文件，请通过文件列表查看")?,
            )
        }
        Request::Discard { id } => {
            install::discard(&dir, &id)?;
            value(())
        }
        Request::Preflight { id, root_ids } => {
            if root_ids.len() > 200 {
                return Err("目标数量超过上限".into());
            }
            let p = install::prepared(&dir, &id)?;
            let plans: Vec<_> = root_ids
                .into_iter()
                .map(
                    |root_id| match install::plan(&defaults, &registry, &p, &root_id) {
                        Ok(plan) => plan,
                        Err(message) => skills::TargetPlan {
                            target: local::roots(&defaults, &registry)
                                .iter()
                                .find(|r| r.id == root_id)
                                .map(|r| r.path.join(&p.folder_name))
                                .unwrap_or_default(),
                            root_id,
                            status: "blocked".into(),
                            current_digest: None,
                            changes: vec![],
                            current_source: None,
                            message,
                        },
                    },
                )
                .collect();
            value(plans)
        }
        Request::Install { id, selections } => {
            let p = install::prepared(&dir, &id)?;
            value(install::install_many(
                &dir,
                &defaults,
                &mut registry,
                &p,
                &selections,
            )?)
        }
        Request::Remove {
            path,
            root_id,
            expected_digest,
        } => {
            let result = install::remove(
                &dir,
                &defaults,
                &mut registry,
                &path,
                &root_id,
                &expected_digest,
            );
            record(&dir, &mut registry, path, &result, "已备份并移除此安装")?;
            value(result?)
        }
        Request::Restore { id } => {
            let path = registry
                .backups
                .iter()
                .find(|b| b.id == id)
                .map(|b| b.original.clone())
                .ok_or("备份不存在")?;
            let result = install::restore(&dir, &defaults, &mut registry, &id);
            record(&dir, &mut registry, path, &result, "已恢复备份到原位置")?;
            value(result?)
        }
        Request::SaveSource { input, remove } => {
            remote::parse(&input)?;
            registry.sources.retain(|s| s != &input);
            if !remove {
                if registry.sources.len() >= 50 {
                    return Err("最多登记 50 个公开来源".into());
                }
                registry.sources.push(input);
            }
            files::save_registry(&dir, &registry)?;
            value(registry.sources)
        }
        Request::Descriptions { .. }
        | Request::Catalog { .. }
        | Request::PrepareRemote { .. }
        | Request::CheckUpdate { .. } => Err("请求需要联网处理".into()),
        Request::Snapshot => value(local::scan(&defaults, &registry)),
        Request::Detail { path } => {
            let package = local::detail(&defaults, &registry, &path)?;
            let installed = registry.installations.get(&path);
            let modified = installed.is_some_and(|i| i.digest != package.digest);
            value(
                serde_json::json!({"package":package,"installation":installed,"modified":modified}),
            )
        }
        Request::ReadFile { path, file } => {
            local::detail(&defaults, &registry, &path)?;
            let relative = files::relative(&file)?;
            let raw = files::read_bounded(
                &path
                    .canonicalize()
                    .map_err(|e| e.to_string())?
                    .join(relative),
                1024 * 1024,
            )?;
            value(
                String::from_utf8(raw)
                    .map_err(|_| "这是二进制文件，请通过文件夹查看".to_string())?,
            )
        }
        Request::SaveRoot { root } => {
            let root = local::register(&mut registry, &defaults, root)?;
            files::save_registry(&dir, &registry)?;
            value(root)
        }
        Request::AddProject { path } => {
            let roots = local::project_roots(&path)?;
            for root in &roots {
                if !registry.roots.iter().any(|r| r.path == root.path) {
                    registry.roots.push(root.clone());
                }
            }
            files::save_registry(&dir, &registry)?;
            value(roots)
        }
        Request::ForgetRoot { id } => {
            registry.roots.retain(|r| r.id != id);
            files::save_registry(&dir, &registry)?;
            value(())
        }
        Request::OpenDirectory { path } => {
            files::absolute(&path)?;
            if !local::roots(&defaults, &registry)
                .iter()
                .any(|r| path.starts_with(&r.path))
                && !registry.backups.iter().any(|b| path == b.path)
            {
                return Err("目录不在已登记范围".into());
            }
            if !path.is_dir() {
                return Err("目录不存在".into());
            }
            open::that(path).map_err(|e| e.to_string())?;
            value(())
        }
    }
}
#[tauri::command]
pub async fn skills_command(
    app: AppHandle,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, SkillsState>,
    request: Request,
    request_id: String,
) -> skills::Result<serde_json::Value> {
    if window.label() != "main" {
        return Err("请从主窗口管理 Skills".into());
    }
    let _guard = state
        .gate
        .try_lock()
        .map_err(|_| "另一项 Skills 操作正在进行，请稍后重试")?;
    let dir = data_dir(&app)?;
    let defaults = defaults(&app)?;
    let cancel = Arc::new(AtomicBool::new(false));
    *state.active.lock().map_err(|e| e.to_string())? = Some((request_id.clone(), cancel.clone()));
    let progress = |completed, total, file: &str| {
        let _ = app.emit(
            "skills-progress",
            skills::Progress {
                request_id: request_id.clone(),
                phase: "download".into(),
                completed,
                total,
                file: file.into(),
            },
        );
    };
    let result = async {
        match request {
            Request::Descriptions { source, entries } => value(
                remote::Github::new()?
                    .descriptions(&source, entries, &cancel)
                    .await?,
            ),
            Request::Catalog { input } => {
                value(remote::Github::new()?.catalog(&input, &cancel).await?)
            }
            Request::PrepareRemote { source } => value(
                remote::Github::new()?
                    .prepare(&dir, source, &cancel, progress)
                    .await?,
            ),
            Request::CheckUpdate { path } => {
                let mut registry = files::load_registry(&dir)?;
                install::recover(&dir, &mut registry)?;
                local::detail(&defaults, &registry, &path)?;
                let installation = registry
                    .installations
                    .get(&path)
                    .ok_or("未记录此安装的来源")?
                    .clone();
                let prepared = remote::Github::new()?
                    .update(&dir, &installation, &cancel, progress)
                    .await?;
                if let Some(i) = registry.installations.get_mut(&path) {
                    i.checked_at = Some(skills::now());
                    i.upstream_commit = prepared.source.as_ref().map(|s| s.commit.clone());
                }
                files::save_registry(&dir, &registry)?;
                value(prepared)
            }
            request => {
                tauri::async_runtime::spawn_blocking(move || run_local(dir, defaults, request))
                    .await
                    .map_err(|e| e.to_string())?
            }
        }
    }
    .await;
    *state.active.lock().map_err(|e| e.to_string())? = None;
    result
}
#[tauri::command]
pub fn skills_cancel(state: tauri::State<'_, SkillsState>, request_id: String) {
    if let Ok(active) = state.active.lock() {
        if let Some((id, cancel)) = &*active {
            if *id == request_id {
                cancel.store(true, Ordering::Relaxed);
            }
        }
    }
}
#[tauri::command]
pub async fn skills_choose_directory(app: AppHandle) -> skills::Result<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title("选择 Skill、项目或扫描目录")
            .blocking_pick_folder()
            .map(|file| {
                file.into_path()
                    .map(|path| path.to_string_lossy().into_owned())
                    .map_err(|e| e.to_string())
            })
            .transpose()
    })
    .await
    .map_err(|e| e.to_string())?
}
