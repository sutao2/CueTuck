use crate::{postgres::Pg, AppState};
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, net::SocketAddr, sync::Mutex, time::Instant};

#[derive(Default)]
pub struct AuthLimits {
    memory: Mutex<HashMap<String, (Instant, u32)>>,
}
fn key(scope: &str, value: &str) -> String {
    format!("{scope}:{:x}", Sha256::digest(value.as_bytes()))
}

impl AuthLimits {
    fn take_memory(&self, key: &str, limit: u32, now: Instant) -> Result<(), StatusCode> {
        let mut buckets = self
            .memory
            .lock()
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        buckets.retain(|_, (start, _)| now.duration_since(*start).as_secs() < 60);
        let bucket = buckets.entry(key.into()).or_insert((now, 0));
        if bucket.1 >= limit {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        bucket.1 += 1;
        Ok(())
    }
}

impl Pg {
    pub async fn take_auth_attempt(&self, key: &str, limit: u32) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "DELETE FROM {} WHERE expires_at<=now()",
            self.t("auth_limits")
        ))
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        // UPSERT locks each bucket; concurrent processes share a single admitted count.
        let admitted: Option<i32> = sqlx::query_scalar(&format!("INSERT INTO {} AS b (bucket_key,attempts,expires_at) VALUES ($1,1,now()+interval '60 seconds') ON CONFLICT(bucket_key) DO UPDATE SET attempts=CASE WHEN b.expires_at<=now() THEN 1 ELSE b.attempts+1 END, expires_at=CASE WHEN b.expires_at<=now() THEN now()+interval '60 seconds' ELSE b.expires_at END WHERE b.attempts<$2 OR b.expires_at<=now() RETURNING attempts", self.t("auth_limits")))
            .bind(key).bind(limit as i32).fetch_optional(&self.pool).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        admitted.map(|_| ()).ok_or(StatusCode::TOO_MANY_REQUESTS)
    }
}

impl AppState {
    async fn take_auth_attempt(
        &self,
        scope: &str,
        value: &str,
        limit: u32,
    ) -> Result<(), StatusCode> {
        let key = key(scope, value);
        if let Some(pg) = &self.db {
            pg.take_auth_attempt(&key, limit).await
        } else {
            self.auth_limits.take_memory(&key, limit, Instant::now())
        }
    }
    pub(crate) async fn limit_account_auth(&self, email: &str) -> Result<(), StatusCode> {
        self.take_auth_attempt("account", &email.trim().to_lowercase(), 10)
            .await
    }
}

