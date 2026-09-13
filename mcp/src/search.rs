//! Ephemeral search index: the source library is always opened read-only.
use crate::{library_path, open_library};
use rusqlite::{params, params_from_iter, types::Value as SqlValue, Connection};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};

#[derive(Default)]
pub struct SearchCache {
    index: Option<Index>,
}
struct Index {
    path: PathBuf,
    identity: FileIdentity,
    source: Connection,
    data_version: i64,
    db: Connection,
}
#[derive(PartialEq)]
struct FileIdentity {
    modified: Option<SystemTime>,
    len: u64,
    #[cfg(unix)]
    inode: (u64, u64),
}
fn identity(path: &Path) -> Result<FileIdentity, String> {
    let meta = std::fs::metadata(path).map_err(|_| "本机库文件不存在或无法读取")?;
    Ok(FileIdentity {
        modified: meta.modified().ok(),
        len: meta.len(),
        #[cfg(unix)]
        inode: {
            use std::os::unix::fs::MetadataExt;
            (meta.dev(), meta.ino())
        },
    })
}
fn version(db: &Connection) -> rusqlite::Result<i64> {
    db.query_row("PRAGMA data_version", [], |r| r.get(0))
}
fn guard(db: &Connection, cancel: Arc<AtomicBool>, deadline: Instant) {
    db.progress_handler(
        1000,
        Some(move || cancel.load(Ordering::Relaxed) || Instant::now() >= deadline),
    );
}
// A two-character term becomes a three-character trigram (e.g. 写作 -> 写 作).
// Always verify the original literal too, including punctuation and whitespace.
fn fragments(text: &str) -> String {
    let mut result = String::with_capacity(text.len().saturating_mul(2));
    for c in text.chars() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push(c);
    }
    result
}

