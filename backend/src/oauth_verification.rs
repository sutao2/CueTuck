use crate::{
    admin_risk::{actor_lock, audit},
    bearer_token,
    oauth::{OAuthCallbackQuery, ProviderConfig},
    postgres::Pg,
    require_configuration_admin, AppState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use ring::rand::{SecureRandom, SystemRandom};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::Duration;
fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
pub(crate) fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
pub(crate) async fn config_lock(
    pg: &Pg,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), StatusCode> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{}:oauth-config", pg.schema))
        .execute(&mut **tx)
        .await
        .map_err(db_error)?;
    Ok(())
}
pub(crate) async fn owner_lock(
    pg: &Pg,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &str,
    token: &str,
) -> Result<(), StatusCode> {
    actor_lock(pg, tx, actor, token, true).await?;
    let owner: bool = sqlx::query_scalar(&format!(
        "SELECT role='owner' FROM {} WHERE email=$1",
        pg.t("accounts")
    ))
    .bind(actor)
    .fetch_one(&mut **tx)
    .await
    .map_err(db_error)?;
    if owner {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}
impl Pg {
    pub async fn init_oauth_verification(&self) -> Result<(), sqlx::Error> {
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY,provider TEXT NOT NULL,actor TEXT NOT NULL,fingerprint TEXT NOT NULL,status TEXT NOT NULL,message TEXT NOT NULL DEFAULT '',created_at TIMESTAMPTZ NOT NULL DEFAULT now(),expires_at TIMESTAMPTZ NOT NULL DEFAULT now()+interval '10 minutes',finished_at TIMESTAMPTZ)",self.t("oauth_verifications"))).execute(&self.pool).await?;
        Ok(())
    }
}
impl AppState {
    pub(crate) async fn oauth_fingerprint(&self, provider: &str) -> Result<String, StatusCode> {
        let revision = self.oauth_revision(provider).await?;
        let value = match self.runtime_provider(provider).await? {
            Some(c) => json!([
                revision,
                c.client_id,
                c.client_secret,
                c.redirect_uri,
                c.authorization_uri,
                c.token_uri,
                c.user_info_uri,
                c.emails_uri
            ]),
            None => json!([revision, "disabled"]),
        };
        Ok(digest(&value.to_string()))
    }
    pub(crate) async fn oauth_verification_view(
        &self,
        provider: &str,
    ) -> Result<Value, StatusCode> {
        let Some(pg) = &self.db else {
            return Ok(json!({"status":"unverified"}));
        };
        let latest:Option<Value>=sqlx::query_scalar(&format!("SELECT json_build_object('status',CASE WHEN status IN ('pending','verifying') AND expires_at<now() THEN 'expired' ELSE status END,'message',message,'created_at',created_at,'finished_at',finished_at,'fingerprint',fingerprint) FROM {} WHERE provider=$1 ORDER BY created_at DESC,id DESC LIMIT 1",pg.t("oauth_verifications"))).bind(provider).fetch_optional(&pg.pool).await.map_err(db_error)?;
        let Some(mut latest) = latest else {
            return Ok(json!({"status":"unverified"}));
        };
        if latest["fingerprint"] != self.oauth_fingerprint(provider).await? {
            latest["status"] = json!("configuration_changed");
            latest["message"] = json!("配置已变化，请重新验证");
        }
        latest.as_object_mut().unwrap().remove("fingerprint");
        Ok(latest)
    }
}
fn authorization_url(config: &ProviderConfig, nonce: &str) -> Result<String, StatusCode> {
    let mut url = url::Url::parse(&config.authorization_uri)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", &config.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", &config.scope)
        .append_pair("state", nonce);
    Ok(url.to_string())
}
pub async fn start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_configuration_admin(&state, &headers).await?;
    if !crate::oauth_admin::PROVIDERS.contains(&provider.as_str()) {
        return Err(StatusCode::NOT_FOUND);
    }
    state.limit_account_auth(&actor).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    owner_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
    )
    .await?;
    config_lock(pg, &mut tx).await?;
    let config = state
        .runtime_provider(&provider)
        .await?
        .ok_or(StatusCode::CONFLICT)?;
    let mut bytes = [0u8; 32];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let raw = format!(
        "admin-test.{}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    );
    let id = digest(&raw);
    let fingerprint = state.oauth_fingerprint(&provider).await?;
    sqlx::query(&format!("UPDATE {} SET status='cancelled',message='已发起新的验证',finished_at=now() WHERE provider=$1 AND status IN ('pending','verifying')",pg.t("oauth_verifications"))).bind(&provider).execute(&mut *tx).await.map_err(db_error)?;
    sqlx::query(&format!(
        "INSERT INTO {} (id,provider,actor,fingerprint,status) VALUES ($1,$2,$3,$4,'pending')",
        pg.t("oauth_verifications")
    ))
    .bind(&id)
    .bind(&provider)
    .bind(&actor)
    .bind(&fingerprint)
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    audit(
        pg,
        &mut tx,
        &actor,
        "oauth_verification_started",
        json!({"provider":provider}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(
        json!({"authorization_url":authorization_url(&config,&raw)?,"expires_in":600}),
    ))
}
pub async fn callback(
    state: &AppState,
    query: &OAuthCallbackQuery,
) -> Result<Response, StatusCode> {
    let signed = query.state.as_deref().ok_or(StatusCode::BAD_REQUEST)?;
    if signed.len() != 75 || !signed.starts_with("admin-test.") {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let id = digest(signed);
    let claim:Option<(String,String,String)>=sqlx::query_as(&format!("UPDATE {} SET status='verifying' WHERE id=$1 AND status='pending' AND expires_at>now() RETURNING provider,actor,fingerprint",pg.t("oauth_verifications"))).bind(&id).fetch_optional(&pg.pool).await.map_err(db_error)?;
    let (provider, actor, fingerprint) = claim.ok_or(StatusCode::BAD_REQUEST)?;
    let mut result = if query.error.as_deref().is_some_and(|e| !e.is_empty()) {
        Err("提供商拒绝授权或用户取消")
    } else if state.oauth_fingerprint(&provider).await? != fingerprint {
        Err("配置已变化，请重新验证")
    } else if let Some(code) = query
        .code
        .as_deref()
        .filter(|c| !c.is_empty() && c.len() <= 8192)
    {
        let config = state
            .runtime_provider(&provider)
            .await?
            .ok_or(StatusCode::CONFLICT)?;
        // Test-only in-memory identities must never certify a real provider configuration.
        if state.oauth.mock_users.contains_key(code) {
            Err("模拟身份不能作为真实授权验证")
        } else {
            match tokio::time::timeout(
                Duration::from_secs(25),
                crate::oauth::fetch_verified_user(&config, &provider, code),
            )
            .await
            {
                Ok(Ok(_)) => Ok(()),
                _ => Err("提供商请求失败或未返回已验证邮箱，请检查凭据、回调和授权范围"),
            }
        }
    } else {
        Err("未收到有效授权码")
    };
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    let owner: Option<bool> = sqlx::query_scalar(&format!(
        "SELECT role='owner' AND NOT disabled FROM {} WHERE email=$1 FOR SHARE",
        pg.t("accounts")
    ))
    .bind(&actor)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_error)?;
    config_lock(pg, &mut tx).await?;
    if owner != Some(true) {
        result = Err("发起人的管理权限已失效")
    }
    if state.oauth_fingerprint(&provider).await? != fingerprint {
        result = Err("配置已变化，请重新验证")
    }
    let success = result.is_ok();
    let message = result
        .err()
        .unwrap_or("真实授权验证成功，未创建账号或登录会话");
    let count=sqlx::query(&format!("UPDATE {} SET status=$2,message=$3,finished_at=now() WHERE id=$1 AND status='verifying' AND expires_at>now()",pg.t("oauth_verifications"))).bind(&id).bind(if success{"succeeded"}else{"failed"}).bind(message).execute(&mut *tx).await.map_err(db_error)?.rows_affected();
    if count == 0 {
        return Err(StatusCode::CONFLICT);
    }
    audit(pg,&mut tx,&actor,"oauth_verification_finished",json!({"provider":provider,"status":if success{"succeeded"}else{"failed"},"reason":message})).await?;
    tx.commit().await.map_err(db_error)?;
    let mut response=Html(format!("<!doctype html><html lang=zh-CN><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><title>授权验证</title><body><h1>{}</h1><p>{message}</p><p>返回管理端，点击「刷新验证结果」。可以关闭此窗口。</p></body></html>",if success{"验证成功"}else{"验证未通过"})).into_response();
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
        .headers_mut()
        .insert("referrer-policy", "no-referrer".parse().unwrap());
    Ok(response)
}
