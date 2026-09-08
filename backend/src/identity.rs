use crate::{
    admin_risk::{actor_lock, audit},
    bearer_token,
    postgres::Pg,
    require_admin, require_configuration_admin, AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::Row;
fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
fn normalize(email: &str) -> Result<String, StatusCode> {
    let value = email.trim().to_lowercase();
    if !crate::mail_transport::address(&value) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(value)
}
fn accepted() -> (StatusCode, Json<Value>) {
    (
        StatusCode::ACCEPTED,
        Json(
            json!({"message":"若该邮箱符合条件，将发送验证邮件；请检查邮箱，稍后重试前先查看垃圾邮件。"}),
        ),
    )
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub revision: i64,
    pub registration_open: bool,
}
impl Pg {
    pub async fn init_identity(&self) -> Result<(), sqlx::Error> {
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY CHECK(id=1),data JSONB NOT NULL)",
            self.t("identity_policy")
        ))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!(
            "INSERT INTO {} (id,data) VALUES (1,$1) ON CONFLICT DO NOTHING",
            self.t("identity_policy")
        ))
        .bind(json!(Policy {
            revision: 0,
            registration_open: true
        }))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY,token_hash TEXT NOT NULL UNIQUE,email TEXT NOT NULL,purpose TEXT NOT NULL,role TEXT NOT NULL,inviter TEXT,status TEXT NOT NULL DEFAULT 'pending',revision BIGINT NOT NULL DEFAULT 0,mail_id TEXT NOT NULL,created_at TIMESTAMPTZ NOT NULL DEFAULT now(),expires_at TIMESTAMPTZ NOT NULL,used_at TIMESTAMPTZ)",self.t("identity_challenges"))).execute(&self.pool).await?;
        Ok(())
    }
    pub(crate) async fn identity_lock(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), StatusCode> {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(format!("{}:identity", self.schema))
            .execute(&mut **tx)
            .await
            .map_err(db_error)?;
        Ok(())
    }
    pub async fn identity_policy(&self) -> Result<Policy, StatusCode> {
        let value: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1",
            self.t("identity_policy")
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;
        serde_json::from_value(value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
    async fn policy_in(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Policy, StatusCode> {
        let value: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1 FOR SHARE",
            self.t("identity_policy")
        ))
        .fetch_one(&mut **tx)
        .await
        .map_err(db_error)?;
        serde_json::from_value(value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
    async fn challenge(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        key: &[u8; 32],
        email: &str,
        purpose: &str,
        role: &str,
        actor: Option<&str>,
    ) -> Result<Option<String>, StatusCode> {
        let value: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1 FOR SHARE",
            self.t("mail_configuration")
        ))
        .fetch_one(&mut **tx)
        .await
        .map_err(db_error)?;
        let mail: crate::admin_mail::Config =
            serde_json::from_value(value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if !mail.enabled || mail.encrypted_secret.is_empty() {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
        let count:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {} WHERE lower(email)=lower($1) AND created_at>now()-interval '24 hours'",self.t("identity_challenges"))).bind(email).fetch_one(&mut **tx).await.map_err(db_error)?;
        if count >= 5 {
            return Ok(None);
        }
        sqlx::query(&format!("UPDATE {} SET status='expired',payload='',claim=NULL,revision=revision+1,updated_at=now() WHERE id IN (SELECT mail_id FROM {} WHERE lower(email)=lower($1) AND purpose=$2 AND status='pending') AND status IN ('queued','failed','sending')",self.t("mail_outbox"),self.t("identity_challenges"))).bind(email).bind(purpose).execute(&mut **tx).await.map_err(db_error)?;
        sqlx::query(&format!("UPDATE {} SET status='revoked',revision=revision+1 WHERE lower(email)=lower($1) AND purpose=$2 AND status='pending'",self.t("identity_challenges"))).bind(email).bind(purpose).execute(&mut **tx).await.map_err(db_error)?;
        use ring::rand::SecureRandom;
        let mut bytes = [0u8; 32];
        ring::rand::SystemRandom::new()
            .fill(&mut bytes)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        let hash = format!("{:x}", Sha256::digest(token.as_bytes()));
        let id = uuid::Uuid::new_v4().to_string();
        let label = match purpose {
            "registration" => "注册验证",
            "invitation" => "管理员邀请",
            _ => "密码重置",
        };
        let body=format!("你请求了 PromptArk {label}。请在软件的邮箱验证页面选择对应用途，填写邮箱、下方验证码和新密码。\n\n验证码：\n{token}\n\n30 分钟内有效，仅可使用一次。不要将验证码转发给任何人。如果不是你本人或你确认的管理员发起，请忽略本邮件。");
        let mail_id = self
            .queue_mail(
                tx,
                key,
                mail.revision,
                purpose,
                email,
                &format!("PromptArk {label}"),
                &body,
                actor.unwrap_or("self-service"),
                1800,
            )
            .await?;
        sqlx::query(&format!("INSERT INTO {} (id,token_hash,email,purpose,role,inviter,mail_id,expires_at) VALUES ($1,$2,$3,$4,$5,$6,$7,now()+interval '30 minutes')",self.t("identity_challenges"))).bind(&id).bind(hash).bind(email).bind(purpose).bind(role).bind(actor).bind(mail_id).execute(&mut **tx).await.map_err(db_error)?;
        audit(
            self,
            tx,
            actor.unwrap_or("self-service"),
            "identity_verification_requested",
            json!({"challenge_id":id,"email":email,"purpose":purpose,"role":role}),
        )
        .await?;
        Ok(Some(id))
    }
    async fn account_matches(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        email: &str,
    ) -> Result<Vec<sqlx::postgres::PgRow>, StatusCode> {
        sqlx::query(&format!(
            "SELECT email,role,disabled FROM {} WHERE lower(email)=lower($1) FOR UPDATE",
            self.t("accounts")
        ))
        .bind(email)
        .fetch_all(&mut **tx)
        .await
        .map_err(db_error)
    }
}
pub async fn options(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let policy = pg.identity_policy().await?;
    let mail = pg.mail_config().await?;
    Ok(Json(
        json!({"registration_open":policy.registration_open,"email_enabled":mail.enabled&&!mail.encrypted_secret.is_empty()}),
    ))
}
pub async fn get_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Policy>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    Ok(Json(
        state
            .db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
            .identity_policy()
            .await?,
    ))
}
async fn admin_in(
    state: &AppState,
    pg: &Pg,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    headers: &HeaderMap,
    password: Option<&str>,
    owner: bool,
) -> Result<(String, String), StatusCode> {
    let actor = require_admin(state, headers).await?;
    actor_lock(
        pg,
        tx,
        &actor,
        &bearer_token(headers).ok_or(StatusCode::UNAUTHORIZED)?,
        true,
    )
    .await?;
    let row = sqlx::query(&format!(
        "SELECT role,password_hash FROM {} WHERE email=$1",
        pg.t("accounts")
    ))
    .bind(&actor)
    .fetch_one(&mut **tx)
    .await
    .map_err(db_error)?;
    let role: String = row.get("role");
    if owner && role != "owner" {
        return Err(StatusCode::FORBIDDEN);
    }
    if let Some(password) = password {
        let hash: Option<String> = row.get("password_hash");
        if password.len() > 512 || !hash.is_some_and(|h| crate::verify_password(password, &h)) {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    Ok((actor, role))
}
pub async fn save_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut input): Json<Policy>,
) -> Result<Json<Policy>, StatusCode> {
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    pg.identity_lock(&mut tx).await?;
    let (actor, _) = admin_in(&state, pg, &mut tx, &headers, None, true).await?;
    let current = pg.policy_in(&mut tx).await?;
    if input.revision != current.revision {
        return Err(StatusCode::CONFLICT);
    }
    input.revision += 1;
    sqlx::query(&format!(
        "UPDATE {} SET data=$1 WHERE id=1",
        pg.t("identity_policy")
    ))
    .bind(json!(input))
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    audit(
        pg,
        &mut tx,
        &actor,
        "identity_policy_saved",
        json!({"before":current,"after":input}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(input))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    kind: String,
    email: String,
}
pub async fn request_code(
    State(state): State<AppState>,
    Json(input): Json<Request>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let email = normalize(&input.email)?;
    if !["registration", "reset"].contains(&input.kind.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    state.limit_account_auth(&email).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mail = pg.mail_config().await?;
    if !mail.enabled || mail.encrypted_secret.is_empty() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    pg.identity_lock(&mut tx).await?;
    let policy = pg.policy_in(&mut tx).await?;
    if input.kind == "registration" && !policy.registration_open {
        return Err(StatusCode::FORBIDDEN);
    }
    let accounts = pg.account_matches(&mut tx, &email).await?;
    if input.kind == "registration" && accounts.is_empty() {
        pg.challenge(
            &mut tx,
            &state.oauth_config.key,
            &email,
            "registration",
            "user",
            None,
        )
        .await?;
    } else if input.kind == "reset"
        && accounts.len() == 1
        && !accounts[0].get::<bool, _>("disabled")
    {
        pg.challenge(
            &mut tx,
            &state.oauth_config.key,
            &accounts[0].get::<String, _>("email"),
            "reset",
            "user",
            None,
        )
        .await?;
    }
    tx.commit().await.map_err(db_error)?;
    Ok(accepted())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Confirm {
    kind: String,
    email: String,
    token: String,
    new_password: String,
}
pub async fn confirm(
    State(state): State<AppState>,
    Json(input): Json<Confirm>,
) -> Result<Json<Value>, StatusCode> {
    let email = normalize(&input.email)?;
    state.limit_account_auth(&email).await?;
    if !["registration", "reset", "invitation"].contains(&input.kind.as_str())
        || input.token.len() != 64
        || !input.token.bytes().all(|b| b.is_ascii_hexdigit())
        || !crate::admin_security::valid_password(&input.new_password)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    pg.identity_lock(&mut tx).await?;
    let policy = pg.policy_in(&mut tx).await?;
    if input.kind == "registration" && !policy.registration_open {
        return Err(StatusCode::BAD_REQUEST);
    }
    let hash = format!("{:x}", Sha256::digest(input.token.as_bytes()));
    let challenge=sqlx::query(&format!("SELECT id,email,role,inviter FROM {} WHERE token_hash=$1 AND lower(email)=$2 AND purpose=$3 AND status='pending' AND expires_at>now() FOR UPDATE",pg.t("identity_challenges"))).bind(hash).bind(&email).bind(&input.kind).fetch_optional(&mut *tx).await.map_err(db_error)?.ok_or(StatusCode::BAD_REQUEST)?;
    let id: String = challenge.get("id");
    let actual: String = challenge.get("email");
    let role: String = challenge.get("role");
    if input.kind == "invitation" {
        let inviter: Option<String> = challenge.get("inviter");
        let valid: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE email=$1 AND role='owner' AND NOT disabled)",
            pg.t("accounts")
        ))
        .bind(&inviter)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        if !valid || !["admin", "reviewer"].contains(&role.as_str()) {
            return Err(StatusCode::BAD_REQUEST);
        }
        // Hold the inviter through commit; revocation cannot race the acceptance.
        sqlx::query(&format!(
            "SELECT email FROM {} WHERE email=$1 AND role='owner' AND NOT disabled FOR SHARE",
            pg.t("accounts")
        ))
        .bind(inviter)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?
        .ok_or(StatusCode::BAD_REQUEST)?;
    }
    let accounts = pg.account_matches(&mut tx, &actual).await?;
    if input.kind == "reset" {
        if accounts.len() != 1 || accounts[0].get::<bool, _>("disabled") {
            return Err(StatusCode::BAD_REQUEST);
        }
        sqlx::query(&format!(
            "UPDATE {} SET password_hash=$2 WHERE email=$1",
            pg.t("accounts")
        ))
        .bind(&actual)
        .bind(crate::hash_password(&input.new_password))
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        for table in ["access_tokens", "refresh_tokens"] {
            sqlx::query(&format!("DELETE FROM {} WHERE email=$1", pg.t(table)))
                .bind(&actual)
                .execute(&mut *tx)
                .await
                .map_err(db_error)?;
        }
    } else {
        if !accounts.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
        sqlx::query(&format!(
            "INSERT INTO {} (email,password_hash,role) VALUES ($1,$2,$3)",
            pg.t("accounts")
        ))
        .bind(&actual)
        .bind(crate::hash_password(&input.new_password))
        .bind(if input.kind == "invitation" {
            role.as_str()
        } else {
            "user"
        })
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
    }
    sqlx::query(&format!(
        "UPDATE {} SET status='used',used_at=now(),revision=revision+1 WHERE id=$1",
        pg.t("identity_challenges")
    ))
    .bind(&id)
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    audit(
        pg,
        &mut tx,
        &actual,
        "identity_verified",
        json!({"challenge_id":id,"purpose":input.kind}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(
        json!({"status":"verified","message":"验证完成，请使用新密码登录。"}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invite {
    email: String,
    role: String,
    current_password: String,
}
pub async fn invite(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Invite>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let email = normalize(&input.email)?;
    if !["admin", "reviewer"].contains(&input.role.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let actor = require_configuration_admin(&state, &headers).await?;
    state.limit_account_auth(&actor).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    pg.identity_lock(&mut tx).await?;
    admin_in(
        &state,
        pg,
        &mut tx,
        &headers,
        Some(&input.current_password),
        true,
    )
    .await?;
    if !pg.account_matches(&mut tx, &email).await?.is_empty() {
        return Err(StatusCode::CONFLICT);
    }
    let id = pg
        .challenge(
            &mut tx,
            &state.oauth_config.key,
            &email,
            "invitation",
            &input.role,
            Some(&actor),
        )
        .await?
        .ok_or(StatusCode::TOO_MANY_REQUESTS)?;
    tx.commit().await.map_err(db_error)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id":id,"status":"pending"})),
    ))
}
#[derive(Deserialize, Default)]
pub struct Page {
    #[serde(default)]
    offset: i64,
}
pub async fn invitations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(page): Query<Page>,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    if !(0..=100000).contains(&page.offset) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {} WHERE purpose='invitation'",
        pg.t("identity_challenges")
    ))
    .fetch_one(&pg.pool)
    .await
    .map_err(db_error)?;
    let items:Vec<Value>=sqlx::query_scalar(&format!("SELECT (to_jsonb(c)-'token_hash')||jsonb_build_object('status',CASE WHEN status='pending' AND expires_at<=now() THEN 'expired' ELSE status END) FROM {} c WHERE purpose='invitation' ORDER BY created_at DESC LIMIT 25 OFFSET $1",pg.t("identity_challenges"))).bind(page.offset).fetch_all(&pg.pool).await.map_err(db_error)?;
    Ok(Json(
        json!({"items":items,"total":total,"offset":page.offset,"limit":25}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    revision: i64,
}
pub async fn revoke_invite(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Revision>,
) -> Result<Json<Value>, StatusCode> {
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    pg.identity_lock(&mut tx).await?;
    let (actor, _) = admin_in(&state, pg, &mut tx, &headers, None, true).await?;
    let mail:Option<String>=sqlx::query_scalar(&format!("UPDATE {} SET status='revoked',revision=revision+1 WHERE id=$1 AND purpose='invitation' AND status='pending' AND revision=$2 RETURNING mail_id",pg.t("identity_challenges"))).bind(&id).bind(input.revision).fetch_optional(&mut *tx).await.map_err(db_error)?;
    let mail = mail.ok_or(StatusCode::CONFLICT)?;
    sqlx::query(&format!("UPDATE {} SET status='expired',payload='',claim=NULL,revision=revision+1 WHERE id=$1 AND status IN ('queued','failed','sending')",pg.t("mail_outbox"))).bind(mail).execute(&mut *tx).await.map_err(db_error)?;
    audit(pg, &mut tx, &actor, "invitation_revoked", json!({"id":id})).await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(json!({"status":"revoked"})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reset {
    current_password: String,
}
pub async fn admin_reset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target): Path<String>,
    Json(input): Json<Reset>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    state.limit_account_auth(&actor).await?;
    let email = normalize(&target)?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    pg.identity_lock(&mut tx).await?;
    let (_, role) = admin_in(
        &state,
        pg,
        &mut tx,
        &headers,
        Some(&input.current_password),
        false,
    )
    .await?;
    let users = pg.account_matches(&mut tx, &email).await?;
    if users.len() != 1 {
        return Err(StatusCode::NOT_FOUND);
    }
    let user = &users[0];
    if user.get::<bool, _>("disabled")
        || (role != "owner" && user.get::<String, _>("role") != "user")
    {
        return Err(StatusCode::FORBIDDEN);
    }
    pg.challenge(
        &mut tx,
        &state.oauth_config.key,
        &user.get::<String, _>("email"),
        "reset",
        "user",
        Some(&actor),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(accepted())
}