impl SearchCache {
    pub fn search(
        &mut self,
        dir: &Path,
        args: &Value,
        cancel: Arc<AtomicBool>,
    ) -> Result<Value, String> {
        let object = args.as_object().ok_or("arguments 必须为对象")?;
        if object
            .keys()
            .any(|k| !["query", "limit", "offset", "category_id", "model"].contains(&k.as_str()))
        {
            return Err("未知搜索参数".into());
        }
        let string = |name: &str, max: usize| -> Result<Option<&str>, String> {
            args.get(name)
                .map(|v| {
                    v.as_str()
                        .filter(|s| s.len() <= max)
                        .ok_or_else(|| format!("{name} 必须为不超过 {max} 字节的字符串"))
                })
                .transpose()
        };
        let query = string("query", 1200)?.unwrap_or("").trim().to_lowercase();
        let terms: Vec<_> = query.split_whitespace().collect();
        if terms.len() > 16 {
            return Err("query 最多包含 16 个关键词".into());
        }
        let integer = |name: &str, default, min, max| -> Result<i64, String> {
            match args.get(name) {
                None => Ok(default),
                Some(v) => v
                    .as_i64()
                    .filter(|n| (min..=max).contains(n))
                    .ok_or_else(|| format!("{name} 必须是 {min}–{max} 的整数")),
            }
        };
        let limit = integer("limit", 50, 1, 100)?;
        let offset = integer("offset", 0, 0, 100000)?;
        let category = string("category_id", 200)?;
        let model = string("model", 200)?;
        if cancel.load(Ordering::Relaxed) {
            return Err("搜索已取消".into());
        }
        let path = library_path(dir);
        let file = identity(&path)?;
        let fresh = self.index.as_ref().is_some_and(|i| {
            i.path == path && i.identity == file && version(&i.source).ok() == Some(i.data_version)
        });
        if !fresh {
            // Never serve stale results if rebuilding fails.
            self.index = None;
            self.index = Some(
                Index::build(dir, file, cancel.clone())
                    .map_err(|e| format!("搜索索引构建失败（可能已超时或取消）：{e}"))?,
            );
        }
        let index = self.index.as_ref().unwrap();
        guard(&index.db, cancel, Instant::now() + Duration::from_secs(2));
        index
            .page(&query, &terms, category, model, limit, offset)
            .map_err(|e| format!("搜索失败（可能已超时或取消）：{e}"))
    }
}
impl Index {
    fn build(dir: &Path, identity: FileIdentity, cancel: Arc<AtomicBool>) -> Result<Self, String> {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut source = open_library(dir)?;
        guard(&source, cancel.clone(), deadline);
        let data_version = version(&source).map_err(|e| e.to_string())?;
        let mut db = Connection::open_in_memory().map_err(|e| e.to_string())?;
        guard(&db, cancel.clone(), deadline);
        let mut fill = || -> rusqlite::Result<()> {
            db.execute_batch("CREATE TABLE docs(id TEXT, title TEXT, summary TEXT, title_key TEXT, body_key TEXT, category_id TEXT, model TEXT);
                CREATE VIRTUAL TABLE ft USING fts5(title, body, tokenize='trigram', content='');")?;
            let snapshot = source.transaction()?;
            let columns = snapshot
                .prepare("PRAGMA table_info(prompts)")?
                .query_map([], |r| r.get::<_, String>(1))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let column = |name: &str| {
                if columns.iter().any(|c| c == name) {
                    name.to_owned()
                } else {
                    "NULL".to_owned()
                }
            };
            let mut read = snapshot.prepare(&format!(
                "SELECT id,title,summary,content,{},{} FROM prompts WHERE deleted_at IS NULL",
                column("category_id"),
                column("model")
            ))?;
            let mut rows = read.query([])?;
            let tx = db.transaction()?;
            {
                let mut insert = tx.prepare("INSERT INTO docs VALUES(?1,?2,?3,?4,?5,?6,?7)")?;
                let mut fts = tx.prepare("INSERT INTO ft(rowid,title,body) VALUES(?1,?2,?3)")?;
                while let Some(row) = rows.next()? {
                    if cancel.load(Ordering::Relaxed) || Instant::now() >= deadline {
                        return Err(rusqlite::Error::SqliteFailure(
                            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_INTERRUPT),
                            None,
                        ));
                    }
                    let title: String = row.get(1)?;
                    let title_key = title.to_lowercase();
                    let body_key = row.get::<_, String>(3)?.to_lowercase();
                    insert.execute(params![
                        row.get::<_, String>(0)?,
                        title,
                        row.get::<_, Option<String>>(2)?,
                        title_key,
                        body_key,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?
                    ])?;
                    fts.execute(params![
                        tx.last_insert_rowid(),
                        fragments(&title_key),
                        fragments(&body_key)
                    ])?;
                }
            }
            tx.commit()?;
            db.execute_batch("CREATE INDEX docs_order ON docs(title,id);")?;
            Ok(())
        };
        fill().map_err(|e| e.to_string())?;
        source.progress_handler(0, None::<fn() -> bool>);
        Ok(Self {
            path: library_path(dir),
            identity,
            source,
            data_version,
            db,
        })
    }
    fn page(
        &self,
        query: &str,
        terms: &[&str],
        category: Option<&str>,
        model: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> rusqlite::Result<Value> {
        let matched = terms
            .iter()
            .filter(|t| t.chars().count() >= 2)
            .map(|t| format!("\"{}\"", fragments(t).replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" AND ");
        let mut values = Vec::<SqlValue>::new();
        let mut bind = |s: &str| {
            values.push(s.to_owned().into());
            format!("?{}", values.len())
        };
        let mut conditions = Vec::new();
        if !matched.is_empty() {
            conditions.push(format!("ft MATCH {}", bind(&matched)));
        }
        let mut title_hits = Vec::new();
        for term in terms {
            let p = bind(term);
            conditions.push(format!(
                "(instr(d.title_key,{p})>0 OR instr(d.body_key,{p})>0)"
            ));
            title_hits.push(format!("instr(d.title_key,{p})>0"));
        }
        for (field, value) in [("category_id", category), ("model", model)] {
            if let Some(value) = value {
                conditions.push(format!("d.{field}={}", bind(value)));
            }
        }
        let order = if terms.is_empty() {
            "d.title,d.id".to_owned()
        } else {
            let exact = bind(query);
            format!(
                "(d.title_key={exact}) DESC, ({}) DESC, {}, d.title,d.id",
                title_hits.join(" AND "),
                if matched.is_empty() {
                    "0+0"
                } else {
                    "bm25(ft,8.0,1.0)"
                }
            )
        };
        let from = if matched.is_empty() {
            "docs d"
        } else {
            "ft JOIN docs d ON d.rowid=ft.rowid"
        };
        let filter = if conditions.is_empty() {
            "1".to_owned()
        } else {
            conditions.join(" AND ")
        };
        let sql = format!("SELECT d.id,d.title,d.summary,d.category_id,d.model FROM {from} WHERE {filter} ORDER BY {order} LIMIT {} OFFSET {offset}", limit + 1);
        let mut items = self.db.prepare(&sql)?.query_map(params_from_iter(values), |r| Ok(json!({"id":r.get::<_,String>(0)?,"title":r.get::<_,String>(1)?,"summary":r.get::<_,Option<String>>(2)?,"category_id":r.get::<_,Option<String>>(3)?,"model":r.get::<_,Option<String>>(4)?})))?.collect::<rusqlite::Result<Vec<_>>>()?;
        let has_more = items.len() > limit as usize;
        items.truncate(limit as usize);
        Ok(
            json!({"items":items,"limit":limit,"offset":offset,"has_more":has_more,"next_offset":if has_more && offset+limit<=100000 { Some(offset+limit) } else { None }}),
        )
    }
}

#[cfg(test)]
mod interruption_tests {
    use super::*;

    #[test]
    fn sqlite_query_stops_when_cancellation_arrives() {
        let db = Connection::open_in_memory().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        guard(&db, cancel.clone(), Instant::now() + Duration::from_secs(5));
        let signal = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            cancel.store(true, Ordering::Relaxed);
        });
        let start = Instant::now();
        let result = db.query_row("WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<100000000) SELECT sum(x) FROM n", [], |r| r.get::<_,i64>(0));
        signal.join().unwrap();
        assert_eq!(
            result.unwrap_err().sqlite_error_code(),
            Some(rusqlite::ErrorCode::OperationInterrupted)
        );
        assert!(start.elapsed() < Duration::from_secs(2));
        guard(
            &db,
            Arc::new(AtomicBool::new(false)),
            Instant::now() + Duration::from_secs(2),
        );
        assert_eq!(
            db.query_row("SELECT 1", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn sqlite_query_stops_at_deadline() {
        let db = Connection::open_in_memory().unwrap();
        guard(&db, Arc::new(AtomicBool::new(false)), Instant::now());
        let result = db.query_row("WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<1000000) SELECT sum(x) FROM n", [], |r| r.get::<_,i64>(0));
        assert_eq!(
            result.unwrap_err().sqlite_error_code(),
            Some(rusqlite::ErrorCode::OperationInterrupted)
        );
    }
}
