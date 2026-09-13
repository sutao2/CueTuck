use super::*;
use reqwest::{Client, Url};
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub directory: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub loaded: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Catalog {
    pub repo: String,
    pub reference: String,
    pub commit: String,
    pub license: String,
    pub entries: Vec<Entry>,
}
#[derive(Clone, Debug)]
pub struct Location {
    pub repo: String,
    pub tail: Vec<String>,
}
#[derive(Deserialize)]
struct Tree {
    #[serde(default)]
    truncated: bool,
    tree: Vec<Node>,
}
#[derive(Deserialize)]
struct Node {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    kind: String,
    size: Option<u64>,
}

fn decode_path(input: &str) -> Result<String> {
    let mut bytes = Vec::new();
    let mut chars = input.as_bytes().iter();
    while let Some(&c) = chars.next() {
        if c == b'%' {
            let a = *chars.next().ok_or("地址编码不完整")? as char;
            let b = *chars.next().ok_or("地址编码不完整")? as char;
            let value = a
                .to_digit(16)
                .zip(b.to_digit(16))
                .map(|(a, b)| (a * 16 + b) as u8)
                .ok_or("地址编码无效")?;
            bytes.push(value);
        } else {
            bytes.push(c);
        }
    }
    String::from_utf8(bytes).map_err(|_| "地址不是有效 UTF-8".into())
}
pub fn parse(input: &str) -> Result<Location> {
    let input = input.trim();
    if input.len() > 2000 {
        return Err("仓库地址过长".into());
    }
    let raw = if input.starts_with("https://") {
        let url = Url::parse(input).map_err(|_| "GitHub 地址无效")?;
        if url.host_str() != Some("github.com")
            || url.port().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("仅支持公开 github.com 仓库或目录地址".into());
        }
        // Use the original path so URL normalization cannot erase traversal segments.
        decode_path(
            input
                .strip_prefix("https://")
                .unwrap()
                .split_once('/')
                .map(|(_, p)| p)
                .unwrap_or(""),
        )?
    } else {
        input.trim_matches('/').to_string()
    };
    let parts: Vec<_> = raw.trim_matches('/').split('/').collect();
    if parts
        .iter()
        .any(|p| *p == "." || *p == ".." || p.is_empty())
    {
        return Err("来源路径不可包含空段、.. 或 .".into());
    }
    if parts.len() < 2
        || parts[..2].iter().any(|p| {
            !p.chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
        })
    {
        return Err("请输入 owner/repo 或 GitHub 仓库目录地址".into());
    }
    let name = parts[1].strip_suffix(".git").unwrap_or(parts[1]);
    if name.is_empty() || [".", ".."].contains(&name) {
        return Err("仓库名称无效".into());
    }
    let repo = format!("{}/{}", parts[0], name);
    let tail = if parts.len() == 2 {
        vec![]
    } else {
        if parts.len() < 4 || !["tree", "blob"].contains(&parts[2]) {
            return Err("请使用 GitHub tree 或 blob 地址".into());
        }
        parts[3..].iter().map(|s| s.to_string()).collect()
    };
    for segment in &tail {
        files::relative(segment)?;
    }
    Ok(Location { repo, tail })
}
pub struct Github {
    client: Client,
    api: String,
    raw: String,
}
impl Github {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: crate::http::client_builder()?
                .user_agent("CueTuck-Skills/1")
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| e.to_string())?,
            api: "https://api.github.com".into(),
            raw: "https://raw.githubusercontent.com".into(),
        })
    }
    #[cfg(test)]
    pub fn test(base: &str) -> Self {
        Self {
            client: Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
            api: base.into(),
            raw: base.into(),
        }
    }
    fn url(&self, base: &str, segments: &[&str]) -> Result<Url> {
        let mut url = Url::parse(base).map_err(|e| e.to_string())?;
        url.path_segments_mut()
            .map_err(|_| "无效请求地址")?
            .extend(segments);
        Ok(url)
    }
    async fn bytes(
        &self,
        url: Url,
        max: u64,
        cancel: &AtomicBool,
        optional: bool,
    ) -> Result<Option<Vec<u8>>> {
        if cancel.load(Ordering::Relaxed) {
            return Err("操作已取消".into());
        }
        let mut response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("GitHub 连接失败：{e}"))?;
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND && optional {
            return Ok(None);
        }
        if status.as_u16() == 403 || status.as_u16() == 429 {
            return Err("GitHub 请求受限，请稍后重试；未将失败当成最新状态".into());
        }
        if !status.is_success() {
            return Err(format!("GitHub 返回 {status}；仅支持无需登录的公开仓库"));
        }
        if response.content_length().is_some_and(|n| n > max) {
            return Err("远程内容超过读取上限".into());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
            if cancel.load(Ordering::Relaxed) {
                return Err("操作已取消".into());
            }
            if bytes.len() as u64 + chunk.len() as u64 > max {
                return Err("远程内容超过读取上限".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(Some(bytes))
    }
    async fn json(
        &self,
        segments: &[&str],
        cancel: &AtomicBool,
        optional: bool,
    ) -> Result<Option<serde_json::Value>> {
        self.bytes(
            self.url(&self.api, segments)?,
            16 * 1024 * 1024,
            cancel,
            optional,
        )
        .await?
        .map(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))
        .transpose()
    }
    async fn commit(
        &self,
        repo: &str,
        reference: &str,
        cancel: &AtomicBool,
    ) -> Result<Option<String>> {
        let parts: Vec<_> = repo.split('/').collect();
        let value = self
            .json(
                &["repos", parts[0], parts[1], "commits", reference],
                cancel,
                true,
            )
            .await?;
        let Some(value) = value else { return Ok(None) };
        let sha = value["sha"].as_str().ok_or("GitHub 提交响应无效")?;
        if sha.len() != 40 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("GitHub 提交标识无效".into());
        }
        Ok(Some(sha.into()))
    }
    async fn resolve(&self, input: &str, cancel: &AtomicBool) -> Result<(Source, String)> {
        let location = parse(input)?;
        let parts: Vec<_> = location.repo.split('/').collect();
        let metadata = self
            .json(&["repos", parts[0], parts[1]], cancel, false)
            .await?
            .ok_or("仓库不存在")?;
        let repo = metadata["full_name"]
            .as_str()
            .unwrap_or(&location.repo)
            .to_string();
        parse(&repo)?;
        let license = metadata["license"]["spdx_id"]
            .as_str()
            .filter(|l| *l != "NOASSERTION")
            .unwrap_or("未知")
            .to_string();
        if location.tail.is_empty() {
            let reference = metadata["default_branch"]
                .as_str()
                .ok_or("仓库没有默认分支")?
                .to_string();
            let commit = self
                .commit(&repo, &reference, cancel)
                .await?
                .ok_or("仓库没有可用提交")?;
            return Ok((
                Source {
                    repo,
                    reference,
                    commit,
                    directory: String::new(),
                    license,
                },
                String::new(),
            ));
        }
        // A branch may contain slashes. Resolve actual refs before treating the suffix as a directory.
        if location.tail.len() > 24 {
            return Err("地址目录层级过深".into());
        }
        for split in (1..=location.tail.len()).rev() {
            let reference = location.tail[..split].join("/");
            if let Some(commit) = self.commit(&repo, &reference, cancel).await? {
                let mut directory = location.tail[split..].join("/");
                if directory == "SKILL.md" {
                    directory.clear();
                } else if directory.ends_with("/SKILL.md") {
                    directory.truncate(directory.len() - 9);
                }
                if !directory.is_empty() {
                    files::relative(&directory)?;
                }
                return Ok((
                    Source {
                        repo,
                        reference,
                        commit,
                        directory: directory.clone(),
                        license,
                    },
                    directory,
                ));
            }
        }
        Err("无法找到该分支或提交，请检查 GitHub 地址".into())
    }
    async fn tree(&self, source: &Source, cancel: &AtomicBool) -> Result<Tree> {
        let parts: Vec<_> = source.repo.split('/').collect();
        let mut url = self.url(
            &self.api,
            &["repos", parts[0], parts[1], "git", "trees", &source.commit],
        )?;
        url.query_pairs_mut().append_pair("recursive", "1");
        let tree: Tree = serde_json::from_slice(
            &self
                .bytes(url, 16 * 1024 * 1024, cancel, false)
                .await?
                .ok_or("目录列表为空")?,
        )
        .map_err(|e| e.to_string())?;
        if tree.truncated || tree.tree.len() > 50_000 {
            return Err("仓库文件树过大，GitHub 返回了不完整列表，无法安全安装".into());
        }
        Ok(tree)
    }
    pub async fn catalog(&self, input: &str, cancel: &AtomicBool) -> Result<Catalog> {
        let (source, directory) = self.resolve(input, cancel).await?;
        let tree = self.tree(&source, cancel).await?;
        let prefix = if directory.is_empty() {
            String::new()
        } else {
            format!("{directory}/")
        };
        let mut entries: Vec<Entry> = tree
            .tree
            .iter()
            .filter(|n| {
                n.kind == "blob"
                    && n.path.starts_with(&prefix)
                    && (n.path == "SKILL.md" || n.path.ends_with("/SKILL.md"))
            })
            .map(|n| {
                let directory = n.path.strip_suffix("/SKILL.md").unwrap_or("").to_string();
                Entry {
                    description: String::new(),
                    error: String::new(),
                    loaded: false,
                    name: directory
                        .rsplit('/')
                        .next()
                        .filter(|s| !s.is_empty())
                        .unwrap_or(&source.repo)
                        .into(),
                    directory,
                }
            })
            .collect();
        if entries.len() > 1000 {
            return Err("仓库超过 1000 个 Skills，请缩小来源目录范围".into());
        }
        entries.sort_by(|a, b| a.directory.cmp(&b.directory));
        Ok(Catalog {
            repo: source.repo,
            reference: source.reference,
            commit: source.commit,
            license: source.license,
            entries,
        })
    }
    pub async fn descriptions(
        &self,
        source: &Source,
        entries: Vec<Entry>,
        cancel: &AtomicBool,
    ) -> Result<Vec<Entry>> {
        parse(&source.repo)?;
        if source.commit.len() != 40
            || !source.commit.chars().all(|c| c.is_ascii_hexdigit())
            || entries.len() > 50
        {
            return Err("来源或分页参数无效".into());
        }
        async fn read(
            client: &Github,
            source: &Source,
            mut entry: Entry,
            cancel: &AtomicBool,
        ) -> Entry {
            let result = async {
                let directory = if entry.directory.is_empty() {
                    String::new()
                } else {
                    files::relative(&entry.directory)?;
                    format!("{}/", entry.directory)
                };
                let path = format!("{directory}SKILL.md");
                let mut parts: Vec<_> = source.repo.split('/').collect();
                parts.push(&source.commit);
                parts.extend(path.split('/'));
                let bytes = client
                    .bytes(client.url(&client.raw, &parts)?, MAX_FILE, cancel, false)
                    .await?
                    .ok_or("说明不存在")?;
                let body = String::from_utf8(bytes).map_err(|_| "说明不是 UTF-8")?;
                let (name, description, _, _) = files::metadata(&body, &entry.name);
                Ok::<_, String>((name, description))
            }
            .await;
            match result {
                Ok((name, description)) => {
                    entry.name = name;
                    entry.description = description;
                    entry.loaded = true;
                }
                Err(error) => entry.error = error,
            }
            entry
        }
        let mut result = Vec::new();
        for chunk in entries.chunks(4) {
            if cancel.load(Ordering::Relaxed) {
                return Err("操作已取消".into());
            }
            let a = chunk.first().cloned();
            let b = chunk.get(1).cloned();
            let c = chunk.get(2).cloned();
            let d = chunk.get(3).cloned();
            let fetch = |e: Option<Entry>| async move {
                if let Some(e) = e {
                    Some(read(self, source, e, cancel).await)
                } else {
                    None
                }
            };
            let (a, b, c, d) = tokio::join!(fetch(a), fetch(b), fetch(c), fetch(d));
            result.extend([a, b, c, d].into_iter().flatten());
        }
        Ok(result)
    }
    pub async fn prepare(
        &self,
        data: &Path,
        source: Source,
        cancel: &AtomicBool,
        progress: impl Fn(usize, usize, &str),
    ) -> Result<Prepared> {
        parse(&source.repo)?;
        if source.commit.len() != 40 || !source.commit.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("请重新加载来源以固定版本".into());
        }
        if !source.directory.is_empty() {
            files::relative(&source.directory)?;
        }
        let tree = self.tree(&source, cancel).await?;
        let prefix = if source.directory.is_empty() {
            String::new()
        } else {
            format!("{}/", source.directory)
        };
        let nodes: Vec<_> = tree
            .tree
            .iter()
            .filter(|n| n.path.starts_with(&prefix) && n.kind != "tree")
            .collect();
        if nodes.len() > MAX_FILES {
            return Err("Skill 包超过 1000 个文件".into());
        }
        let mut total = 0u64;
        for node in &nodes {
            let relative = node.path.strip_prefix(&prefix).ok_or("包路径无效")?;
            let path = files::relative(relative)?;
            if path.components().count() > MAX_DEPTH + 1 {
                return Err("Skill 包目录超过 16 层".into());
            }
            if node.kind != "blob" || !["100644", "100755"].contains(&node.mode.as_str()) {
                return Err(format!("包内包含链接、子模块或特殊条目：{relative}"));
            }
            let size = node.size.ok_or("远程文件大小未知，无法校验")?;
            if size > MAX_FILE {
                return Err("Skill 单文件超过 10 MiB".into());
            }
            total = total.checked_add(size).ok_or("包大小无效")?;
            if total > MAX_PACKAGE {
                return Err("Skill 包超过 25 MiB".into());
            }
        }
        fs::create_dir_all(data).map_err(|e| e.to_string())?;
        let temporary = tempfile::tempdir_in(data).map_err(|e| e.to_string())?;
        let folder = temporary.path().join("package");
        fs::create_dir(&folder).map_err(|e| e.to_string())?;
        let parts: Vec<_> = source.repo.split('/').collect();
        for (index, node) in nodes.iter().enumerate() {
            let relative = node.path.strip_prefix(&prefix).ok_or("包路径无效")?;
            progress(index, nodes.len(), relative);
            let mut segments = vec![parts[0], parts[1], source.commit.as_str()];
            segments.extend(node.path.split('/'));
            let bytes = self
                .bytes(self.url(&self.raw, &segments)?, MAX_FILE, cancel, false)
                .await?
                .ok_or("下载内容为空")?;
            if bytes.len() as u64 != node.size.unwrap_or(0) {
                return Err(format!("下载长度校验失败：{relative}"));
            }
            let path = folder.join(files::relative(relative)?);
            fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            use std::io::Write;
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|e| e.to_string())?;
            f.write_all(&bytes).map_err(|e| e.to_string())?;
            f.sync_all().map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    &path,
                    fs::Permissions::from_mode(if node.mode == "100755" { 0o755 } else { 0o644 }),
                )
                .map_err(|e| e.to_string())?;
            }
        }
        progress(nodes.len(), nodes.len(), "校验完整文件包");
        let name = source
            .directory
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or(parts[1])
            .to_string();
        install::store_prepared(data, name, &folder, Some(source), None)
    }
    pub async fn update(
        &self,
        data: &Path,
        installation: &Installation,
        cancel: &AtomicBool,
        progress: impl Fn(usize, usize, &str),
    ) -> Result<Prepared> {
        let mut source = installation
            .source
            .clone()
            .ok_or("该安装没有可验证的 GitHub 来源，请手动选择来源")?;
        source.commit = self
            .commit(&source.repo, &source.reference, cancel)
            .await?
            .ok_or("原分支已不存在，请手动选择新的来源")?;
        self.prepare(data, source, cancel, progress).await
    }
}
