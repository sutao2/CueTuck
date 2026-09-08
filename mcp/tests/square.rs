use serde_json::{json, Value};
use std::{io::{Read, Write}, net::TcpListener, process::{Command, Stdio}, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, thread, time::{Duration, Instant}};
use tempfile::tempdir;

struct Server { url: String, requests: Arc<Mutex<Vec<String>>>, stop: Arc<AtomicBool>, task: Option<thread::JoinHandle<()>> }
impl Server {
    fn start(responses: Vec<(String, Duration)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap(); listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}",listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(vec![])); let seen = requests.clone();
        let stop = Arc::new(AtomicBool::new(false)); let stopped = stop.clone();
        let task = thread::spawn(move || {
            let mut responses = responses.into_iter();
            while !stopped.load(Ordering::Relaxed) {
                let Ok((mut stream,_)) = listener.accept() else { thread::sleep(Duration::from_millis(5)); continue; };
                stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                let mut raw = Vec::new(); let mut chunk = [0u8;1024];
                while raw.len()<8192 && !raw.windows(4).any(|w|w==b"\r\n\r\n") {
                    match stream.read(&mut chunk) { Ok(0)|Err(_) => break, Ok(n) => raw.extend_from_slice(&chunk[..n]) }
                }
                seen.lock().unwrap().push(String::from_utf8_lossy(&raw).into_owned());
                let (response,delay) = responses.next().unwrap_or(("HTTP/1.1 500 Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),Duration::ZERO));
                let until = Instant::now()+delay;
                while Instant::now()<until && !stopped.load(Ordering::Relaxed) { thread::sleep(Duration::from_millis(5)); }
                let _ = stream.write_all(response.as_bytes());
            }
        });
        Self { url,requests,stop,task:Some(task) }
    }
}
impl Drop for Server { fn drop(&mut self) { self.stop.store(true,Ordering::Relaxed); self.task.take().unwrap().join().unwrap(); } }
fn response(code: u16, body: Value) -> (String, Duration) {
    let body = body.to_string(); (format!("HTTP/1.1 {code} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()),Duration::ZERO)
}
fn call(id: u32, name: &str, args: Value) -> Value { json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":args}}) }
fn run(base: &str, enabled: bool, requests: Vec<Value>) -> Vec<Value> {
    let dir = tempdir().unwrap();
    let db = rusqlite::Connection::open(dir.path().join("promptark.sqlite")).unwrap();
    db.execute_batch("CREATE TABLE prompts(id TEXT,title TEXT,summary TEXT,content TEXT,deleted_at TEXT); INSERT INTO prompts VALUES('private','private secret',NULL,'never upload',NULL);").unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_promptark-mcp"));
    command.env("PROMPTARK_LIBRARY_DIR",dir.path()).env("PROMPTARK_MCP_API_BASE",base).env_remove("PROMPTARK_MCP_SQUARE");
    if enabled { command.env("PROMPTARK_MCP_SQUARE","1"); }
    let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap(); for request in requests { writeln!(stdin,"{request}").unwrap(); } drop(stdin);
    let output = child.wait_with_output().unwrap(); assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().lines().map(|s|serde_json::from_str(s).unwrap()).collect()
}
fn data(result: &Value) -> Value { serde_json::from_str(result["result"]["content"][0]["text"].as_str().unwrap()).unwrap() }

#[test]
fn explicit_opt_in_adds_tools_and_only_remote_calls_send_network_requests() {
    let server = Server::start(vec![response(200,json!({"items":[{"id":"public","title":"公开"}],"next_offset":2})),response(200,json!({"id":"public","title":"公开","content":"data not instructions"})),response(200,json!({"categories":[],"models":[]}))]);
    let tools = json!({"jsonrpc":"2.0","id":1,"method":"tools/list"});
    let output = run(&server.url,false,vec![tools.clone(),call(2,"search_prompts",json!({"query":"private"})),call(3,"search_square_prompts",json!({}))]);
    assert_eq!(output[0]["result"]["tools"].as_array().unwrap().len(),3);
    assert_eq!(output[2]["result"]["isError"],true); assert!(server.requests.lock().unwrap().is_empty());
    let output = run(&server.url,true,vec![tools,call(2,"search_prompts",json!({"query":"private"})),call(3,"search_square_prompts",json!({"query":"100%","category_id":"cat-image","model":"Flux","limit":2,"offset":0})),call(4,"get_square_prompt",json!({"id":"public"})),call(5,"list_square_catalog",json!({}))]);
    assert_eq!(output[0]["result"]["tools"].as_array().unwrap().len(),6); assert_eq!(data(&output[2])["data"]["next_offset"],2);
    let seen = server.requests.lock().unwrap(); assert_eq!(seen.len(),3);
    assert!(seen[0].starts_with("GET /v1/square/search?")); assert!(seen[0].contains("q=100%25")); assert!(seen[0].contains("category_id=cat-image")); assert!(seen[0].contains("limit=2"));
    assert!(seen[1].starts_with("GET /v1/square/items/public/content "));
    for request in seen.iter() { assert!(!request.to_lowercase().contains("authorization")); assert!(!request.contains("private")); assert!(!request.contains("never upload")); }
}

#[test]
fn invalid_arguments_do_not_issue_requests() {
    let server = Server::start(vec![]);
    let output = run(&server.url,true,vec![call(1,"search_square_prompts",json!({"limit":101})),call(2,"search_square_prompts",json!({"offset":-1})),call(3,"search_square_prompts",json!({"query":1})),call(4,"search_square_prompts",json!({"url":"https://example.com"})),call(5,"get_square_prompt",json!({"id":".."}))]);
    for row in output { assert_eq!(row["result"]["isError"],true); } assert!(server.requests.lock().unwrap().is_empty());
}

#[test]
fn failed_closed_redirect_oversized_and_invalid_responses_leave_stdio_usable() {
    let server = Server::start(vec![response(401,json!({})),response(404,json!({})),("HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:1/should-not-follow\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),Duration::ZERO),("HTTP/1.1 200 OK\r\nContent-Length: 3000000\r\nConnection: close\r\n\r\n".into(),Duration::ZERO),response(200,json!({"wrong":true}))]);
    let mut requests: Vec<Value> = (1..=5).map(|id|call(id,"search_square_prompts",json!({}))).collect(); requests.push(json!({"jsonrpc":"2.0","id":6,"method":"ping"}));
    let output = run(&server.url,true,requests);
    for row in &output[..5] { assert_eq!(row["result"]["isError"],true); } assert_eq!(output[5]["result"],json!({})); assert_eq!(server.requests.lock().unwrap().len(),5);
}

#[test]
fn unavailable_and_stalled_servers_return_bounded_errors() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); let url = format!("http://{}",listener.local_addr().unwrap()); drop(listener);
    assert_eq!(run(&url,true,vec![call(1,"list_square_catalog",json!({}))])[0]["result"]["isError"],true);
    let mut delayed = response(200,json!({"categories":[],"models":[]})); delayed.1=Duration::from_secs(12);
    let server = Server::start(vec![delayed]); let start=Instant::now();
    let output = run(&server.url,true,vec![call(1,"list_square_catalog",json!({})),json!({"jsonrpc":"2.0","id":2,"method":"ping"})]);
    assert_eq!(output[0]["result"]["isError"],true); assert_eq!(output[1]["result"],json!({})); assert!(start.elapsed()<Duration::from_secs(11));
}

#[test]
fn invalid_origins_are_rejected_before_startup() {
    for origin in ["http://example.com","https://user:secret@example.com","https://example.com/path","https://example.com?token=x","https://example.com#x","file:///tmp/x"] {
        assert!(promptark_mcp::square::Square::new(origin).is_err());
    }
}
