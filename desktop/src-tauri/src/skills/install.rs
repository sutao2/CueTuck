use super::*;
use std::{fs, path::Path};

pub fn prepared_dir(data: &Path, key: &str) -> Result<PathBuf> {
    uuid::Uuid::parse_str(key).map_err(|_| "无效的预览标识")?;
    let path = data.join("staging").join(key);
    files::no_links(&path)?;
    Ok(path)
}
fn clean_previews(data: &Path) -> Result<()> {
    let staging = data.join("staging");
    files::no_links(&staging)?;
    if !staging.exists() {
        return Ok(());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(&staging).map_err(|e| e.to_string())? {
        let e = entry.map_err(|e| e.to_string())?;
        if e.file_type().map_err(|e| e.to_string())?.is_dir()
            && uuid::Uuid::parse_str(&e.file_name().to_string_lossy()).is_ok()
        {
            entries.push((
                e.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                e.path(),
            ));
        }
    }
    entries.sort_by(|a, b| b.0.cmp(&a.0));
    for (index, (modified, path)) in entries.into_iter().enumerate() {
        if index >= 7
            || modified
                .elapsed()
                .is_ok_and(|age| age.as_secs() > 24 * 60 * 60)
        {
            files::no_links(&path)?;
            fs::remove_dir_all(path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
pub fn store_prepared(
    data: &Path,
    folder_name: String,
    folder: &Path,
    source: Option<Source>,
    local_path: Option<PathBuf>,
) -> Result<Prepared> {
    if files::relative(&folder_name)?.components().count() != 1 {
        return Err("Skill 安装目录名无效".into());
    }
    let package = files::package(folder)?;
    clean_previews(data)?;
    let key = id();
    let dir = prepared_dir(data, &key)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let result = (|| {
        files::copy_package(folder, &dir.join("package"), &package)?;
        let prepared = Prepared {
            id: key,
            folder_name,
            package,
            source,
            local_path,
        };
        fs::write(
            dir.join("prepared.json"),
            serde_json::to_vec(&prepared).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok(prepared)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&dir);
    }
    result
}
pub fn prepare_local(data: &Path, path: &Path) -> Result<Prepared> {
    files::no_links(path)?;
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("请选择单个 Skill 文件夹")?
        .to_string();
    let current = files::package(path)?;
    let registry = files::load_registry(data)?;
    let source = registry
        .installations
        .get(path)
        .filter(|i| i.digest == current.digest)
        .and_then(|i| i.source.clone());
    store_prepared(data, name, path, source, Some(path.into()))
}
pub fn prepared(data: &Path, key: &str) -> Result<Prepared> {
    let dir = prepared_dir(data, key)?;
    let p: Prepared = serde_json::from_slice(&files::read_bounded(
        &dir.join("prepared.json"),
        MAX_FILE * 6 + 4 * 1024 * 1024,
    )?)
    .map_err(|e| e.to_string())?;
    if p.id != key
        || files::relative(&p.folder_name)?.components().count() != 1
        || files::package(&dir.join("package"))?.digest != p.package.digest
    {
        return Err("预览缓存发生变化，请重新预览".into());
    }
    Ok(p)
}
pub fn discard(data: &Path, key: &str) -> Result<()> {
    let dir = prepared_dir(data, key)?;
    if dir.exists() {
        fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn writable(defaults: &[Root], registry: &Registry, root_id: &str) -> Result<Root> {
    let root = local::roots(defaults, registry)
        .into_iter()
        .find(|r| r.id == root_id)
        .ok_or("目标目录未登记")?;
    if root.readonly || local::protected(&root.path, defaults) {
        return Err("该目标由外部管理，只允许查看".into());
    }
    files::no_links(&root.path)?;
    Ok(root)
}
pub fn changes(old: Option<&Package>, new: &Package) -> Vec<String> {
    let mut out = Vec::new();
    for f in &new.files {
        match old.and_then(|p| p.files.iter().find(|o| o.path == f.path)) {
            None => out.push(format!("+ {}", f.path)),
            Some(o) if o.digest != f.digest || o.executable != f.executable => {
                out.push(format!("~ {}", f.path))
            }
            _ => {}
        }
    }
    if let Some(old) = old {
        for f in &old.files {
            if !new.files.iter().any(|n| n.path == f.path) {
                out.push(format!("− {}", f.path));
            }
        }
    }
    out
}
pub fn plan(
    defaults: &[Root],
    registry: &Registry,
    p: &Prepared,
    root_id: &str,
) -> Result<TargetPlan> {
    let root = writable(defaults, registry, root_id)?;
    let target = root.path.join(files::relative(&p.folder_name)?);
    files::no_links(&target)?;
    if local::protected(&target, defaults) {
        return Err("目标位于系统 Skill 或插件缓存中，只允许查看".into());
    }
    let old = if target.exists() {
        Some(files::package(&target).map_err(|e| format!("目标内容无法安全备份：{e}"))?)
    } else {
        None
    };
    let installed = registry.installations.get(&target);
    let same_origin = match (&p.source, installed.and_then(|i| i.source.as_ref())) {
        (Some(a), Some(b)) => a.identity() == b.identity(),
        _ => false,
    };
    let same = old.as_ref().is_some_and(|o| o.digest == p.package.digest);
    let modified = installed
        .zip(old.as_ref())
        .is_some_and(|(i, o)| i.digest != o.digest);
    let status = if same
        && (same_origin || (p.source.is_none() && installed.is_some_and(|i| i.source.is_none())))
    {
        "same"
    } else if old.is_none() {
        "new"
    } else if modified {
        "modified"
    } else {
        "conflict"
    };
    Ok(TargetPlan {
        root_id: root.id,
        target,
        status: status.into(),
        current_digest: old.as_ref().map(|o| o.digest.clone()),
        changes: changes(old.as_ref(), &p.package),
        current_source: installed.and_then(|i| i.source.clone()),
        message: if same {
            "文件内容一致"
        } else if modified {
            "检测到本地修改；替换会先备份"
        } else if old.is_some() {
            "目标已存在；默认保留，需明确选择备份后替换"
        } else {
            "将创建独立副本"
        }
        .into(),
    })
}
fn backup(
    data: &Path,
    registry: &mut Registry,
    target: &Path,
    root_id: &str,
    reason: &str,
) -> Result<Backup> {
    let package = files::package(target)?;
    let key = id();
    let path = data.join("backups").join(&key).join("package");
    files::copy_package(target, &path, &package)?;
    let backup = Backup {
        id: key,
        path,
        original: target.into(),
        root_id: root_id.into(),
        created_at: now(),
        bytes: package.bytes,
        reason: reason.into(),
        digest: package.digest,
        installation: registry.installations.get(target).cloned(),
        restored: false,
    };
    registry.backups.push(backup.clone());
    files::save_registry(data, registry)?;
    Ok(backup)
}
fn commit(data: &Path, registry: &mut Registry, pending: Pending) -> Result<()> {
    files::no_links(&pending.target)?;
    files::no_links(&pending.stage)?;
    files::no_links(&pending.rollback)?;
    let current = if pending.target.exists() {
        Some(files::package(&pending.target)?.digest)
    } else {
        None
    };
    if current != pending.expected_digest {
        return Err("目标在确认后发生变化，已停止写入".into());
    }
    registry.pending = Some(pending.clone());
    files::save_registry(data, registry)?;
    let result = (|| {
        if pending.target.exists() {
            fs::rename(&pending.target, &pending.rollback).map_err(|e| e.to_string())?;
        }
        if !pending.remove {
            fs::rename(&pending.stage, &pending.target).map_err(|e| e.to_string())?;
        }
        if let Some(i) = &pending.installation {
            registry
                .installations
                .insert(pending.target.clone(), i.clone());
        } else {
            registry.installations.remove(&pending.target);
        }
        registry.pending = None;
        files::save_registry(data, registry)?;
        Ok(())
    })();
    if result.is_err() {
        // Reload the durable journal. Recovery either finishes a verified copy or restores the old directory.
        *registry = files::load_registry(data)?;
        recover(data, registry)?;
    } else {
        let _ = fs::remove_dir_all(&pending.rollback);
    }
    result
}
pub fn recover(data: &Path, registry: &mut Registry) -> Result<()> {
    let Some(p) = registry.pending.clone() else {
        return Ok(());
    };
    files::no_links(&p.target)?;
    files::no_links(&p.stage)?;
    files::no_links(&p.rollback)?;
    if p.target.parent() != p.stage.parent()
        || p.target.parent() != p.rollback.parent()
        || !p
            .stage
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .starts_with(".cuetuck-stage-")
        || !p
            .rollback
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .starts_with(".cuetuck-rollback-")
    {
        return Err("恢复日志路径无效，已保留文件".into());
    }
    let completed = if p.remove {
        !p.target.exists() && p.rollback.exists()
    } else {
        p.installation
            .as_ref()
            .is_some_and(|i| files::package(&p.target).is_ok_and(|v| v.digest == i.digest))
    };
    if completed {
        if let Some(i) = p.installation {
            registry.installations.insert(p.target.clone(), i);
        } else {
            registry.installations.remove(&p.target);
        }
    } else if !p.target.exists() && p.rollback.exists() {
        fs::rename(&p.rollback, &p.target).map_err(|e| e.to_string())?;
    } else if p.target.exists() && p.rollback.exists() {
        return Err("恢复遇到目录冲突，原目录与备份均已保留，请查看操作位置".into());
    }
    registry.pending = None;
    files::save_registry(data, registry)?;
    let _ = fs::remove_dir_all(&p.stage);
    let _ = fs::remove_dir_all(&p.rollback);
    Ok(())
}
#[derive(Clone, Debug, Deserialize)]
pub struct Selection {
    pub root_id: String,
    pub expected_digest: Option<String>,
    #[serde(default)]
    pub replace: bool,
}
pub fn install_one(
    data: &Path,
    defaults: &[Root],
    registry: &mut Registry,
    p: &Prepared,
    selection: &Selection,
) -> Result<PathBuf> {
    if registry.pending.is_some() {
        return Err("上次操作尚未恢复，请刷新后重试".into());
    }
    let plan = plan(defaults, registry, p, &selection.root_id)?;
    if plan.current_digest != selection.expected_digest {
        return Err("目标在预览后发生变化，请重新比较".into());
    }
    if plan.status == "same" {
        if let Some(i) = registry.installations.get_mut(&plan.target) {
            i.source = p.source.clone();
            i.checked_at = Some(now());
            i.upstream_commit = p.source.as_ref().map(|s| s.commit.clone());
        }
        files::save_registry(data, registry)?;
        return Ok(plan.target);
    }
    if plan.current_digest.is_some() && !selection.replace {
        return Err("已保留现有 Skill；替换需要明确确认".into());
    }
    if plan.current_digest.is_some() {
        backup(
            data,
            registry,
            &plan.target,
            &selection.root_id,
            "替换前备份",
        )?;
    }
    let parent = plan.target.parent().ok_or("无效目标")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let stage = parent.join(format!(".cuetuck-stage-{}", id()));
    let rollback = parent.join(format!(".cuetuck-rollback-{}", id()));
    if let Err(e) = files::copy_package(
        &prepared_dir(data, &p.id)?.join("package"),
        &stage,
        &p.package,
    ) {
        let _ = fs::remove_dir_all(&stage);
        return Err(e);
    }
    // Recheck after copying, immediately before mutation.
    let actual = if plan.target.exists() {
        Some(files::package(&plan.target)?.digest)
    } else {
        None
    };
    if actual != selection.expected_digest {
        let _ = fs::remove_dir_all(stage);
        return Err("目标内容刚刚发生变化，请重新比较".into());
    }
    let installed = Installation {
        path: plan.target.clone(),
        root_id: selection.root_id.clone(),
        digest: p.package.digest.clone(),
        source: p.source.clone(),
        installed_at: now(),
        checked_at: None,
        upstream_commit: None,
    };
    commit(
        data,
        registry,
        Pending {
            target: plan.target.clone(),
            stage,
            rollback,
            installation: Some(installed),
            remove: false,
            expected_digest: selection.expected_digest.clone(),
        },
    )?;
    Ok(plan.target)
}
pub fn install_many(
    data: &Path,
    defaults: &[Root],
    registry: &mut Registry,
    p: &Prepared,
    selections: &[Selection],
) -> Result<Vec<OperationResult>> {
    if selections.is_empty() || selections.len() > 200 {
        return Err("请选择 1–200 个安装位置".into());
    }
    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for selection in selections {
        let target = local::roots(defaults, registry)
            .into_iter()
            .find(|r| r.id == selection.root_id)
            .map(|r| r.path.join(&p.folder_name))
            .unwrap_or_else(|| PathBuf::from(&selection.root_id));
        let key = target
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .map(|parent| parent.join(&p.folder_name))
            .unwrap_or_else(|| target.clone());
        if !seen.insert(key) {
            continue;
        }
        let result = match install_one(data, defaults, registry, p, selection) {
            Ok(target) => OperationResult {
                target,
                status: "success".into(),
                message: "文件已就绪；请在目标客户端确认加载".into(),
                at: now(),
            },
            Err(message) => OperationResult {
                target,
                status: "failed".into(),
                message,
                at: now(),
            },
        };
        registry.operations.push(result.clone());
        results.push(result);
        registry.operations = registry
            .operations
            .iter()
            .rev()
            .take(100)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        files::save_registry(data, registry)?;
    }
    Ok(results)
}
pub fn remove(
    data: &Path,
    defaults: &[Root],
    registry: &mut Registry,
    path: &Path,
    root_id: &str,
    expected: &str,
) -> Result<Backup> {
    let root = writable(defaults, registry, root_id)?;
    files::no_links(path)?;
    if local::protected(path, defaults) {
        return Err("该位置由外部管理，只允许查看".into());
    }
    if path == root.path || !path.starts_with(&root.path) {
        return Err("只能移除登记根目录内的单个 Skill".into());
    }
    if files::package(path)?.digest != expected {
        return Err("内容已变化，请重新预览后移除".into());
    }
    let b = backup(data, registry, path, root_id, "移除前备份")?;
    let parent = path.parent().ok_or("无效目标")?;
    commit(
        data,
        registry,
        Pending {
            target: path.into(),
            stage: parent.join(format!(".cuetuck-stage-{}", id())),
            rollback: parent.join(format!(".cuetuck-rollback-{}", id())),
            installation: None,
            remove: true,
            expected_digest: Some(expected.into()),
        },
    )?;
    Ok(b)
}
pub fn restore(
    data: &Path,
    defaults: &[Root],
    registry: &mut Registry,
    key: &str,
) -> Result<PathBuf> {
    let b = registry
        .backups
        .iter()
        .find(|b| b.id == key)
        .cloned()
        .ok_or("备份不存在")?;
    files::no_links(&b.original)?;
    if local::protected(&b.original, defaults) {
        return Err("目标是受保护目录".into());
    }
    if b.original.exists() {
        return Err("原位置已有文件，恢复不会覆盖；请先移除或移走现有目录".into());
    }
    let package = files::package(&b.path)?;
    if package.digest != b.digest {
        return Err("备份校验失败，已保留文件".into());
    }
    let parent = b.original.parent().ok_or("无效目标")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let stage = parent.join(format!(".cuetuck-stage-{}", id()));
    files::copy_package(&b.path, &stage, &package)?;
    let installation = b.installation.clone().unwrap_or(Installation {
        path: b.original.clone(),
        root_id: b.root_id.clone(),
        digest: b.digest.clone(),
        source: None,
        installed_at: now(),
        checked_at: None,
        upstream_commit: None,
    });
    commit(
        data,
        registry,
        Pending {
            target: b.original.clone(),
            stage,
            rollback: parent.join(format!(".cuetuck-rollback-{}", id())),
            installation: Some(installation),
            remove: false,
            expected_digest: None,
        },
    )?;
    if let Some(backup) = registry.backups.iter_mut().find(|item| item.id == key) {
        backup.restored = true;
    }
    files::save_registry(data, registry)?;
    Ok(b.original)
}
