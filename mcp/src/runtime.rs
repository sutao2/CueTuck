//! Keep blocking public HTTP calls off the local-library and protocol-control lanes.
use crate::{handle_rpc_with_search, search::SearchCache, square::Square};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
        Arc, Mutex,
    },
    thread,
};

const QUEUE_LIMIT: usize = 32;
const MAX_LINE: usize = 1024 * 1024;
struct Job {
    request: Value,
    key: String,
    cancel: Arc<AtomicBool>,
}
struct State<W> {
    output: Mutex<W>,
    active: Mutex<HashMap<String, Arc<AtomicBool>>>,
}
impl<W: Write> State<W> {
    fn write(&self, value: &Value) -> io::Result<()> {
        let mut output = self.output.lock().unwrap();
        writeln!(output, "{value}")?;
        output.flush()
    }
    fn finish(&self, job: &Job, response: Option<Value>) {
        self.active.lock().unwrap().remove(&job.key);
        if !job.cancel.load(Ordering::Relaxed) {
            if let Some(response) = response {
                let _ = self.write(&response);
            }
        }
    }
}
fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}
fn worker<W: Write + Send + 'static>(
    dir: PathBuf,
    square: Option<Arc<Square>>,
    state: Arc<State<W>>,
) -> (SyncSender<Job>, thread::JoinHandle<()>) {
    let (send, recv) = mpsc::sync_channel::<Job>(QUEUE_LIMIT);
    let thread = thread::spawn(move || {
        let mut search = SearchCache::default();
        for job in recv {
            let response = if job.cancel.load(Ordering::Relaxed) {
                None
            } else {
                handle_rpc_with_search(
                    &dir,
                    &job.request,
                    square.as_deref(),
                    &mut search,
                    job.cancel.clone(),
                )
            };
            state.finish(&job, response);
        }
    });
    (send, thread)
}

pub fn serve(
    dir: PathBuf,
    square: Option<Square>,
    mut input: impl BufRead,
    output: impl Write + Send + 'static,
) -> io::Result<()> {
    let square = square.map(Arc::new);
    let state = Arc::new(State {
        output: Mutex::new(output),
        active: Mutex::new(HashMap::new()),
    });
    let (local, local_thread) = worker(dir.clone(), square.clone(), state.clone());
    let (remote, remote_thread) = worker(dir.clone(), square.clone(), state.clone());
    let mut control_search = SearchCache::default();
    let result = (|| -> io::Result<()> {
        loop {
            let mut line = Vec::new();
            let size = std::io::Read::take(&mut input, (MAX_LINE + 1) as u64)
                .read_until(b'\n', &mut line)?;
            if size == 0 {
                break;
            }
            if size > MAX_LINE {
                // Discard the rest without allocating an unbounded second buffer.
                if line.last() != Some(&b'\n') {
                    loop {
                        let buf = input.fill_buf()?;
                        if buf.is_empty() {
                            break;
                        }
                        let end = buf.iter().position(|b| *b == b'\n');
                        let count = end.map_or(buf.len(), |i| i + 1);
                        input.consume(count);
                        if end.is_some() {
                            break;
                        }
                    }
                }
                state.write(&error(Value::Null, -32700, "消息超过 1 MiB 限制"))?;
                continue;
            }
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let request: Value = match serde_json::from_slice(&line) {
                Ok(value) => value,
                Err(_) => {
                    state.write(&error(Value::Null, -32700, "JSON 解析失败"))?;
                    continue;
                }
            };
            if request["jsonrpc"] == "2.0"
                && request["method"] == "notifications/cancelled"
                && request.get("id").is_none()
            {
                if let Some(id) = request["params"]
                    .get("requestId")
                    .filter(|id| id.is_string() || id.is_number())
                {
                    if let Some(cancel) = state.active.lock().unwrap().get(&id.to_string()) {
                        cancel.store(true, Ordering::Relaxed);
                    }
                }
                continue;
            }
            if request["jsonrpc"] == "2.0"
                && request["method"] == "tools/call"
                && request.get("id").is_some()
            {
                let id = request["id"].clone();
                if !id.is_string() && !id.is_number() {
                    state.write(&error(Value::Null, -32600, "请求 id 必须为字符串或数字"))?;
                    continue;
                }
                let key = id.to_string();
                let cancel = Arc::new(AtomicBool::new(false));
                {
                    let mut active = state.active.lock().unwrap();
                    if active.contains_key(&key) {
                        drop(active);
                        state.write(&error(id, -32600, "请求 id 正在使用"))?;
                        continue;
                    }
                    active.insert(key.clone(), cancel.clone());
                }
                let send = match request["params"]["name"].as_str() {
                    Some("search_square_prompts" | "get_square_prompt" | "list_square_catalog") => {
                        &remote
                    }
                    _ => &local,
                };
                if send
                    .try_send(Job {
                        request,
                        key: key.clone(),
                        cancel,
                    })
                    .is_err()
                {
                    state.active.lock().unwrap().remove(&key);
                    state.write(&error(id, -32000, "工具队列已满，请稍后重试"))?;
                }
            } else if let Some(response) = handle_rpc_with_search(
                &dir,
                &request,
                square.as_deref(),
                &mut control_search,
                Arc::new(AtomicBool::new(false)),
            ) {
                state.write(&response)?;
            }
        }
        Ok(())
    })();
    drop(local);
    drop(remote);
    let local_result = local_thread.join();
    let remote_result = remote_thread.join();
    if local_result.is_err() || remote_result.is_err() {
        return Err(io::Error::other("MCP 工具线程异常退出"));
    }
    result
}
