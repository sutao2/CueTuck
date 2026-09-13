use promptark_mcp::{library_path, search::SearchCache};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::sync::{atomic::AtomicBool, Arc};
use tempfile::tempdir;

fn seed(path: &std::path::Path) -> Connection {
    let db = Connection::open(library_path(path)).unwrap();
    db.execute_batch("CREATE TABLE prompts(id TEXT PRIMARY KEY,title TEXT,summary TEXT,content TEXT,category_id TEXT,model TEXT,deleted_at TEXT);
        INSERT INTO prompts VALUES('exact','写作',NULL,'自然光 portrait','photo','gpt',NULL);
        INSERT INTO prompts VALUES('title','写作技巧',NULL,'自然光 PORTRAIT','photo','gpt',NULL);
        INSERT INTO prompts VALUES('body','AAA',NULL,'写作 自然光 portrait','text','claude',NULL);
        INSERT INTO prompts VALUES('partial','写作',NULL,'缺少另一关键词','text','gpt',NULL);
        INSERT INTO prompts VALUES('gone','写作',NULL,'自然光 portrait','photo','gpt','1');").unwrap();
    db
}
fn search(cache: &mut SearchCache, path: &std::path::Path, args: Value) -> Value {
    cache
        .search(path, &args, Arc::new(AtomicBool::new(false)))
        .unwrap()
}
fn ids(page: &Value) -> Vec<&str> {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_str().unwrap())
        .collect()
}

