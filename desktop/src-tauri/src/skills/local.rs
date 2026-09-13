use super::*;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub fn default_roots(home: &Path, env: impl Fn(&str) -> Option<String>) -> Vec<Root> {
    let configured = |key: &str, fallback: PathBuf| {
        env(key)
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or(fallback)
    };
    let codex = configured("CODEX_HOME", home.join(".codex"));
    let claude = configured("CLAUDE_CONFIG_DIR", home.join(".claude"));
    let pi = configured("PI_CODING_AGENT_DIR", home.join(".pi/agent"));
    let config = configured("XDG_CONFIG_HOME", home.join(".config"));
    let definitions = [
        (
            "shared",
            "Codex / 标准共享",
            home.join(".agents/skills"),
            false,
            vec!["Codex", "Cursor", "OpenCode"],
        ),
        (
            "claude",
            "Claude Code",
            claude.join("skills"),
            false,
            vec!["Claude Code", "Cursor", "OpenCode"],
        ),
        (
            "cursor",
            "Cursor",
            home.join(".cursor/skills"),
            false,
            vec!["Cursor"],
        ),
        ("pi", "Pi", pi.join("skills"), false, vec!["Pi"]),
        (
            "opencode",
            "OpenCode",
            config.join("opencode/skills"),
            false,
            vec!["OpenCode"],
        ),
        (
            "codex-legacy",
            "Codex 兼容目录",
            codex.join("skills"),
            false,
            vec!["Codex", "Cursor"],
        ),
        (
            "codex-system",
            "Codex 系统 Skills",
            codex.join("skills/.system"),
            true,
            vec!["Codex"],
        ),
        (
            "codex-cache",
            "Codex 插件缓存",
            codex.join("plugins/cache"),
            true,
            vec!["Codex"],
        ),
        (
            "claude-cache",
            "Claude 插件缓存",
            claude.join("plugins/cache"),
            true,
            vec!["Claude Code"],
        ),
    ];
    definitions
        .into_iter()
        .map(|(id, name, path, readonly, shared)| Root {
            id: id.into(),
            name: name.into(),
            agent: if id.starts_with("codex") {
                "codex".into()
            } else if id.starts_with("claude") {
                "claude".into()
            } else {
                id.into()
            },
            scope: "global".into(),
            path,
            readonly,
            custom: false,
            shared_with: shared.into_iter().map(String::from).collect(),
            status: String::new(),
        })
        .collect()
}
pub fn roots(defaults: &[Root], registry: &Registry) -> Vec<Root> {
    defaults
        .iter()
        .chain(&registry.roots)
        .cloned()
        .map(|mut root| {
            root.status = match fs::symlink_metadata(&root.path) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => "目录不存在".into(),
                Err(e) => format!("不可读：{e}"),
                Ok(m) if m.file_type().is_symlink() => {
                    root.readonly = true;
                    "链接目录 · 只读".into()
                }
                Ok(m) if !m.is_dir() => "不是目录".into(),
                Ok(_) => {
                    if fs::read_dir(&root.path).is_err() {
                        "不可读".into()
                    } else if root.readonly {
                        "只读".into()
                    } else {
                        "目录已发现 · 客户端加载未验证".into()
                    }
                }
            };
            root
        })
        .collect()
}
pub fn protected(path: &Path, defaults: &[Root]) -> bool {
    defaults.iter().filter(|r| r.readonly).any(|r| {
        path.starts_with(&r.path)
            || path
                .canonicalize()
                .ok()
                .zip(r.path.canonicalize().ok())
                .is_some_and(|(p, r)| p.starts_with(r))
    })
}
pub fn register(registry: &mut Registry, defaults: &[Root], mut root: Root) -> Result<Root> {
    files::absolute(&root.path)?;
    if registry.roots.len() >= 200 && root.id.is_empty() {
        return Err("最多登记 200 个目录".into());
    }
    if !root.path.is_absolute() || !root.path.is_dir() {
        return Err("请选择现有目录".into());
    }
    if !["global", "project"].contains(&root.scope.as_str()) {
        return Err("作用范围无效".into());
    }
    if root.name.trim().is_empty() || root.name.len() > 120 {
        return Err("请填写 120 字节以内的目录名称".into());
    }
    if root.id.is_empty() {
        root.id = id();
    } else if !registry.roots.iter().any(|r| r.id == root.id) {
        return Err("只能修改已登记的自定义目录".into());
    }
    if ![
        "shared", "codex", "claude", "cursor", "pi", "opencode", "custom",
    ]
    .contains(&root.agent.as_str())
    {
        return Err("智能体类型无效".into());
    }
    root.shared_with = match root.agent.as_str() {
        "shared" => vec!["Codex", "Cursor", "OpenCode"],
        "claude" => vec!["Claude Code", "Cursor", "OpenCode"],
        "codex" => vec!["Codex", "Cursor"],
        "cursor" => vec!["Cursor"],
        "pi" => vec!["Pi"],
        "opencode" => vec!["OpenCode"],
        _ => vec![],
    }
    .into_iter()
    .map(String::from)
    .collect();
    root.custom = true;
    root.readonly |= protected(&root.path, defaults) || files::no_links(&root.path).is_err();
    if registry
        .roots
        .iter()
        .any(|r| r.id != root.id && r.path == root.path)
        || defaults.iter().any(|r| r.path == root.path)
    {
        return Err("该目录已经登记".into());
    }
    registry.roots.retain(|r| r.id != root.id);
    registry.roots.push(root.clone());
    Ok(root)
}
pub fn project_roots(project: &Path) -> Result<Vec<Root>> {
    files::no_links(project)?;
    if !project.is_dir() {
        return Err("项目目录不存在".into());
    }
    let definitions = [
        (
            "shared",
            "Codex / 标准共享",
            ".agents/skills",
            vec!["Codex", "Cursor", "OpenCode"],
        ),
        (
            "claude",
            "Claude Code",
            ".claude/skills",
            vec!["Claude Code", "Cursor", "OpenCode"],
        ),
        ("cursor", "Cursor", ".cursor/skills", vec!["Cursor"]),
        ("pi", "Pi", ".pi/skills", vec!["Pi"]),
        ("opencode", "OpenCode", ".opencode/skills", vec!["OpenCode"]),
    ];
    Ok(definitions
        .into_iter()
        .map(|(agent, name, suffix, shared)| Root {
            id: format!(
                "project-{}",
                files::digest(project.join(suffix).to_string_lossy().as_bytes())
            ),
            name: format!(
                "{} · {name}",
                project.file_name().unwrap_or_default().to_string_lossy()
            ),
            agent: agent.into(),
            scope: "project".into(),
            path: project.join(suffix),
            readonly: false,
            custom: true,
            shared_with: shared.into_iter().map(String::from).collect(),
            status: "目录适配方案 · 客户端加载未验证".into(),
        })
        .collect())
}
#[derive(Serialize)]
pub struct Scan {
    pub roots: Vec<Root>,
    pub skills: Vec<FoundSkill>,
    pub warnings: Vec<String>,
    pub backups: Vec<Backup>,
    pub operations: Vec<OperationResult>,
    pub sources: Vec<String>,
}
pub fn scan(defaults: &[Root], registry: &Registry) -> Scan {
    let roots = roots(defaults, registry);
    let mut skills = Vec::new();
    let mut warnings = Vec::new();
    let mut visited = 0;
    let mut seen = HashSet::new();
    fn walk(
        path: &Path,
        root: &Root,
        defaults: &[Root],
        registry: &Registry,
        depth: usize,
        visited: &mut usize,
        seen: &mut HashSet<(String, PathBuf)>,
        skills: &mut Vec<FoundSkill>,
        warnings: &mut Vec<String>,
    ) {
        if *visited >= MAX_SCAN || depth > MAX_DEPTH {
            if warnings.len() < 100 {
                warnings.push(format!("扫描达到数量或层级上限：{}", path.display()));
            }
            return;
        }
        *visited += 1;
        let meta = match fs::symlink_metadata(path) {
            Ok(v) => v,
            Err(e) => {
                if warnings.len() < 100 {
                    warnings.push(format!("{}：{e}", path.display()));
                }
                return;
            }
        };
        let link = meta.file_type().is_symlink();
        let physical = match path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                warnings.push(format!("无效路径或链接 {}：{e}", path.display()));
                return;
            }
        };
        if !seen.insert((root.id.clone(), physical.clone())) {
            if link {
                warnings.push(format!("跳过重复链接：{}", path.display()));
            }
            return;
        }
        if physical.join("SKILL.md").is_file() {
            let raw = match files::read_bounded(&physical.join("SKILL.md"), MAX_FILE) {
                Ok(raw) => raw,
                Err(e) => {
                    warnings.push(e);
                    return;
                }
            };
            let body = match String::from_utf8(raw) {
                Ok(body) => body,
                Err(_) => {
                    warnings.push(format!("SKILL.md 编码无效：{}", path.display()));
                    return;
                }
            };
            let fallback = path.file_name().unwrap_or_default().to_string_lossy();
            let (name, description, _, mut notes) = files::metadata(&body, &fallback);
            let installation = registry.installations.get(path);
            let readonly = root.readonly
                || link
                || protected(&physical, defaults)
                || files::no_links(path).is_err();
            let source = installation.and_then(|i| i.source.clone());
            let status = if root.id.contains("cache") {
                "缓存中发现，启用状态未知"
            } else if readonly {
                "外部管理 · 只读"
            } else if source.is_some() {
                "可检查更新"
            } else if installation.is_some() {
                "本地来源"
            } else {
                "外部安装"
            }
            .to_string();
            if link {
                notes.push(format!("链接目标：{}", physical.display()));
            }
            let key = source
                .as_ref()
                .map(Source::identity)
                .unwrap_or_else(|| physical.to_string_lossy().into_owned());
            skills.push(FoundSkill {
                key,
                path: path.into(),
                physical_path: physical,
                root_id: root.id.clone(),
                name,
                description,
                readonly,
                status,
                source,
                installed_digest: installation.map(|i| i.digest.clone()),
                warnings: notes,
                checked_at: installation.and_then(|i| i.checked_at),
                upstream_commit: installation.and_then(|i| i.upstream_commit.clone()),
            });
            return;
        }
        if link {
            warnings.push(format!("未递归扫描链接目录：{}", path.display()));
            return;
        }
        if !meta.is_dir() {
            return;
        }
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    if *visited >= MAX_SCAN {
                        warnings.push("扫描达到 20000 条目上限，结果可能不完整".into());
                        break;
                    }
                    match entry {
                        Ok(e) => {
                            let name = e.file_name();
                            let name = name.to_string_lossy();
                            if [".git", "node_modules", "target"].contains(&name.as_ref())
                                || name.starts_with(".cuetuck-")
                            {
                                continue;
                            }
                            if root.id == "codex-legacy" && name == ".system" {
                                continue;
                            }
                            walk(
                                &e.path(),
                                root,
                                defaults,
                                registry,
                                depth + 1,
                                visited,
                                seen,
                                skills,
                                warnings,
                            )
                        }
                        Err(e) => warnings.push(e.to_string()),
                    }
                }
            }
            Err(e) => warnings.push(format!("{}：{e}", path.display())),
        }
    }
    for root in &roots {
        if root.path.exists() || fs::symlink_metadata(&root.path).is_ok() {
            walk(
                &root.path,
                root,
                defaults,
                registry,
                0,
                &mut visited,
                &mut seen,
                &mut skills,
                &mut warnings,
            );
        }
    }
    skills.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.path.cmp(&b.path))
    });
    Scan {
        roots,
        skills,
        warnings,
        sources: registry.sources.clone(),
        backups: registry.backups.clone(),
        operations: registry
            .operations
            .iter()
            .rev()
            .take(100)
            .cloned()
            .collect(),
    }
}
pub fn detail(defaults: &[Root], registry: &Registry, path: &Path) -> Result<Package> {
    files::absolute(path)?;
    let root = roots(defaults, registry)
        .into_iter()
        .find(|r| path.starts_with(&r.path))
        .ok_or("目录未登记")?;
    if !path.starts_with(root.path) {
        return Err("目录不在扫描范围".into());
    }
    let actual = path.canonicalize().map_err(|e| e.to_string())?;
    files::package(&actual)
}
