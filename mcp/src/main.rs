use promptark_mcp::{handle_rpc_with_square, square::Square};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

fn main() {
    let dir = match std::env::var("PROMPTARK_LIBRARY_DIR") {
        Ok(dir) if !dir.trim().is_empty() => dir,
        _ => { eprintln!("请设置 PROMPTARK_LIBRARY_DIR 为含 promptark.sqlite 的目录"); std::process::exit(2); }
    };
    let dir = PathBuf::from(dir);
    let square = match std::env::var("PROMPTARK_MCP_SQUARE").as_deref() {
        Ok("1") => match Square::new(&std::env::var("PROMPTARK_MCP_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:8787".into())) {
            Ok(client) => Some(client), Err(message) => { eprintln!("{message}"); std::process::exit(2); }
        },
        Ok("0")|Err(_) => None,
        _ => { eprintln!("PROMPTARK_MCP_SQUARE 只能为 0 或 1"); std::process::exit(2); }
    };
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle_rpc_with_square(&dir, &request, square.as_ref()),
            Err(_) => Some(serde_json::json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "JSON 解析失败"}})),
        };
        if let Some(response) = response {
            if writeln!(stdout, "{response}").and_then(|_| stdout.flush()).is_err() { break; }
        }
    }
}
