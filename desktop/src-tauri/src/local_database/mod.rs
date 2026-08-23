mod backup;
mod categories;
mod collections;
mod prompts;
mod settings;

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub use categories::{create_category_in_dir, list_categories_in_dir, CategoryRecord};
pub use collections::{
    add_prompt_to_collection_in_dir, collection_member_count, create_collection_in_dir,
    list_collection_members_in_dir, list_collections_in_dir, CollectionRecord,
};
pub use prompts::{
    create_prompt_in_dir, delete_prompt_in_dir, import_downloaded_prompt_in_dir, list_prompts_in_dir,
    prompt_deleted_at, prompt_use_count, record_prompt_use_in_dir, update_prompt_in_dir, PromptRecord,
};
pub use backup::{backup_library_in_dir, restore_library_in_dir};
pub use settings::{
    apply_import_json_in_dir, export_library_json_in_dir, get_setting_in_dir,
    preview_import_json_in_dir, set_setting_in_dir, ImportPreview,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DatabaseStatus {
    Pending,
    Ready,
    Failed,
}

impl DatabaseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

pub struct LocalDatabase {
    status: Mutex<DatabaseStatus>,
}

impl Default for LocalDatabase {
    fn default() -> Self {
        Self {
            status: Mutex::new(DatabaseStatus::Pending),
        }
    }
}

impl LocalDatabase {
    pub fn status(&self) -> DatabaseStatus {
        self.status
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or(DatabaseStatus::Failed)
    }

    pub fn initialize(&self, dir: &Path) -> Result<String, String> {
        match initialize_in_dir(dir) {
            Ok(status) => {
                *self.status.lock().map_err(|error| error.to_string())? = DatabaseStatus::Ready;
                Ok(status)
            }
            Err(error) => {
                if let Ok(mut status) = self.status.lock() {
                    *status = DatabaseStatus::Failed;
                }
                Err(error)
            }
        }
    }
}

const SYSTEM_CATEGORIES: &[(&str, &str, &[&str])] = &[
    ("cat-software", "软件开发", &["网站开发", "前端工程", "后端与数据库", "测试与审查"]),
    ("cat-image", "图片生成", &["人像摄影", "商品视觉", "插画与海报"]),
    ("cat-video", "视频创作", &["分镜脚本", "短视频"]),
    ("cat-office", "办公效率", &["PPT 制作", "数据表格", "会议与邮件"]),
    ("cat-writing", "内容写作", &["社交媒体", "长文写作", "SEO"]),
    ("cat-product", "产品设计", &["PRD 与需求", "竞品分析", "用户研究"]),
    ("cat-marketing", "市场营销", &["品牌与广告", "增长运营", "销售话术"]),
    ("cat-data", "数据分析", &["SQL 与清洗", "业务洞察", "可视化"]),
    ("cat-education", "教育学习", &["课程与教案", "私人导师", "论文与研究"]),
    ("cat-life", "生活助手", &["旅行规划", "饮食与健身", "求职成长"]),
];

pub fn initialize_in_dir(dir: &Path) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = dir.join("promptark.sqlite");
    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value_json TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS categories (
                id TEXT PRIMARY KEY,
                parent_id TEXT,
                name TEXT NOT NULL,
                icon TEXT,
                is_system INTEGER NOT NULL DEFAULT 1,
                sort_order INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                category_id TEXT,
                cover_type TEXT NOT NULL DEFAULT 'none',
                cover_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT,
                updated_at TEXT,
                deleted_at TEXT
            );
            CREATE TABLE IF NOT EXISTS prompts (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                summary TEXT,
                content TEXT NOT NULL DEFAULT '',
                category_id TEXT,
                collection_id TEXT,
                model TEXT,
                source TEXT NOT NULL DEFAULT 'local',
                remote_id TEXT,
                version INTEGER NOT NULL DEFAULT 1,
                use_count INTEGER NOT NULL DEFAULT 0,
                last_used_at TEXT,
                created_at TEXT,
                updated_at TEXT,
                deleted_at TEXT
            );
            ",
        )
        .map_err(|error| error.to_string())?;
    ensure_prompt_columns(&connection)?;
    seed_system_categories(&connection)?;
    Ok(DatabaseStatus::Ready.as_str().to_string())
}

fn ensure_prompt_columns(connection: &Connection) -> Result<(), String> {
    let existing = table_columns(connection, "prompts")?;
    let needed = [
        ("category_id", "TEXT"),
        ("collection_id", "TEXT"),
        ("model", "TEXT"),
        ("source", "TEXT NOT NULL DEFAULT 'local'"),
        ("remote_id", "TEXT"),
        ("version", "INTEGER NOT NULL DEFAULT 1"),
        ("use_count", "INTEGER NOT NULL DEFAULT 0"),
        ("last_used_at", "TEXT"),
        ("created_at", "TEXT"),
        ("updated_at", "TEXT"),
    ];
    for (name, ddl) in needed {
        if !existing.iter().any(|column| column == name) {
            connection
                .execute(
                    &format!("ALTER TABLE prompts ADD COLUMN {name} {ddl}"),
                    [],
                )
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn table_columns(connection: &Connection, table: &str) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|error| error.to_string())?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(columns)
}

fn seed_system_categories(connection: &Connection) -> Result<(), String> {
    for (index, (id, name, children)) in SYSTEM_CATEGORIES.iter().enumerate() {
        connection
            .execute(
                "INSERT OR IGNORE INTO categories (id, parent_id, name, icon, is_system, sort_order)
                 VALUES (?1, NULL, ?2, NULL, 1, ?3)",
                rusqlite::params![id, name, index as i64],
            )
            .map_err(|error| error.to_string())?;
        for (child_index, child) in children.iter().enumerate() {
            let child_id = format!("{id}-{child_index}");
            connection
                .execute(
                    "INSERT OR IGNORE INTO categories (id, parent_id, name, icon, is_system, sort_order)
                     VALUES (?1, ?2, ?3, NULL, 1, ?4)",
                    rusqlite::params![child_id, id, child, child_index as i64],
                )
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

pub fn list_system_category_names(dir: &Path) -> Result<Vec<String>, String> {
    let path = dir.join("promptark.sqlite");
    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT name FROM categories
             WHERE parent_id IS NULL AND is_system = 1
             ORDER BY sort_order",
        )
        .map_err(|error| error.to_string())?;
    let names = statement
        .query_map([], |row| row.get(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(names)
}

pub fn count_local_prompts_in_dir(dir: &Path) -> Result<i64, String> {
    let path = dir.join("promptark.sqlite");
    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM prompts WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    Ok(count)
}

#[cfg(test)]
mod tests;
