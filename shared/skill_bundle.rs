//! Wire format shared by the API and native installer; contains files, never commands.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
pub const MAX_BUNDLE_JSON: usize = 36 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub name: String,
    pub files: Vec<BundleFile>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleFile {
    pub path: String,
    pub content: String,
    pub executable: bool,
}
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && valid_path(name)
}
pub fn valid_path(path: &str) -> bool {
    if path.len() > 1024
        || path.split('/').count() > 16
        || path
            .chars()
            .any(|c| c.is_control() || "\\:*?\"<>|".contains(c))
    {
        return false;
    }
    path.split('/').all(|p| {
        let stem = p.split('.').next().unwrap_or("").to_ascii_uppercase();
        !p.is_empty()
            && !matches!(p, "." | "..")
            && !p.ends_with([' ', '.'])
            && !matches!(p.to_ascii_lowercase().as_str(), ".git" | ".env")
            && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            && !(stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.as_bytes()[3].is_ascii_digit())
    })
}
impl Bundle {
    pub fn validate(&self) -> Result<Vec<Vec<u8>>, String> {
        if !valid_name(&self.name) || self.files.is_empty() || self.files.len() > 1000 {
            return Err("Skill 名称或文件数量无效".into());
        }
        let mut paths = BTreeSet::new();
        let mut decoded = Vec::new();
        let mut total = 0usize;
        let mut has_skill = false;
        for file in &self.files {
            let key = file.path.to_lowercase();
            if !valid_path(&file.path) || !paths.insert(key) {
                return Err("文件路径无效或存在大小写重名".into());
            }
            if file.content.len() > 14 * 1024 * 1024 {
                return Err("单文件超过 10 MiB".into());
            }
            let bytes = STANDARD.decode(&file.content).map_err(|_| "文件编码无效")?;
            total += bytes.len();
            if bytes.len() > 10 * 1024 * 1024 || total > 25 * 1024 * 1024 {
                return Err("文件包超过大小上限".into());
            }
            if file.path == "SKILL.md" {
                let text = std::str::from_utf8(&bytes).map_err(|_| "SKILL.md 必须为 UTF-8 文本")?;
                if text.trim().is_empty() || text.contains('\0') {
                    return Err("SKILL.md 正文为空或无效".into());
                }
                has_skill = true;
            }
            decoded.push(bytes);
        }
        for path in &paths {
            let mut parent = path.as_str();
            while let Some((p, _)) = parent.rsplit_once('/') {
                if paths.contains(p) {
                    return Err("文件与目录路径冲突".into());
                }
                parent = p;
            }
        }
        if !has_skill {
            return Err("文件包缺少 SKILL.md".into());
        }
        Ok(decoded)
    }
    pub fn digest(&self) -> String {
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(self).expect("serializable bundle"))
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn bundle() -> Bundle {
        Bundle {
            name: "my-skill".into(),
            files: vec![BundleFile {
                path: "SKILL.md".into(),
                content: STANDARD.encode("# 测试"),
                executable: false,
            }],
        }
    }
    #[test]
    fn rejects_unsafe_paths_and_collisions() {
        for path in [
            "../a",
            "/etc/passwd",
            "C:/a",
            "a\\b",
            "a/NUL.txt",
            "a./b",
            ".git/config",
            "a//b",
        ] {
            assert!(!valid_path(path), "{path}");
        }
        let mut b = bundle();
        b.files.push(BundleFile {
            path: "skill.md".into(),
            ..b.files[0].clone()
        });
        assert!(b.validate().is_err());
        b.files[1].path = "SKILL.md/child".into();
        assert!(b.validate().is_err());
    }
    #[test]
    fn validates_full_bytes_and_immutable_digest() {
        let mut b = bundle();
        b.files.push(BundleFile {
            path: "scripts/run.sh".into(),
            content: STANDARD.encode([0, 1, 2, 255]),
            executable: true,
        });
        assert_eq!(b.validate().unwrap()[1], vec![0, 1, 2, 255]);
        let old = b.digest();
        b.files[1].executable = false;
        assert_ne!(old, b.digest());
    }
    #[test]
    fn rejects_missing_invalid_and_large_content() {
        let mut b = bundle();
        b.files[0].content = STANDARD.encode([255]);
        assert!(b.validate().is_err());
        b.files[0].content = "?".into();
        assert!(b.validate().is_err());
        b.files[0].content = "A".repeat(14 * 1024 * 1024 + 1);
        assert!(b.validate().is_err());
        assert!(!valid_name("CON"));
        assert!(!valid_name("../skill"));
    }
}