#[test]
fn chinese_two_char_multiterm_and_title_ranking() {
    let dir = tempdir().unwrap();
    let _db = seed(dir.path());
    let mut cache = SearchCache::default();
    let page = search(
        &mut cache,
        dir.path(),
        json!({"query":"写作 自然光 PORTRAIT"}),
    );
    assert_eq!(ids(&page).len(), 3);
    assert!(!ids(&page).contains(&"partial"));
    let page = search(&mut cache, dir.path(), json!({"query":"写作"}));
    assert_eq!(&ids(&page)[2..], ["title", "body"]);
    let mut exact = ids(&page)[..2].to_vec();
    exact.sort();
    assert_eq!(exact, ["exact", "partial"]);
    assert_eq!(
        ids(&search(&mut cache, dir.path(), json!({"query":"写"}))),
        ["exact", "partial", "title", "body"]
    );
    assert!(ids(&search(
        &mut cache,
        dir.path(),
        json!({"query":"缺少 自然光"})
    ))
    .is_empty());
}
#[test]
fn literal_punctuation_and_unicode_are_not_fts_syntax() {
    let dir = tempdir().unwrap();
    let db = seed(dir.path());
    let mut cache = SearchCache::default();
    db.execute(
        "INSERT INTO prompts(id,title,content) VALUES('literal',?1,?2)",
        params!["50%_ C++ \"引号\" café 🦊🙂", "a b"],
    )
    .unwrap();
    for q in ["50%_", "%", "_", "C++", "\"引号\"", "CAFÉ", "🦊🙂"] {
        assert_eq!(
            ids(&search(&mut cache, dir.path(), json!({"query":q}))),
            ["literal"],
            "{q}"
        );
    }
    assert_eq!(
        ids(&search(&mut cache, dir.path(), json!({"query":"OR"}))).len(),
        3
    ); // Literal substring of portrait.
    for q in ["NOT", "title:写作", "a\"b", "不存在"] {
        assert!(
            ids(&search(&mut cache, dir.path(), json!({"query":q}))).is_empty(),
            "{q}"
        );
    }
}
#[test]
fn filters_and_pagination_are_stable_and_bounded() {
    let dir = tempdir().unwrap();
    let _db = seed(dir.path());
    let mut cache = SearchCache::default();
    let first = search(
        &mut cache,
        dir.path(),
        json!({"category_id":"photo","model":"gpt","limit":1}),
    );
    let second = search(
        &mut cache,
        dir.path(),
        json!({"category_id":"photo","model":"gpt","limit":1,"offset":first["next_offset"]}),
    );
    assert_eq!(ids(&first), ["exact"]);
    assert_eq!(ids(&second), ["title"]);
    assert_eq!(first["has_more"], true);
    assert_eq!(second["has_more"], false);
    assert!(second["next_offset"].is_null());
    assert!(ids(&search(
        &mut cache,
        dir.path(),
        json!({"category_id":"photo","model":"claude"})
    ))
    .is_empty());
}
#[test]
fn cached_index_observes_wal_insert_update_and_soft_delete_without_writes() {
    let dir = tempdir().unwrap();
    let db = seed(dir.path());
    db.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
    let mut cache = SearchCache::default();
    assert!(ids(&search(&mut cache, dir.path(), json!({"query":"新增"}))).is_empty());
    db.execute_batch("INSERT INTO prompts(id,title,content) VALUES('new','新增',''); UPDATE prompts SET title='更改' WHERE id='exact'; UPDATE prompts SET deleted_at='1' WHERE id='body';").unwrap();
    let path = library_path(dir.path());
    let wal = path.with_extension("sqlite-wal");
    let before = std::fs::read(&path).unwrap();
    let wal_before = std::fs::read(&wal).unwrap();
    assert_eq!(
        ids(&search(&mut cache, dir.path(), json!({"query":"新增"}))),
        ["new"]
    );
    assert_eq!(
        ids(&search(&mut cache, dir.path(), json!({"query":"更改"}))),
        ["exact"]
    );
    assert_eq!(
        ids(&search(&mut cache, dir.path(), json!({"query":"写作"}))),
        ["partial", "title"]
    );
    assert_eq!(std::fs::read(path).unwrap(), before);
    assert_eq!(std::fs::read(wal).unwrap(), wal_before);
}
#[test]
fn replacement_missing_and_corrupt_library_never_serve_old_index() {
    let dir = tempdir().unwrap();
    drop(seed(dir.path()));
    let mut cache = SearchCache::default();
    assert!(!ids(&search(&mut cache, dir.path(), json!({}))).is_empty());
    let replacement = tempdir().unwrap();
    let db = seed(replacement.path());
    db.execute_batch("DELETE FROM prompts;").unwrap();
    drop(db);
    std::fs::rename(library_path(replacement.path()), library_path(dir.path())).unwrap();
    assert!(ids(&search(&mut cache, dir.path(), json!({}))).is_empty());
    std::fs::remove_file(library_path(dir.path())).unwrap();
    assert!(cache
        .search(dir.path(), &json!({}), Arc::new(AtomicBool::new(false)))
        .is_err());
    std::fs::write(library_path(dir.path()), "not sqlite").unwrap();
    assert!(cache
        .search(dir.path(), &json!({}), Arc::new(AtomicBool::new(false)))
        .is_err());
}
#[test]
fn oversized_or_invalid_parameters_fail_before_opening_library() {
    let dir = tempdir().unwrap();
    let mut cache = SearchCache::default();
    for args in [
        json!({"query":"a".repeat(1201)}),
        json!({"query":"a ".repeat(17)}),
        json!({"category_id":3}),
        json!({"model":"m".repeat(201)}),
        json!({"offset":100001}),
    ] {
        let err = cache
            .search(dir.path(), &args, Arc::new(AtomicBool::new(false)))
            .unwrap_err();
        assert!(!err.contains("不存在"));
    }
}
#[test]
fn cancelled_search_does_not_poison_next_request() {
    let dir = tempdir().unwrap();
    let _db = seed(dir.path());
    let mut cache = SearchCache::default();
    assert!(cache
        .search(dir.path(), &json!({}), Arc::new(AtomicBool::new(true)))
        .unwrap_err()
        .contains("取消"));
    assert!(!ids(&search(&mut cache, dir.path(), json!({}))).is_empty());
}
