use promptark_mcp::handle_rpc;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

fn main() {
    let dir = std::env::var("PROMPTARK_LIBRARY_DIR").unwrap_or_else(|_| ".".into());
    let dir = PathBuf::from(dir);
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(request) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if let Some(response) = handle_rpc(&dir, &request) {
            let _ = writeln!(stdout, "{response}");
            let _ = stdout.flush();
        }
    }
}
