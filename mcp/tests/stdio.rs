use std::io::Write;
use std::process::{Command, Stdio};
use serde_json::{json, Value};
use tempfile::tempdir;

fn run(dir: &std::path::Path, requests: &[String]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_promptark-mcp"))
        .env("PROMPTARK_LIBRARY_DIR", dir).env("PROMPTARK_MCP_SQUARE", "0")
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for request in requests { writeln!(stdin, "{request}").unwrap(); }
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let ids: Vec<Value> = requests.iter().filter_map(|q| match serde_json::from_str::<Value>(q) {
        Ok(q) => q.get("id").cloned(), Err(_) => Some(Value::Null),
    }).collect();
    let mut responses: Vec<Value> = String::from_utf8(output.stdout).unwrap().lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    responses.sort_by_key(|r| ids.iter().position(|id| *id == r["id"]).expect("unexpected response id"));
    responses
}

fn call(id: u32, name: &str, arguments: Value) -> String {
    json!({"jsonrpc":"2.0", "id":id, "method":"tools/call", "params":{"name":name,"arguments":arguments}}).to_string()
}

#[test]
fn real_stdio_process_initializes_searches_reads_and_renders_without_writes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("promptark.sqlite");
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE prompts(id TEXT PRIMARY KEY,title TEXT,summary TEXT,content TEXT,deleted_at TEXT);
        INSERT INTO prompts VALUES('p1','自然光人像','用于拍摄','给 {{受众}} 的说明',NULL);
        INSERT INTO prompts VALUES('deleted','自然光旧版',NULL,'不可读取','1');").unwrap();
    let before = std::fs::read(&path).unwrap();
    let output = run(dir.path(), &[
        json!({"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test-agent","version":"1"}}}).to_string(),
        json!({"jsonrpc":"2.0", "method":"notifications/initialized"}).to_string(),
        json!({"jsonrpc":"2.0", "id":2, "method":"tools/list"}).to_string(),
        call(3,"search_prompts",json!({"query":"自然光"})),
        call(4,"get_prompt",json!({"id":"p1"})),
        call(5,"render_prompt",json!({"id":"p1","values":{"受众":"摄影师"}})),
        call(6,"get_prompt",json!({"id":"deleted"})),
        "not json".into(),
        json!({"jsonrpc":"2.0","id":7,"method":"ping"}).to_string(),
        call(8,"render_prompt",json!({"id":"p1"})),
    ]);
    assert_eq!(output.len(), 9);
    assert_eq!(output[0]["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(output[1]["result"]["tools"].as_array().unwrap().len(), 3);
    let hits: Value = serde_json::from_str(output[2]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(hits.as_array().unwrap().len(), 1);
    assert_eq!(hits[0]["id"], "p1");
    assert!(output[3]["result"]["content"][0]["text"].as_str().unwrap().contains("给 {{受众}} 的说明"));
    assert_eq!(output[4]["result"]["content"][0]["text"], "给 摄影师 的说明");
    assert_eq!(output[5]["result"]["isError"], true);
    assert_eq!(output[6]["error"]["code"], -32700);
    assert_eq!(output[7]["result"], json!({}));
    assert_eq!(output[8]["result"]["content"][0]["text"], "给 {{受众}} 的说明");
    assert_eq!(std::fs::read(path).unwrap(), before);
    assert_eq!(conn.query_row("SELECT count(*) FROM prompts", [], |row| row.get::<_, i64>(0)).unwrap(), 2);
}

#[test]
fn missing_library_returns_tool_error_without_creating_database() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &[call(1,"search_prompts", json!({"query":"test"}))]);
    assert_eq!(output[0]["result"]["isError"], true);
    assert!(!dir.path().join("promptark.sqlite").exists());
}

#[test]
fn missing_configuration_exits_with_stderr_only() {
    let output = Command::new(env!("CARGO_BIN_EXE_promptark-mcp")).env_remove("PROMPTARK_LIBRARY_DIR").output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("PROMPTARK_LIBRARY_DIR"));
}

#[test]
fn oversized_line_is_discarded_and_next_message_is_read() {
    let dir=tempdir().unwrap();
    let output=run(dir.path(),&["x".repeat(2*1024*1024),json!({"jsonrpc":"2.0","id":9,"method":"ping"}).to_string()]);
    assert_eq!(output.len(),2); assert_eq!(output[0]["error"]["code"],-32700); assert_eq!(output[1]["id"],9);
}