fn sensitive(method: &Method, path: &str) -> bool {
    (method == Method::POST && path == "/v1/session")
        || (method != Method::GET && path.starts_with("/v1/admin/ai/"))
        || (method != Method::GET && path.starts_with("/v1/admin/oauth/"))
        || (method != Method::GET && path.starts_with("/v1/admin/notifications/"))
        || (method != Method::GET && path.starts_with("/v1/admin/mail/"))
        || (method != Method::GET && path.starts_with("/v1/admin/mock-billing/"))
        || (method == Method::POST
            && matches!(path, "/v1/billing/mock/redeem" | "/v1/billing/checkout"))
        || (method != Method::GET
            && (path.starts_with("/v1/session/identity/")
                || path.starts_with("/v1/admin/identity/")
                || path.ends_with("/password-reset")))
        || (method == Method::PUT && path == "/v1/admin/security/password")
        || (method == Method::POST
            && path.starts_with("/v1/admin/users/")
            && path.ends_with("/actions"))
}
pub async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if !sensitive(request.method(), request.uri().path()) {
        return next.run(request).await;
    }
    // Never trust arbitrary forwarding headers. The production listener supplies ConnectInfo.
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip().to_string())
        .unwrap_or_else(|| "no-peer".into());
    let admitted = match state
        .take_auth_attempt("global", "authentication", 300)
        .await
    {
        Ok(()) => state.take_auth_attempt("peer", &peer, 60).await,
        Err(status) => Err(status),
    };
    let mut response = match admitted {
        Ok(()) => next.run(request).await,
        Err(status) => (status, Json(json!({"message": if status == StatusCode::TOO_MANY_REQUESTS {"操作过于频繁，请一分钟后重试"} else {"认证服务暂不可用，请稍后重试"}}))).into_response(),
    };
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        response
            .headers_mut()
            .insert("retry-after", HeaderValue::from_static("60"));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_security_tests::{request, state};
    use axum::{body::Body, http::Request as HttpRequest};
    use tower::ServiceExt;
    #[test]
    fn memory_window_expires_and_scopes_do_not_share_buckets() {
        let limits = AuthLimits::default();
        let now = Instant::now();
        for _ in 0..10 {
            limits.take_memory("a", 10, now).unwrap();
        }
        assert_eq!(
            limits.take_memory("a", 10, now),
            Err(StatusCode::TOO_MANY_REQUESTS)
        );
        limits.take_memory("b", 10, now).unwrap();
        limits
            .take_memory("a", 10, now + std::time::Duration::from_secs(60))
            .unwrap();
        assert_ne!(key("peer", "same"), key("account", "same"));
        assert!(!key("account", "name@example.com").contains("name@example.com"));
    }
    #[tokio::test]
    async fn postgres_limits_survive_new_state_are_atomic_and_expire() {
        let state = state().await;
        let pg = state.db.as_ref().unwrap();
        let mut jobs = Vec::new();
        for _ in 0..20 {
            let pg = pg.clone();
            jobs.push(tokio::spawn(async move {
                pg.take_auth_attempt("parallel", 10).await
            }));
        }
        let mut admitted = 0;
        for job in jobs {
            if job.await.unwrap().is_ok() {
                admitted += 1;
            }
        }
        assert_eq!(admitted, 10);
        let reopened = Pg::new(pg.pool.clone(), &pg.schema).unwrap();
        reopened.apply_schema(false).await.unwrap();
        let restarted = AppState {
            db: Some(reopened),
            ..AppState::default()
        };
        assert_eq!(
            restarted
                .db
                .as_ref()
                .unwrap()
                .take_auth_attempt("parallel", 10)
                .await,
            Err(StatusCode::TOO_MANY_REQUESTS)
        );
        sqlx::query(&format!(
            "UPDATE {} SET expires_at=now()-interval '1 second'",
            pg.t("auth_limits")
        ))
        .execute(&pg.pool)
        .await
        .unwrap();
        pg.take_auth_attempt("parallel", 10).await.unwrap();
        sqlx::query(&format!("DROP TABLE {}", pg.t("auth_limits")))
            .execute(&pg.pool)
            .await
            .unwrap();
        assert_eq!(
            pg.take_auth_attempt("parallel", 10).await,
            Err(StatusCode::SERVICE_UNAVAILABLE)
        );
    }
    #[tokio::test]
    async fn nonexistent_login_and_reauthentication_have_account_limits() {
        let state = state().await;
        for _ in 0..10 {
            assert_eq!(
                request(
                    &state,
                    "POST",
                    "/v1/session",
                    "",
                    json!({"email":"absent@example.com","password":"wrong"})
                )
                .await
                .0,
                StatusCode::UNAUTHORIZED
            );
        }
        assert_eq!(
            request(
                &state,
                "POST",
                "/v1/session",
                "",
                json!({"email":"ABSENT@example.com","password":"correct"})
            )
            .await
            .0,
            StatusCode::TOO_MANY_REQUESTS
        );
        let pg = state.db.as_ref().unwrap();
        pg.upsert_account("admin@example.com", Some("administrator-password"), "admin")
            .await
            .unwrap();
        let token = state
            .issue_session("admin@example.com".into())
            .await
            .unwrap()
            .access_token;
        for _ in 0..10 {
            assert_eq!(
                request(
                    &state,
                    "PUT",
                    "/v1/admin/security/password",
                    &token,
                    json!({"current_password":"wrong","new_password":"changed-password"})
                )
                .await
                .0,
                StatusCode::UNPROCESSABLE_ENTITY
            );
        }
        assert_eq!(request(&state,"PUT","/v1/admin/security/password",&token,json!({"current_password":"administrator-password","new_password":"changed-password"})).await.0,StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            request(
                &state,
                "POST",
                "/v1/admin/users/other/actions",
                &token,
                json!({"action":"disable","current_password":"administrator-password","reason":"限流测试"})
            )
            .await
            .0,
            StatusCode::TOO_MANY_REQUESTS
        );
        pg.password_session("admin@example.com", "administrator-password")
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn peer_limit_ignores_forged_headers_and_emits_retry_after() {
        let state = AppState::default();
        for index in 0..61 {
            let response=crate::app(state.clone()).oneshot(HttpRequest::builder().method("POST").uri("/v1/session")
                .header("content-type","application/json").header("x-forwarded-for",format!("10.0.0.{index}"))
                .extension(ConnectInfo("127.0.0.1:43210".parse::<SocketAddr>().unwrap()))
                .body(Body::from(json!({"email":format!("unknown{index}@example.com"),"password":"wrong"}).to_string())).unwrap()).await.unwrap();
            assert_eq!(
                response.status(),
                if index < 60 {
                    StatusCode::UNAUTHORIZED
                } else {
                    StatusCode::TOO_MANY_REQUESTS
                }
            );
            if index == 60 {
                assert_eq!(response.headers()["retry-after"], "60");
            }
        }
        assert_eq!(
            request(&state, "GET", "/v1/health", "", json!(null))
                .await
                .0,
            StatusCode::OK
        );
    }
}
