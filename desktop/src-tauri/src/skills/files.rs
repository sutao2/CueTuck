use super::*;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path},
};

pub fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
pub fn relative(path: &str) -> Result<PathBuf> {
    if path.is_empty()
        || path.contains('\\')
        || path.contains(':')
        || path.contains('\0')
        || path.starts_with('/')
        || path.split('/').any(|s| {
            s.is_empty()
                || s == "."
                || s == ".."
                || s.ends_with([' ', '.'])
                || s.chars().any(char::is_control)
        })
    {
        return Err(format!("不安全的文件路径：{path}"));
    }
    let p = PathBuf::from(path);
    if p.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err("文件路径越界".into());
    }
    for part in path.split('/') {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ]
        .contains(&stem.as_str())
        {
            return Err("不支持跨平台保留文件名".into());
        }
    }
    Ok(p)
}
pub fn absolute(path: &Path) -> Result<()> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err("请选择不含 .. 的绝对路径".into());
    }
    Ok(())
}
pub fn no_links(path: &Path) -> Result<()> {
    absolute(path)?;
    if !path.is_absolute() {
        return Err("请选择绝对目录路径".into());
    }
    let mut cursor = PathBuf::new();
    for c in path.components() {
        if matches!(c, Component::ParentDir | Component::CurDir) {
            return Err("目录路径不可包含 .. 或 .".into());
        }
        cursor.push(c);
        match fs::symlink_metadata(&cursor) {
            Ok(m) if m.file_type().is_symlink() => {
                #[cfg(target_os = "macos")]
                if [
                    ("/var", "/private/var"),
                    ("/tmp", "/private/tmp"),
                    ("/etc", "/private/etc"),
                ]
                .iter()
                .any(|(from, to)| {
                    cursor == Path::new(from)
                        && fs::canonicalize(&cursor).ok().as_deref() == Some(Path::new(to))
                }) {
                    continue;
                }
                return Err(format!("链接位置只读：{}", cursor.display()));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}
pub fn read_bounded(path: &Path, max: u64) -> Result<Vec<u8>> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err(format!("拒绝链接或特殊文件：{}", path.display()));
    }
    if meta.len() > max {
        return Err(format!("文件超过限额：{}", path.display()));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > max {
        return Err("读取文件超过限额".into());
    }
    Ok(bytes)
}
pub fn metadata(text: &str, fallback: &str) -> (String, String, String, Vec<String>) {
    let mut warnings = Vec::new();
    let normalized = text.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    let header = normalized
        .strip_prefix("---\n")
        .and_then(|s| s.find("\n---").map(|end| &s[..end]));
    if let Some(header) = header.filter(|s| s.len() <= 65536) {
        use yaml_rust2::scanner::{Scanner, TokenType};
        let mut depth = 0usize;
        let mut complexity = 0usize;
        for token in Scanner::new(header.chars()) {
            complexity += 1;
            match token.1 {
                TokenType::Alias(_) | TokenType::Anchor(_) => {
                    return (
                        fallback.into(),
                        String::new(),
                        "未知".into(),
                        vec!["元数据含 YAML 引用，仅显示原始正文".into()],
                    );
                }
                TokenType::BlockSequenceStart
                | TokenType::BlockMappingStart
                | TokenType::FlowSequenceStart
                | TokenType::FlowMappingStart => depth += 1,
                TokenType::BlockEnd | TokenType::FlowSequenceEnd | TokenType::FlowMappingEnd => {
                    depth = depth.saturating_sub(1)
                }
                _ => {}
            }
            if depth > 32 || complexity > 4096 {
                return (
                    fallback.into(),
                    String::new(),
                    "未知".into(),
                    vec!["元数据过于复杂，仅显示原始正文".into()],
                );
            }
        }
        match yaml_rust2::YamlLoader::load_from_str(header) {
            Ok(docs) if !docs.is_empty() => {
                let d = &docs[0];
                let name = d["name"].as_str().unwrap_or(fallback).to_string();
                let description = d["description"].as_str().unwrap_or("").trim().to_string();
                let license = d["license"].as_str().unwrap_or("未知").to_string();
                if description.is_empty() {
                    warnings.push("未声明描述".into());
                }
                return (name, description, license, warnings);
            }
            _ => warnings.push("YAML 元数据无法解析，正文仍可查看".into()),
        }
    } else {
        warnings.push("未找到有效 YAML 元数据".into());
    }
    (fallback.to_string(), String::new(), "未知".into(), warnings)
}
pub fn package(dir: &Path) -> Result<Package> {
    no_links(dir)?;
    let mut files = Vec::new();
    let mut bytes = 0;
    let mut seen = std::collections::HashSet::new();
    fn walk(
        root: &Path,
        dir: &Path,
        depth: usize,
        files: &mut Vec<FileEntry>,
        bytes: &mut u64,
        seen: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        if depth > MAX_DEPTH {
            return Err("Skill 包目录超过 16 层".into());
        }
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            let name = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("文件名不是 UTF-8")?
                .replace(std::path::MAIN_SEPARATOR, "/");
            relative(&name)?;
            if !seen.insert(name.to_lowercase()) {
                return Err("包内存在大小写冲突路径".into());
            }
            if seen.len() > 4000 {
                return Err("Skill 包超过 4000 个目录与文件条目".into());
            }
            if meta.file_type().is_symlink() {
                return Err(format!("包内链接不支持独立安装：{name}"));
            }
            if meta.is_dir() {
                if entry.file_name() == ".git" {
                    continue;
                }
                walk(root, &path, depth + 1, files, bytes, seen)?;
            } else if meta.is_file() {
                if files.len() >= MAX_FILES {
                    return Err("Skill 包超过 1000 个文件".into());
                }
                let data = read_bounded(&path, MAX_FILE)?;
                *bytes += data.len() as u64;
                if *bytes > MAX_PACKAGE {
                    return Err("Skill 包超过 25 MiB".into());
                }
                #[cfg(unix)]
                let executable = {
                    use std::os::unix::fs::PermissionsExt;
                    meta.permissions().mode() & 0o111 != 0
                };
                #[cfg(not(unix))]
                let executable = false;
                files.push(FileEntry {
                    path: name,
                    size: data.len() as u64,
                    digest: digest(&data),
                    executable,
                });
            } else {
                return Err(format!("包内存在特殊文件：{name}"));
            }
        }
        Ok(())
    }
    walk(dir, dir, 0, &mut files, &mut bytes, &mut seen)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let raw = read_bounded(&dir.join("SKILL.md"), MAX_FILE)?;
    let body = String::from_utf8(raw).map_err(|_| "SKILL.md 不是 UTF-8")?;
    let fallback = dir.file_name().and_then(|s| s.to_str()).unwrap_or("Skill");
    let (name, description, license, mut warnings) = metadata(&body, fallback);
    if body.contains("../") {
        warnings.push("说明包含包外相对路径，请确认外部依赖；只安装当前目录".into());
    }
    if body.contains("MCP") || body.contains("mcp") {
        warnings.push("说明提及 MCP，请在目标客户端配置其依赖".into());
    }
    let digest = digest(&serde_json::to_vec(&files).map_err(|e| e.to_string())?);
    Ok(Package {
        name,
        description,
        body,
        license,
        files,
        bytes,
        digest,
        warnings,
    })
}
pub fn copy_package(source: &Path, target: &Path, expected: &Package) -> Result<()> {
    if package(source)?.digest != expected.digest {
        return Err("来源内容已变化，请重新预览".into());
    }
    no_links(target)?;
    fs::create_dir_all(target).map_err(|e| e.to_string())?;
    for file in &expected.files {
        let relative = relative(&file.path)?;
        let data = read_bounded(&source.join(&relative), MAX_FILE)?;
        if digest(&data) != file.digest {
            return Err("复制过程中源文件发生变化".into());
        }
        let dest = target.join(relative);
        fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&dest)
            .map_err(|e| e.to_string())?;
        f.write_all(&data).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                &dest,
                fs::Permissions::from_mode(if file.executable { 0o755 } else { 0o644 }),
            )
            .map_err(|e| e.to_string())?;
        }
    }
    if package(target)?.digest != expected.digest {
        return Err("写入后的文件校验不一致".into());
    }
    Ok(())
}
pub fn save_registry(dir: &Path, registry: &Registry) -> Result<()> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut f = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut f, registry).map_err(|e| e.to_string())?;
    f.as_file().sync_all().map_err(|e| e.to_string())?;
    f.persist(dir.join("registry.json"))
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn load_registry(dir: &Path) -> Result<Registry> {
    let path = dir.join("registry.json");
    if !path.exists() {
        return Ok(Registry::default());
    }
    serde_json::from_slice(&read_bounded(&path, 10 * 1024 * 1024)?)
        .map_err(|e| format!("Skills 管理记录损坏，保留原文件：{e}"))
}
