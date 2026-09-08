use super::*;
use axum::response::IntoResponse;
use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{Path, State},
    http::Request,
    routing::put,
};
use serde_json::{json, Value};
use sha2::Digest;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tower::ServiceExt;

#[derive(Clone, Default)]
struct Store {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    gets: Arc<AtomicUsize>,
    mode: Arc<AtomicUsize>,
}
struct Server(tokio::task::JoinHandle<()>);
impl Drop for Server {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn fixture() -> (AppState, String, String, String, Store, Server) {
    let store = Store::default();
    let router = Router::new()
        .route(
            "/*key",
            put(
                |State(s): State<Store>, Path(key): Path<String>, body: Bytes| async move {
                    if s.mode.load(Ordering::SeqCst) == 3 {
                        return StatusCode::BAD_GATEWAY;
                    }
                    s.objects.lock().unwrap().insert(key, body.to_vec());
                    StatusCode::OK
                },
            )
            .get(
                |State(s): State<Store>, Path(key): Path<String>| async move {
                    s.gets.fetch_add(1, Ordering::SeqCst);
                    if s.mode.load(Ordering::SeqCst) == 4 {
                        return axum::response::Redirect::temporary("http://127.0.0.1:1/blocked")
                            .into_response();
                    }
                    match s.objects.lock().unwrap().get(&key).cloned() {
                        Some(mut bytes) => {
                            if s.mode.load(Ordering::SeqCst) == 1 {
                                bytes[0] ^= 1;
                            }
                            if s.mode.load(Ordering::SeqCst) == 2 {
                                bytes.push(0);
                            }
                            (StatusCode::OK, bytes).into_response()
                        }
                        None => StatusCode::NOT_FOUND.into_response(),
                    }
                },
            )
            .delete(
                |State(s): State<Store>, Path(key): Path<String>| async move {
                    s.objects.lock().unwrap().remove(&key);
                    StatusCode::NO_CONTENT
                },
            ),
        )
        .layer(axum::extract::DefaultBodyLimit::max(6 * 1024 * 1024))
        .with_state(store.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let server = Server(tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    }));
    let mut state = crate::admin_security_tests::state().await;
    state.media = Some(media::MediaConfig {
        endpoint,
        access_key: "test".into(),
        secret_key: "test".into(),
        bucket: "private-test".into(),
    });
    let pg = state.db.as_ref().unwrap();
    for (email, role) in [
        ("a@example.com", "user"),
        ("b@example.com", "user"),
        ("owner@example.com", "owner"),
    ] {
        pg.upsert_account(email, None, role).await.unwrap();
    }
    let a = state
        .issue_session("a@example.com".into())
        .await
        .unwrap()
        .access_token;
    let b = state
        .issue_session("b@example.com".into())
        .await
        .unwrap()
        .access_token;
    let admin = state
        .issue_session("owner@example.com".into())
        .await
        .unwrap()
        .access_token;
    (state, a, b, admin, store, server)
}

