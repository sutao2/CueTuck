use crate::{
    bearer_token, hash_password, postgres::Pg, require_staff, verify_password, AppState,
    SessionResponse,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

fn database_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

pub fn valid_password(password: &str) -> bool {
    (12..=128).contains(&password.chars().count()) && password.len() <= 512
}

impl Pg {
    pub async fn initialize_admin(
        &self,
        email: Option<&str>,
        password: Option<&str>,
        development: bool,
    ) -> Result<(), String> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| "无法开始管理员初始化")?;
        sqlx::query(&format!(
            "LOCK TABLE {} IN SHARE ROW EXCLUSIVE MODE",
            self.t("accounts")
        ))
        .execute(&mut *tx)
        .await
        .map_err(|_| "无法锁定管理员初始化")?;
        let exists: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE role IN ('admin','owner'))",
            self.t("accounts")
        ))
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| "无法检查管理员")?;
        if !exists {
            let initialized: bool = sqlx::query_scalar(&format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE key='admin_initialized')",
                self.t("settings")
            ))
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| "无法检查初始化标记")?;
            if initialized {
                return Err(
                    "已初始化但没有管理员，请通过受控账号恢复流程处理，不会自动重建管理员".into(),
                );
            }
            let email = email
                .or(if development {
                    Some("admin@promptark.local")
                } else {
                    None
                })
                .map(str::trim)
                .filter(|s| !s.is_empty() && s.contains('@') && s.len() <= 254)
                .ok_or("首次启动须设置 PROMPTARK_ADMIN_EMAIL")?;
            let password = password
                .or(if development { Some("adminpass") } else { None })
                .ok_or("首次启动须设置 PROMPTARK_ADMIN_PASSWORD")?;
            if (!development && !valid_password(password))
                || password.is_empty()
                || password.len() > 512
            {
                return Err("初始管理员密码须为 12–128 字符，默认短密码仅限显式开发模式".into());
            }
            let inserted = sqlx::query(&format!("INSERT INTO {} (email,password_hash,role) VALUES ($1,$2,'owner') ON CONFLICT(email) DO NOTHING", self.t("accounts")))
                .bind(email).bind(hash_password(password)).execute(&mut *tx).await.map_err(|_| "无法创建初始管理员")?;
            if inserted.rows_affected() != 1 {
                return Err("初始管理员邮箱已被普通账号使用，不能隐式提权".into());
            }
            sqlx::query(&format!("INSERT INTO {} (key,value) VALUES ('owner_initialized','true') ON CONFLICT DO NOTHING", self.t("settings")))
                .execute(&mut *tx).await.map_err(|_| "无法保存所有者初始化标记")?;
        }
        sqlx::query(&format!("INSERT INTO {} (key,value) VALUES ('admin_initialized','true') ON CONFLICT(key) DO NOTHING", self.t("settings")))
            .execute(&mut *tx).await.map_err(|_| "无法保存初始化标记")?;
        tx.commit().await.map_err(|_| "无法完成管理员初始化".into())
    }

    pub async fn insert_development_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(), StatusCode> {
        sqlx::query(&format!("INSERT INTO {} (email,password_hash,role) VALUES ($1,$2,'user') ON CONFLICT(email) DO NOTHING", self.t("accounts")))
            .bind(email).bind(hash_password(password)).execute(&self.pool).await.map_err(database_error)?;
        Ok(())
    }

    async fn lock_account(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        email: &str,
    ) -> Result<sqlx::postgres::PgRow, StatusCode> {
        sqlx::query(&format!(
            "SELECT password_hash,role FROM {} WHERE email=$1 AND NOT disabled FOR UPDATE",
            self.t("accounts")
        ))
        .bind(email)
        .fetch_optional(&mut **tx)
        .await
        .map_err(database_error)?
        .ok_or(StatusCode::UNAUTHORIZED)
    }

    async fn insert_session_pair(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        email: &str,
    ) -> Result<SessionResponse, StatusCode> {
        let access_token = format!("acc.{}", Uuid::new_v4());
        let refresh_token = format!("ref.{}", Uuid::new_v4());
        for (table, token) in [
            ("access_tokens", &access_token),
            ("refresh_tokens", &refresh_token),
        ] {
            sqlx::query(&format!(
                "INSERT INTO {} (token,email) VALUES ($1,$2)",
                self.t(table)
            ))
            .bind(token)
            .bind(email)
            .execute(&mut **tx)
            .await
            .map_err(database_error)?;
        }
        Ok(SessionResponse {
            email: email.into(),
            access_token,
            refresh_token,
        })
    }

    pub async fn password_session(
        &self,
        email: &str,
        password: &str,
    ) -> Result<SessionResponse, StatusCode> {
        let mut tx = self.pool.begin().await.map_err(database_error)?;
        let row = self.lock_account(&mut tx, email).await?;
        let hash: Option<String> = row.get("password_hash");
        if !hash.is_some_and(|hash| verify_password(password, &hash)) {
            return Err(StatusCode::UNAUTHORIZED);
        }
        let session = self.insert_session_pair(&mut tx, email).await?;
        tx.commit().await.map_err(database_error)?;
        Ok(session)
    }

    pub async fn issue_session(&self, email: &str) -> Result<(String, String), StatusCode> {
        let mut tx = self.pool.begin().await.map_err(database_error)?;
        self.lock_account(&mut tx, email).await?;
        let session = self.insert_session_pair(&mut tx, email).await?;
        tx.commit().await.map_err(database_error)?;
        Ok((session.access_token, session.refresh_token))
    }

    pub async fn refresh_session(&self, token: &str) -> Result<SessionResponse, StatusCode> {
        let mut tx = self.pool.begin().await.map_err(database_error)?;
        let email: String = sqlx::query_scalar(&format!(
            "SELECT email FROM {} WHERE token=$1",
            self.t("refresh_tokens")
        ))
        .bind(token)
        .fetch_optional(&mut *tx)
        .await
        .map_err(database_error)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
        self.lock_account(&mut tx, &email).await?;
        // Recheck after waiting for the account lock: password change/revocation may have removed the token.
        let removed = sqlx::query(&format!(
            "DELETE FROM {} WHERE token=$1",
            self.t("refresh_tokens")
        ))
        .bind(token)
        .execute(&mut *tx)
        .await
        .map_err(database_error)?;
        if removed.rows_affected() != 1 {
            return Err(StatusCode::UNAUTHORIZED);
        }
        sqlx::query(&format!(
            "DELETE FROM {} WHERE email=$1",
            self.t("access_tokens")
        ))
        .bind(&email)
        .execute(&mut *tx)
        .await
        .map_err(database_error)?;
        let session = self.insert_session_pair(&mut tx, &email).await?;
        tx.commit().await.map_err(database_error)?;
        Ok(session)
    }

    pub async fn security_view(&self, email: &str) -> Result<Value, StatusCode> {
        let row = sqlx::query(&format!("SELECT (password_hash IS NOT NULL AND password_hash <> '') AS has_password, (SELECT count(*) FROM {} WHERE email=$1) AS active_access_count FROM {} WHERE email=$1", self.t("access_tokens"), self.t("accounts")))
            .bind(email).fetch_one(&self.pool).await.map_err(database_error)?;
        Ok(
            json!({"has_password":row.get::<bool,_>("has_password"), "active_access_count":row.get::<i64,_>("active_access_count")}),
        )
    }

    pub async fn secure_account(
        &self,
        email: &str,
        token: &str,
        password: Option<&PasswordChange>,
    ) -> Result<(), StatusCode> {
        let mut tx = self.pool.begin().await.map_err(database_error)?;
        // Match management/initialization lock order before any account-row lock.
        sqlx::query(&format!("LOCK TABLE {} IN SHARE ROW EXCLUSIVE MODE", self.t("accounts")))
            .execute(&mut *tx).await.map_err(database_error)?;
        let row = self.lock_account(&mut tx, email).await?;
        let valid: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE token=$1 AND email=$2)",
            self.t("access_tokens")
        ))
        .bind(token)
        .bind(email)
        .fetch_one(&mut *tx)
        .await
        .map_err(database_error)?;
        if !valid {
            return Err(StatusCode::UNAUTHORIZED);
        }
        if !crate::admin_users::staff(&row.get::<String, _>("role")) {
            return Err(StatusCode::FORBIDDEN);
        }
        if let Some(input) = password {
            let hash: Option<String> = row.get("password_hash");
            if !hash.is_some_and(|hash| verify_password(&input.current_password, &hash)) {
                return Err(StatusCode::UNPROCESSABLE_ENTITY);
            }
            sqlx::query(&format!(
                "UPDATE {} SET password_hash=$1 WHERE email=$2",
                self.t("accounts")
            ))
            .bind(hash_password(&input.new_password))
            .bind(email)
            .execute(&mut *tx)
            .await
            .map_err(database_error)?;
        }
        for table in ["access_tokens", "refresh_tokens"] {
            sqlx::query(&format!("DELETE FROM {} WHERE email=$1", self.t(table)))
                .bind(email)
                .execute(&mut *tx)
                .await
                .map_err(database_error)?;
        }
        let action = if password.is_some() {
            "password_changed"
        } else {
            "sessions_revoked"
        };
        sqlx::query(&format!(
            "INSERT INTO {} (id,actor_email,action) VALUES ($1,$2,$3)",
            self.t("security_audit")
        ))
        .bind(Uuid::new_v4().to_string())
        .bind(email)
        .bind(action)
        .execute(&mut *tx)
        .await
        .map_err(database_error)?;
        tx.commit().await.map_err(database_error)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PasswordChange {
    current_password: String,
    new_password: String,
}

