use regex::Regex;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const TOOLS: &[&str] = &["search_prompts", "get_prompt", "render_prompt"];

pub fn library_path(dir: &Path) -> PathBuf {
    dir.join("promptark.sqlite")
}

pub fn search_prompts(dir: &Path, query: &str) -> Result<Vec<Value>, String> {
    let path = library_path(dir);
    if !path.exists() {
        return Err("本机库文件不存在".into());
    }
    let connection = Connection::open(&path).map_err(|error| error.to_string())?;
    let pattern = format!("%{}%", query.trim());
    let mut statement = connection
        .prepare(
            "SELECT id, title, summary FROM prompts
             WHERE deleted_at IS NULL
               AND (?1 = '' OR title LIKE ?2 OR content LIKE ?2)
             ORDER BY title",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(rusqlite::params![query.trim(), pattern], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "summary": row.get::<_, Option<String>>(2)?,
            }))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}

pub fn get_prompt(dir: &Path, id: &str) -> Result<Value, String> {
    let path = library_path(dir);
    if !path.exists() {
        return Err("本机库文件不存在".into());
    }
    let connection = Connection::open(&path).map_err(|error| error.to_string())?;
    connection
        .query_row(
            "SELECT id, title, content FROM prompts WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "content": row.get::<_, String>(2)?,
                }))
            },
        )
        .map_err(|_| "提示词不存在".into())
}

pub fn render_prompt_text(content: &str, values: &HashMap<String, String>) -> String {
    let pattern = Regex::new(r"\{\{\s*([^}]*?)\s*\}\}").expect("variable pattern");
    pattern
        .replace_all(content, |caps: &regex::Captures| {
            let name = caps[1].trim();
            if name.is_empty() {
                return caps[0].to_string();
            }
            match values.get(name) {
                Some(value) if !value.is_empty() => value.clone(),
                _ => format!("{{{{{name}}}}}"),
            }
        })
        .into_owned()
}

pub fn handle_rpc(dir: &Path, request: &Value) -> Option<Value> {
    let method = request.get("method")?.as_str()?;
    if method.starts_with("notifications/") {
        return None;
    }
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "promptark-mcp", "version": "0.1.0" }
        }),
        "tools/list" => json!({ "tools": tool_defs() }),
        "tools/call" => match call_tool(dir, request.get("params").unwrap_or(&Value::Null)) {
            Ok(value) => value,
            Err(message) => json!({
                "content": [{ "type": "text", "text": message }],
                "isError": true
            }),
        },
        other => {
            return Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("未知方法 {other}") }
            }));
        }
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn tool_defs() -> Vec<Value> {
    vec![
        json!({
            "name": "search_prompts",
            "description": "搜索本机提示词标题或正文",
            "inputSchema": {
                "type": "object",
                "properties": { "query": { "type": "string" } }
            }
        }),
        json!({
            "name": "get_prompt",
            "description": "按 id 读取本机提示词",
            "inputSchema": {
                "type": "object",
                "properties": { "id": { "type": "string" } },
                "required": ["id"]
            }
        }),
        json!({
            "name": "render_prompt",
            "description": "用变量值渲染提示词正文；未填保留 {{名称}}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "values": { "type": "object", "additionalProperties": { "type": "string" } }
                },
                "required": ["id"]
            }
        }),
    ]
}

fn call_tool(dir: &Path, params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "缺少工具名".to_string())?;
    let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
    match name {
        "search_prompts" => {
            let query = arguments.get("query").and_then(Value::as_str).unwrap_or("");
            let items = search_prompts(dir, query)?;
            Ok(json!({ "content": [{ "type": "text", "text": Value::Array(items).to_string() }] }))
        }
        "get_prompt" => {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| "缺少 id".to_string())?;
            let item = get_prompt(dir, id)?;
            Ok(json!({ "content": [{ "type": "text", "text": item.to_string() }] }))
        }
        "render_prompt" => {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| "缺少 id".to_string())?;
            let item = get_prompt(dir, id)?;
            let content = item
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or("");
            let mut values = HashMap::new();
            if let Some(map) = arguments.get("values").and_then(Value::as_object) {
                for (key, value) in map {
                    if let Some(text) = value.as_str() {
                        values.insert(key.clone(), text.to_string());
                    }
                }
            }
            let rendered = render_prompt_text(content, &values);
            Ok(json!({ "content": [{ "type": "text", "text": rendered }] }))
        }
        other => Err(format!("未知工具 {other}")),
    }
}

pub fn listed_tool_names() -> Vec<&'static str> {
    TOOLS.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::tempdir;

    fn seed(dir: &Path) {
        let connection = Connection::open(library_path(dir)).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE prompts (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    summary TEXT,
                    content TEXT NOT NULL DEFAULT '',
                    deleted_at TEXT
                );",
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO prompts (id, title, summary, content, deleted_at)
                 VALUES ('p-1', '自然光群像', NULL, '给 {{受众}} 的说明', NULL)",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO prompts (id, title, summary, content, deleted_at)
                 VALUES ('p-gone', '已删', NULL, 'x', '1')",
                [],
            )
            .unwrap();
    }

    #[test]
    fn lists_required_tools() {
        let response = handle_rpc(Path::new("."), &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .unwrap();
        let names: Vec<_> = response["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["search_prompts", "get_prompt", "render_prompt"]);
    }

    #[test]
    fn search_hits_title() {
        let dir = tempdir().unwrap();
        seed(dir.path());
        let hits = search_prompts(dir.path(), "自然光").unwrap();
        assert_eq!(hits[0]["id"], "p-1");
        assert_eq!(hits[0]["title"], "自然光群像");
        assert!(hits.iter().all(|hit| hit["id"] != "p-gone"));
    }

    #[test]
    fn search_missing_library_errors() {
        let dir = tempdir().unwrap();
        let error = search_prompts(dir.path(), "自然光").unwrap_err();
        assert!(error.contains("不存在"));
    }

    #[test]
    fn render_keeps_unfilled_placeholder() {
        let rendered = render_prompt_text("给 {{受众}} 的说明", &HashMap::new());
        assert_eq!(rendered, "给 {{受众}} 的说明");
    }

    #[test]
    fn search_has_no_http_client() {
        let manifest = include_str!("../Cargo.toml");
        assert!(!manifest.contains("reqwest"));
        assert!(!manifest.contains("ureq"));
    }
}