async fn upload(
    state: &AppState,
    token: &str,
    name: &str,
    mime: &str,
    bytes: &[u8],
    repeat: bool,
) -> (StatusCode, Value) {
    let mut body = Vec::new();
    for _ in 0..if repeat { 2 } else { 1 } {
        body.extend_from_slice(format!("--boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: {mime}\r\n\r\n").as_bytes());
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(b"--boundary--\r\n");
    let response = app(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/media/upload")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "multipart/form-data; boundary=boundary")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn get(state: &AppState, token: &str, path: &str) -> axum::response::Response {
    app(state.clone())
        .oneshot(
            Request::builder()
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn private_media_roundtrip_survives_restart_and_rechecks_ownership_and_sessions() {
    let (state, a, b, admin, store, _server) = fixture().await;
    let bytes = "私有资料\n测试".as_bytes();
    let (status, value) = upload(&state, &a, "参考.md", "text/plain", bytes, false).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    let path = value["url"].as_str().unwrap();
    assert!(path.starts_with("/v1/media/"));
    assert_eq!(value["size"], bytes.len());
    assert_eq!(
        value["sha256"],
        format!("{:x}", sha2::Sha256::digest(bytes))
    );
    let url_path = format!("/v1/media/{}/url", value["id"].as_str().unwrap());
    for target in [path, &url_path] {
        assert_eq!(
            get(&state, "", target).await.status(),
            StatusCode::UNAUTHORIZED
        );
        for token in [&b, &admin] {
            assert_eq!(
                get(&state, token, target).await.status(),
                StatusCode::NOT_FOUND
            );
        }
    }
    assert_eq!(store.gets.load(Ordering::SeqCst), 0);
    let restarted = AppState {
        db: state.db.clone(),
        media: state.media.clone(),
        ..AppState::default()
    };
    let response = get(&restarted, &a, path).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    assert!(response.headers()["content-disposition"]
        .to_str()
        .unwrap()
        .starts_with("attachment;"));
    assert_eq!(
        to_bytes(response.into_body(), media::MAX_FILE)
            .await
            .unwrap()
            .as_ref(),
        bytes
    );
    let response = get(&state, &a, &url_path).await;
    let returned: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(returned, json!({"url":path}));
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!(
        "UPDATE {} SET disabled=TRUE WHERE email='a@example.com'",
        pg.t("accounts")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        get(&state, &a, path).await.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(store.gets.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn private_upload_rejects_invalid_or_oversized_files_without_storage_writes() {
    let (state, a, _, _, store, _server) = fixture().await;
    assert_eq!(
        upload(&state, "", "a.txt", "text/plain", b"a", false)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let unavailable = AppState {
        media: state.media.clone(),
        ..AppState::default()
    };
    let session = unavailable
        .issue_session("a@example.com".into())
        .await
        .unwrap();
    assert_eq!(
        upload(
            &unavailable,
            &session.access_token,
            "a.txt",
            "text/plain",
            b"a",
            false
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    for (name, mime, data, repeat, status) in [
        (
            "bad.svg",
            "image/svg+xml",
            &b"<svg/>"[..],
            false,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "fake.png",
            "image/png",
            &b"<html>"[..],
            false,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "../a.txt",
            "text/plain",
            &b"a"[..],
            false,
            StatusCode::BAD_REQUEST,
        ),
        (
            "a.txt",
            "text/plain",
            &b"a"[..],
            true,
            StatusCode::BAD_REQUEST,
        ),
    ] {
        assert_eq!(upload(&state, &a, name, mime, data, repeat).await.0, status);
    }
    assert_eq!(
        upload(
            &state,
            &a,
            "big.txt",
            "text/plain",
            &vec![b'a'; media::MAX_FILE + 1],
            false
        )
        .await
        .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert_eq!(
        upload(
            &state,
            &a,
            "huge.txt",
            "text/plain",
            &vec![b'a'; 7 * 1024 * 1024],
            false
        )
        .await
        .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert!(store.objects.lock().unwrap().is_empty());
    let pg = state.db.as_ref().unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {}", pg.t("media_objects")))
            .fetch_one(&pg.pool)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn storage_failure_cleans_reservation_and_download_rejects_corruption_or_redirects() {
    let (state, a, _, _, store, _server) = fixture().await;
    store.mode.store(3, Ordering::SeqCst);
    assert_eq!(
        upload(&state, &a, "a.txt", "text/plain", b"test", false)
            .await
            .0,
        StatusCode::BAD_GATEWAY
    );
    let pg = state.db.as_ref().unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {}", pg.t("media_objects")))
            .fetch_one(&pg.pool)
            .await
            .unwrap(),
        0
    );
    // If registering success fails, never acknowledge an upload and clean only its own object.
    sqlx::query(&format!(
        "ALTER TABLE {} ADD CONSTRAINT fail_ready CHECK (NOT ready)",
        pg.t("media_objects")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    store.mode.store(0, Ordering::SeqCst);
    assert_eq!(
        upload(&state, &a, "failed.txt", "text/plain", b"test", false)
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert!(store.objects.lock().unwrap().is_empty());
    assert_eq!(
        sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {}", pg.t("media_objects")))
            .fetch_one(&pg.pool)
            .await
            .unwrap(),
        0
    );
    sqlx::query(&format!(
        "ALTER TABLE {} DROP CONSTRAINT fail_ready",
        pg.t("media_objects")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    store.mode.store(0, Ordering::SeqCst);
    let (status, file) = upload(&state, &a, "a.txt", "text/plain", b"test", false).await;
    assert_eq!(status, StatusCode::OK);
    for mode in [1, 2, 4] {
        store.mode.store(mode, Ordering::SeqCst);
        assert_eq!(
            get(&state, &a, file["url"].as_str().unwrap())
                .await
                .status(),
            StatusCode::BAD_GATEWAY
        );
    }
}

#[tokio::test]
#[ignore = "requires local private MinIO bucket; creates and removes only one generated test object"]
async fn real_minio_private_attachment_roundtrip() {
    use rusty_s3::S3Action;
    use std::{borrow::Cow, time::Duration};
    let (mut state, a, _, _, _, _server) = fixture().await;
    let config = media::MediaConfig::from_env().unwrap();
    assert!(config.ping().await, "local MinIO bucket required");
    let bucket = rusty_s3::Bucket::new(
        url::Url::parse(&config.endpoint).unwrap(),
        rusty_s3::UrlStyle::Path,
        Cow::Owned(config.bucket.clone()),
        Cow::Borrowed("us-east-1"),
    )
    .unwrap();
    let credentials =
        rusty_s3::Credentials::new(config.access_key.clone(), config.secret_key.clone());
    state.media = Some(config);
    let bytes = "PromptArk 附件完整性测试\n只用于隔离验收".as_bytes();
    let (status, result) = upload(&state, &a, "往返验证.txt", "text/plain", bytes, false).await;
    assert_eq!(status, StatusCode::OK);
    let key = format!("promptark/{}", result["id"].as_str().unwrap());
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();
    let anonymous = client
        .get(bucket.object_url(&key).unwrap())
        .send()
        .await
        .unwrap()
        .status();
    let response = get(&state, &a, result["url"].as_str().unwrap()).await;
    let download_status = response.status();
    let downloaded = to_bytes(response.into_body(), media::MAX_FILE)
        .await
        .unwrap();
    let cleanup = bucket.delete_object(Some(&credentials), &key);
    assert!(client
        .delete(cleanup.sign(Duration::from_secs(60)))
        .send()
        .await
        .unwrap()
        .status()
        .is_success());
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!(
        "DELETE FROM {} WHERE id=$1",
        pg.t("media_objects")
    ))
    .bind(result["id"].as_str().unwrap())
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        anonymous,
        StatusCode::FORBIDDEN,
        "bucket must not permit anonymous object access"
    );
    assert_eq!(download_status, StatusCode::OK);
    assert_eq!(downloaded.as_ref(), bytes);
}

#[tokio::test]
async fn concurrent_upload_reservations_enforce_account_quota_and_pending_is_unreadable() {
    let (state, a, _, _, _, _server) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!("INSERT INTO {} (id,owner_email,object_key,size) SELECT 'seed.'||n,'a@example.com','seed',0 FROM generate_series(1,255) n", pg.t("media_objects"))).execute(&pg.pool).await.unwrap();
    let file = |id: &str| media::MediaUpload {
        id: id.into(),
        url: String::new(),
        name: "a.txt".into(),
        mime: "text/plain".into(),
        size: 1,
        sha256: "a".repeat(64),
    };
    let x = file("x");
    let y = file("y");
    let (first, second) = tokio::join!(
        media::reserve(pg, "a@example.com", "x", &x),
        media::reserve(pg, "a@example.com", "y", &y)
    );
    assert_ne!(first.is_ok(), second.is_ok());
    let id = if first.is_ok() { "x" } else { "y" };
    assert_eq!(
        get(&state, &a, &format!("/v1/media/{id}/content"))
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&state, &a, "/v1/media/seed.1/url").await.status(),
        StatusCode::CONFLICT
    );
    // A separate account is governed by its own byte quota, including pending uploads.
    let mut large = file("large");
    large.size = 100 * 1024 * 1024;
    media::reserve(pg, "b@example.com", "large", &large)
        .await
        .unwrap();
    assert_eq!(
        media::reserve(pg, "b@example.com", "extra", &file("extra")).await,
        Err(StatusCode::CONFLICT)
    );
}