type ApiError = (StatusCode, Json<Value>);
fn api_error(status: StatusCode) -> ApiError {
    let message = match status {
        StatusCode::TOO_MANY_REQUESTS => "密码验证过于频繁，请一分钟后重试",
        StatusCode::UNAUTHORIZED => "登录已失效，请重新登录",
        StatusCode::FORBIDDEN => "需要管理员权限",
        StatusCode::UNPROCESSABLE_ENTITY => "当前密码不正确，或此账号尚未设置密码",
        StatusCode::BAD_REQUEST => "新密码须为 12–128 字符，且不能与当前密码相同",
        StatusCode::SERVICE_UNAVAILABLE => "账号安全服务需要数据库连接",
        _ => "操作未完成，请稍后重试",
    };
    (status, Json(json!({"message":message})))
}

pub async fn view(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let email = require_staff(&state, &headers).await.map_err(api_error)?;
    let pg = state
        .db
        .as_ref()
        .ok_or_else(|| api_error(StatusCode::SERVICE_UNAVAILABLE))?;
    Ok(Json(pg.security_view(&email).await.map_err(api_error)?))
}

pub async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PasswordChange>,
) -> Result<Json<Value>, ApiError> {
    require_staff(&state, &headers).await.map_err(api_error)?;
    if !valid_password(&input.new_password)
        || input.current_password.len() > 512
        || input.new_password == input.current_password
    {
        return Err(api_error(StatusCode::BAD_REQUEST));
    }
    secure(&state, &headers, Some(&input)).await
}

pub async fn revoke_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    secure(&state, &headers, None).await
}

async fn secure(
    state: &AppState,
    headers: &HeaderMap,
    input: Option<&PasswordChange>,
) -> Result<Json<Value>, ApiError> {
    let email = require_staff(state, headers).await.map_err(api_error)?;
    let token = bearer_token(headers).ok_or_else(|| api_error(StatusCode::UNAUTHORIZED))?;
    let pg = state
        .db
        .as_ref()
        .ok_or_else(|| api_error(StatusCode::SERVICE_UNAVAILABLE))?;
    if input.is_some() { state.limit_account_auth(&email).await.map_err(api_error)?; }
    pg.secure_account(&email, &token, input)
        .await
        .map_err(api_error)?;
    Ok(Json(json!({"signed_out":true})))
}
