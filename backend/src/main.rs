use promptark_api::{app, AppState};

#[tokio::main]
async fn main() {
    let addr = std::env::var("PROMPTARK_API_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let state = AppState::from_runtime()
        .await
        .unwrap_or_else(|err| panic!("backend runtime: {err}"));
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind API");
    axum::serve(listener, app(state)).await.expect("API");
}
