mod backup;
mod auto_backup;
pub use auto_backup::{set_auto_backup_in_dir, start_auto_backup_worker};
mod categories;
mod collections;
mod prompts;
pub mod assets;
mod settings;
mod sync;
pub use sync::{apply_sync_changes, export_sync_changes, SyncChange};

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

pub use categories::{create_category_in_dir, delete_category_in_dir, list_categories_in_dir, CategoryRecord};
pub use collections::{
    remove_prompt_from_collection_in_dir, update_collection_in_dir, delete_collection_in_dir,
    add_prompt_to_collection_in_dir, collection_member_count, create_collection_in_dir,
    list_collection_members_in_dir, list_collections_in_dir, CollectionRecord,
};
pub use prompts::{
    import_downloaded_prompt_with_metadata,
    clear_prompt_use_in_dir, create_prompt_in_dir, create_prompt_in_dir_with_model, delete_prompt_in_dir,
    import_downloaded_prompt_in_dir, list_prompts_in_dir, prompt_deleted_at, prompt_use_count,
    record_prompt_use_in_dir, update_prompt_in_dir, update_prompt_in_dir_with_model,
    upsert_synced_prompt_in_dir, PromptRecord,
};
pub use backup::{backup_library_in_dir, export_library_zip_in_dir, restore_library_in_dir};
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
                deleted_at TEXT,
                author TEXT
            );
            CREATE TABLE IF NOT EXISTS prompt_assets (
                prompt_id TEXT NOT NULL, id TEXT NOT NULL,
                name TEXT NOT NULL, mime TEXT NOT NULL, data BLOB NOT NULL,
                position INTEGER NOT NULL,
                PRIMARY KEY (prompt_id, id)
            );
            ",
        )
        .map_err(|error| error.to_string())?;
    ensure_prompt_columns(&connection)?;
    for table in ["categories", "settings"] {
        if !table_columns(&connection, table)?.iter().any(|column| column == "updated_at") {
            connection.execute(&format!("ALTER TABLE {table} ADD COLUMN updated_at TEXT NOT NULL DEFAULT '0'"), [])
                .map_err(|error| error.to_string())?;
        }
    }
    if !table_columns(&connection, "categories")?.iter().any(|column| column == "deleted_at") {
        connection.execute("ALTER TABLE categories ADD COLUMN deleted_at TEXT", [])
            .map_err(|error| error.to_string())?;
    }
    seed_system_categories(&connection)?;
    for table in ["prompts", "collections", "categories", "settings"] {
        let latest: i64 = connection.query_row(&format!(
            "SELECT COALESCE(MAX(CASE WHEN CAST(updated_at AS INTEGER) >= 1000000000 AND CAST(updated_at AS INTEGER) < 100000000000 THEN CAST(updated_at AS INTEGER) * 1000 ELSE CAST(updated_at AS INTEGER) END), 0) FROM {table}"
        ), [], |row| row.get(0)).map_err(|error| error.to_string())?;
        observe_timestamp(latest.max(0) as u64);
    }
    Ok(DatabaseStatus::Ready.as_str().to_string())
}

fn ensure_prompt_columns(connection: &Connection) -> Result<(), String> {
    let existing = table_columns(connection, "prompts")?;
    let needed = [
        ("summary", "TEXT"),
        ("deleted_at", "TEXT"),
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
        ("author", "TEXT"),
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

pub(crate) fn timestamp_ms(raw: &str) -> u64 {
    let value = raw.parse::<u64>().unwrap_or(0);
    if (1_000_000_000..100_000_000_000).contains(&value) { value * 1000 } else { value }
}

pub(crate) fn now_millis() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    let previous = LAST_TIMESTAMP.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |last| Some(now.max(last + 1))).unwrap();
    now.max(previous + 1).to_string()
}

pub(crate) fn observe_timestamp(timestamp: u64) {
    LAST_TIMESTAMP.fetch_max(timestamp, Ordering::SeqCst);
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
