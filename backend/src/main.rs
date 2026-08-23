use promptark_api::{app, AppState};

#[tokio::main]
async fn main() {
    let email = std::env::var("PROMPTARK_DEV_EMAIL").unwrap_or_else(|_| "dev@promptark.local".into());
    let password = std::env::var("PROMPTARK_DEV_PASSWORD").unwrap_or_else(|_| "devpass".into());
    let addr = std::env::var("PROMPTARK_API_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let state = AppState::with_user(&email, &password);
    state.seed_square_demo();
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind API");
    axum::serve(listener, app(state))
        .await
        .expect("API");
}
