use super::*;
use crate::admin_security_tests::{request, state};
use serde_json::json;
#[tokio::test]
async fn overview_is_real_and_does_not_invent_unknown_account_history() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("owner@ops.test", Some("test-password"), "owner")
        .await
        .unwrap();
    pg.upsert_account("legacy@ops.test", Some("test-password"), "user")
        .await
        .unwrap();
    sqlx::query(&format!(
        "UPDATE {} SET created_at=NULL WHERE email='legacy@ops.test'",
        pg.t("accounts")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    let owner = state
        .issue_session("owner@ops.test".into())
        .await
        .unwrap()
        .access_token;
    let user = state
        .issue_session("legacy@ops.test".into())
        .await
        .unwrap()
        .access_token;
    assert_eq!(
        request(&state, "GET", "/v1/admin/overview?days=7", &user, json!({}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let (status, overview) = request(
        &state,
        "GET",
        "/v1/admin/overview?days=7",
        &owner,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(overview["accounts"], 2);
    assert_eq!(overview["new_accounts"], 1);
    assert_eq!(overview["unknown_account_dates"], 1);
    assert_eq!(overview["mock_orders"], 0);
    assert_eq!(overview["days"], 7);
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/admin/overview?days=999",
            &owner,
            json!({})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
}
#[tokio::test]
async fn audit_export_and_request_failures_exclude_secrets_and_path_parameters() {
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account("owner@ops.test", Some("test-password"), "owner")
        .await
        .unwrap();
    pg.upsert_account("admin@ops.test", Some("test-password"), "admin")
        .await
        .unwrap();
    let owner = state
        .issue_session("owner@ops.test".into())
        .await
        .unwrap()
        .access_token;
    let admin = state
        .issue_session("admin@ops.test".into())
        .await
        .unwrap()
        .access_token;
    sqlx::query(&format!("INSERT INTO {} (id,actor_email,action,details) VALUES ('sensitive','owner@ops.test','test',$1)",pg.t("security_audit"))).bind(json!({"password":"private-password","token":"private-token","before":{"secret":"private-key","revision":1},"revision":2,"content":"private-body","retention_days":90,"counts":{"mail_outbox":2,"password":"private-password"}})).execute(&pg.pool).await.unwrap();
    request(
        &state,
        "POST",
        "/v1/admin/users/private%40target.test/password-reset",
        &admin,
        json!({"current_password":"never-log-this"}),
    )
    .await;
    let (status, events) = request(&state, "GET", "/v1/admin/audit", &owner, json!({})).await;
    assert_eq!(status, StatusCode::OK);
    let text = events.to_string();
    for secret in [
        "private-password",
        "private-token",
        "private-key",
        "private-body",
        "never-log-this",
        "private%40target.test",
    ] {
        assert!(!text.contains(secret));
    }
    assert!(text.contains("request_failed"));
    assert!(text.contains("revision"));
    let sensitive = events["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"] == "sensitive")
        .unwrap();
    assert_eq!(sensitive["details"]["counts"]["mail_outbox"], 2);
    assert_eq!(sensitive["details"]["retention_days"], 90);
    assert_eq!(
        request(&state, "GET", "/v1/admin/audit", &admin, json!({}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &state,
            "GET",
            "/v1/admin/audit?from=2026-02-30",
            &owner,
            json!({})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (_, export) = request(&state, "GET", "/v1/admin/audit/export", &owner, json!({})).await;
    assert_eq!(export["limit"], 500);
    assert!(export["items"].as_array().unwrap().len() <= 500);
    sqlx::query(&format!("INSERT INTO {} (id,actor_email,action,details) SELECT 'bulk-'||i,'owner@ops.test','bulk','{{}}'::jsonb FROM generate_series(1,510) i",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    let (_, export) = request(
        &state,
        "GET",
        "/v1/admin/audit/export?action=bulk",
        &owner,
        json!({}),
    )
    .await;
    assert_eq!(export["total"], 510);
    assert_eq!(export["items"].as_array().unwrap().len(), 500);
}

#[tokio::test]
async fn media_health_rejects_http_errors_and_redirects_and_system_hides_configuration() {
    for code in [200, 403, 500, 302] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buffer = [0; 4096];
            stream.read(&mut buffer).await.unwrap();
            stream.write_all(format!("HTTP/1.1 {code} Test\r\nContent-Length: 0\r\nLocation: http://127.0.0.1:1/private\r\nConnection: close\r\n\r\n").as_bytes()).await.unwrap();
        });
        let config = media::MediaConfig {
            endpoint: format!("http://{address}"),
            access_key: "synthetic-access".into(),
            secret_key: "synthetic-secret".into(),
            bucket: "test".into(),
        };
        assert_eq!(config.ping().await, code == 200);
        server.await.unwrap();
    }
    let state = state().await;
    state
        .db
        .as_ref()
        .unwrap()
        .upsert_account("owner@ops.test", Some("test-password"), "owner")
        .await
        .unwrap();
    let token = state
        .issue_session("owner@ops.test".into())
        .await
        .unwrap()
        .access_token;
    let (status, system) = request(&state, "GET", "/v1/admin/system", &token, json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(system["postgres"]["status"], "healthy");
    assert_eq!(system["redis"]["status"], "not_configured");
    assert_eq!(system["media"]["status"], "not_configured");
    assert!(!system.to_string().contains("postgres://"));
    assert!(system["mail_queue"].is_object());
    assert!(system["notification_queue"].is_object());
}

// Explicit opt-in: Docker must contain the local PostgreSQL 16 development instance.
// Only a freshly created schema is dumped; production/public schema is never selected.
#[tokio::test]
#[ignore = "requires Docker backend-db-1 and permission to create an isolated recovery database"]
async fn isolated_postgres_restore_preserves_credentials_content_and_encrypted_configuration() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let state = state().await;
    let pg = state.db.as_ref().unwrap();
    pg.upsert_account(
        "restore@synthetic.test",
        Some("synthetic-restore-password"),
        "owner",
    )
    .await
    .unwrap();
    sqlx::query(&format!("INSERT INTO {} (id,title,kind,content) VALUES ('recovery-sample','Synthetic recovery','prompt','Synthetic snapshot')",pg.t("square_items"))).execute(&pg.pool).await.unwrap();
    use ring::rand::{SecureRandom, SystemRandom};
    let mut key = [0u8; 32];
    SystemRandom::new().fill(&mut key).unwrap();
    let cipher = oauth_admin::seal(&key, "google", "synthetic-provider-secret").unwrap();
    pg.set_oauth_config("google", &cipher).await.unwrap();
    let target = format!("recovery_{}", Uuid::new_v4().simple());
    // Names are generated here, not supplied by environment variables or users.
    sqlx::query(&format!("CREATE DATABASE \"{target}\""))
        .execute(&pg.pool)
        .await
        .unwrap();
    let result = async {
        let dump = Command::new("docker")
            .args([
                "exec",
                "backend-db-1",
                "pg_dump",
                "-U",
                "pl",
                "-d",
                "promptark",
                "--format=custom",
                "--no-owner",
                "--no-privileges",
                "--schema",
                &pg.schema,
            ])
            .output()
            .map_err(|e| e.to_string())?;
        if !dump.status.success() {
            return Err("synthetic schema dump failed".to_owned());
        }
        let mut child = Command::new("docker")
            .args([
                "exec",
                "-i",
                "backend-db-1",
                "pg_restore",
                "-U",
                "pl",
                "--dbname",
                &target,
                "--no-owner",
                "--no-privileges",
                "--exit-on-error",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&dump.stdout)
            .map_err(|e| e.to_string())?;
        if !child
            .wait_with_output()
            .map_err(|e| e.to_string())?
            .status
            .success()
        {
            return Err("isolated restore failed".to_owned());
        }
        let pool = sqlx::PgPool::connect(&format!(
            "postgres://pl:pl@127.0.0.1:5432/{target}?sslmode=disable"
        ))
        .await
        .map_err(|e| e.to_string())?;
        let restored = postgres::Pg::new(pool.clone(), &pg.schema).map_err(|e| e.to_string())?;
        let checks = async {
            restored
                .verify_login("restore@synthetic.test", "synthetic-restore-password")
                .await
                .map_err(|e| e.to_string())?;
            if restored
                .verify_login("restore@synthetic.test", "incorrect-password")
                .await
                .is_ok()
            {
                return Err("wrong password accepted".into());
            }
            for table in [
                "accounts",
                "square_items",
                "settings",
                "catalog",
                "security_audit",
            ] {
                let source: i64 =
                    sqlx::query_scalar(&format!("SELECT count(*) FROM {}", pg.t(table)))
                        .fetch_one(&pg.pool)
                        .await
                        .map_err(|e| e.to_string())?;
                let dest: i64 =
                    sqlx::query_scalar(&format!("SELECT count(*) FROM {}", restored.t(table)))
                        .fetch_one(&pool)
                        .await
                        .map_err(|e| e.to_string())?;
                if source != dest {
                    return Err(format!("row count mismatch: {table}"));
                }
            }
            let snapshot: String = sqlx::query_scalar(&format!(
                "SELECT content FROM {} WHERE id='recovery-sample'",
                restored.t("square_items")
            ))
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?;
            if snapshot != "Synthetic snapshot" {
                return Err("snapshot mismatch".into());
            }
            let encrypted = restored
                .oauth_config("google")
                .await
                .map_err(|e| e.to_string())?
                .ok_or("missing ciphertext")?;
            if oauth_admin::unseal(&key, "google", &encrypted).map_err(|e| e.to_string())?
                != "synthetic-provider-secret"
            {
                return Err("decryption mismatch".into());
            }
            let mut wrong_key = key;
            wrong_key[0] ^= 1;
            if oauth_admin::unseal(&wrong_key, "google", &encrypted).is_ok() {
                return Err("incorrect encryption key accepted".into());
            }
            Ok::<(), String>(())
        }
        .await;
        pool.close().await;
        checks
    }
    .await;
    // Only objects created above are removed; no CASCADE on any existing database.
    let cleanup = sqlx::query(&format!("DROP DATABASE \"{target}\""))
        .execute(&pg.pool)
        .await;
    let schema_cleanup = sqlx::query(&format!("DROP SCHEMA \"{}\" CASCADE", pg.schema))
        .execute(&pg.pool)
        .await;
    assert!(cleanup.is_ok(), "isolated database cleanup failed");
    assert!(schema_cleanup.is_ok(), "synthetic schema cleanup failed");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}
