use promptark_mcp::{runtime, square::Square};
use std::io;
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
    if let Err(error) = runtime::serve(dir, square, io::stdin().lock(), io::stdout()) {
        eprintln!("MCP 服务已停止：{error}");
        std::process::exit(1);
    }
}
