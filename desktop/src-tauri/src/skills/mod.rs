pub mod files;
pub mod install;
pub mod local;
pub mod remote;

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

pub const MAX_FILE: u64 = 10 * 1024 * 1024;
pub const MAX_PACKAGE: u64 = 25 * 1024 * 1024;
pub const MAX_FILES: usize = 1000;
pub const MAX_DEPTH: usize = 16;
pub const MAX_SCAN: usize = 20_000;
pub type Result<T> = std::result::Result<T, String>;
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Root {
    pub id: String,
    pub name: String,
    pub agent: String,
    pub scope: String,
    pub path: PathBuf,
    pub readonly: bool,
    pub custom: bool,
    #[serde(default)]
    pub shared_with: Vec<String>,
    #[serde(default)]
    pub status: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Source {
    pub repo: String,
    pub reference: String,
    pub commit: String,
    pub directory: String,
    #[serde(default)]
    pub license: String,
}
impl Source {
    pub fn identity(&self) -> String {
        format!("{}:{}", self.repo.to_lowercase(), self.directory)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub digest: String,
    pub executable: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub body: String,
    pub license: String,
    pub files: Vec<FileEntry>,
    pub bytes: u64,
    pub digest: String,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Installation {
    pub path: PathBuf,
    pub root_id: String,
    pub digest: String,
    pub source: Option<Source>,
    pub installed_at: u64,
    #[serde(default)]
    pub checked_at: Option<u64>,
    #[serde(default)]
    pub upstream_commit: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoundSkill {
    pub key: String,
    pub path: PathBuf,
    pub physical_path: PathBuf,
    pub root_id: String,
    pub name: String,
    pub description: String,
    pub readonly: bool,
    pub status: String,
    pub source: Option<Source>,
    pub installed_digest: Option<String>,
    pub warnings: Vec<String>,
    pub checked_at: Option<u64>,
    pub upstream_commit: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
    pub path: PathBuf,
    pub original: PathBuf,
    pub root_id: String,
    pub created_at: u64,
    pub bytes: u64,
    pub reason: String,
    pub digest: String,
    pub installation: Option<Installation>,
    #[serde(default)]
    pub restored: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pending {
    pub target: PathBuf,
    pub stage: PathBuf,
    pub rollback: PathBuf,
    pub installation: Option<Installation>,
    pub remove: bool,
    pub expected_digest: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default)]
    pub roots: Vec<Root>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub installations: BTreeMap<PathBuf, Installation>,
    #[serde(default)]
    pub backups: Vec<Backup>,
    #[serde(default)]
    pub pending: Option<Pending>,
    #[serde(default)]
    pub operations: Vec<OperationResult>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationResult {
    pub target: PathBuf,
    pub status: String,
    pub message: String,
    pub at: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub request_id: String,
    pub phase: String,
    pub completed: usize,
    pub total: usize,
    pub file: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prepared {
    pub id: String,
    pub folder_name: String,
    pub package: Package,
    pub source: Option<Source>,
    pub local_path: Option<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetPlan {
    pub root_id: String,
    pub target: PathBuf,
    pub status: String,
    pub current_digest: Option<String>,
    pub changes: Vec<String>,
    pub current_source: Option<Source>,
    pub message: String,
}

#[cfg(test)]
mod tests;
