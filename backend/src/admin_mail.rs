use crate::{
    admin_risk::{actor_lock, audit},
    bearer_token,
    postgres::Pg,
    require_configuration_admin, AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub revision: i64,
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub tls: String,
    pub from: String,
    pub username: String,
    pub encrypted_secret: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            revision: 0,
            enabled: false,
            host: String::new(),
            port: 465,
            tls: "implicit".into(),
            from: String::new(),
            username: String::new(),
            encrypted_secret: String::new(),
        }
    }
}
impl Config {
    fn view(&self) -> Value {
        json!({"revision":self.revision,"enabled":self.enabled,"host":self.host,"port":self.port,"tls":self.tls,"from":self.from,"username":self.username,"secret_configured":!self.encrypted_secret.is_empty()})
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Save {
    revision: i64,
    enabled: bool,
    host: String,
    port: u16,
    tls: String,
    from: String,
    username: String,
    #[serde(default)]
    password: String,
    current_password: String,
}
pub struct Job {
    pub id: String,
    pub claim: String,
    pub payload: String,
    pub recipient: String,
    pub config_revision: i64,
}
impl Pg {
    pub async fn init_mail(&self) -> Result<(), sqlx::Error> {
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY CHECK(id=1),data JSONB NOT NULL)",
            self.t("mail_configuration")
        ))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!(
            "INSERT INTO {} (id,data) VALUES (1,$1) ON CONFLICT DO NOTHING",
            self.t("mail_configuration")
        ))
        .bind(json!(Config::default()))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY,purpose TEXT NOT NULL,recipient TEXT NOT NULL,payload TEXT NOT NULL,actor TEXT NOT NULL,config_revision BIGINT NOT NULL,status TEXT NOT NULL DEFAULT 'queued',attempts INT NOT NULL DEFAULT 0,revision BIGINT NOT NULL DEFAULT 0,claim TEXT,lease_until TIMESTAMPTZ,next_at TIMESTAMPTZ NOT NULL DEFAULT now(),expires_at TIMESTAMPTZ NOT NULL,created_at TIMESTAMPTZ NOT NULL DEFAULT now(),updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),error TEXT)",self.t("mail_outbox"))).execute(&self.pool).await?;
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (id BIGSERIAL PRIMARY KEY,mail_id TEXT NOT NULL,attempt INT NOT NULL,status TEXT NOT NULL,error TEXT,created_at TIMESTAMPTZ NOT NULL DEFAULT now())",self.t("mail_attempts"))).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn mail_config(&self) -> Result<Config, StatusCode> {
        let value: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1",
            self.t("mail_configuration")
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;
        serde_json::from_value(value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
    pub async fn has_mail_secrets(&self) -> Result<bool, StatusCode> {
        sqlx::query_scalar(&format!("SELECT (SELECT data->>'encrypted_secret'<>'' FROM {} WHERE id=1) OR EXISTS(SELECT 1 FROM {} WHERE payload<>'')",self.t("mail_configuration"),self.t("mail_outbox"))).fetch_one(&self.pool).await.map_err(db_error)
    }
    pub async fn queue_mail(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        key: &[u8; 32],
        config_revision: i64,
        purpose: &str,
        recipient: &str,
        subject: &str,
        body: &str,
        actor: &str,
        expires_seconds: i64,
    ) -> Result<String, StatusCode> {
        if !crate::mail_transport::address(recipient)
            || body.len() > 40000
            || subject.len() > 200
            || !(1..=86400).contains(&expires_seconds)
        {
            return Err(StatusCode::BAD_REQUEST);
        }
        let id = uuid::Uuid::new_v4().to_string();
        let payload = crate::oauth_admin::seal(
            key,
            &format!("mail:{id}"),
            &json!({"subject":subject,"body":body}).to_string(),
        )?;
        sqlx::query(&format!("INSERT INTO {} (id,purpose,recipient,payload,actor,config_revision,expires_at) VALUES ($1,$2,$3,$4,$5,$6,now()+$7::bigint*interval '1 second')",self.t("mail_outbox"))).bind(&id).bind(purpose).bind(recipient).bind(payload).bind(actor).bind(config_revision).bind(expires_seconds).execute(&mut **tx).await.map_err(db_error)?;
        Ok(id)
    }
    pub async fn claim_mail(&self, enabled: bool) -> Result<Option<Job>, StatusCode> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        sqlx::query(&format!("UPDATE {} SET status='expired',payload='',claim=NULL,revision=revision+1,updated_at=now() WHERE status IN ('queued','failed','sending') AND expires_at<=now()",self.t("mail_outbox"))).execute(&mut *tx).await.map_err(db_error)?;
        sqlx::query(&format!("UPDATE {} SET status='failed',claim=NULL,error='最后一次发送租约超时，投递结果未知',revision=revision+1,updated_at=now() WHERE status='sending' AND lease_until<now() AND attempts>=3",self.t("mail_outbox"))).execute(&mut *tx).await.map_err(db_error)?;
        let row=sqlx::query(&format!("SELECT id FROM {} WHERE expires_at>now()+interval '25 seconds' AND attempts<3 AND ($1 OR purpose='test') AND ((status IN ('queued','failed') AND next_at<=now()) OR (status='sending' AND lease_until<now())) ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT 1",self.t("mail_outbox"))).bind(enabled).fetch_optional(&mut *tx).await.map_err(db_error)?;
        let Some(row) = row else {
            tx.commit().await.map_err(db_error)?;
            return Ok(None);
        };
        let id: String = row.get("id");
        let claim = uuid::Uuid::new_v4().to_string();
        let row=sqlx::query(&format!("UPDATE {} SET status='sending',claim=$2,lease_until=now()+interval '45 seconds',attempts=attempts+1,revision=revision+1,updated_at=now() WHERE id=$1 RETURNING payload,recipient,config_revision",self.t("mail_outbox"))).bind(&id).bind(&claim).fetch_one(&mut *tx).await.map_err(db_error)?;
        let job = Job {
            id,
            claim,
            payload: row.get("payload"),
            recipient: row.get("recipient"),
            config_revision: row.get("config_revision"),
        };
        sqlx::query(&format!("INSERT INTO {} (mail_id,attempt,status) SELECT id,attempts,'sending' FROM {} WHERE id=$1",self.t("mail_attempts"),self.t("mail_outbox"))).bind(&job.id).execute(&mut *tx).await.map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(Some(job))
    }
    pub async fn complete_mail(
        &self,
        id: &str,
        claim: &str,
        result: Result<(), String>,
    ) -> Result<bool, StatusCode> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        let status = if result.is_ok() { "accepted" } else { "failed" };
        let error = result.err();
        let attempt:Option<i32>=sqlx::query_scalar(&format!("UPDATE {} SET status=$3,error=$4,payload=CASE WHEN $3='accepted' THEN '' ELSE payload END,claim=NULL,lease_until=NULL,next_at=now()+attempts*interval '30 seconds',revision=revision+1,updated_at=now() WHERE id=$1 AND claim=$2 AND status='sending' AND lease_until>now() RETURNING attempts",self.t("mail_outbox"))).bind(id).bind(claim).bind(status).bind(&error).fetch_optional(&mut *tx).await.map_err(db_error)?;
        let Some(attempt) = attempt else {
            return Ok(false);
        };
        sqlx::query(&format!(
            "INSERT INTO {} (mail_id,attempt,status,error) VALUES ($1,$2,$3,$4)",
            self.t("mail_attempts")
        ))
        .bind(id)
        .bind(attempt)
        .bind(status)
        .bind(&error)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(true)
    }
}
async fn locked_owner(
    pg: &Pg,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &str,
    headers: &HeaderMap,
) -> Result<(), StatusCode> {
    actor_lock(
        pg,
        tx,
        actor,
        &bearer_token(headers).ok_or(StatusCode::UNAUTHORIZED)?,
        true,
    )
    .await?;
    let role: String = sqlx::query_scalar(&format!(
        "SELECT role FROM {} WHERE email=$1",
        pg.t("accounts")
    ))
    .bind(actor)
    .fetch_one(&mut **tx)
    .await
    .map_err(db_error)?;
    if role != "owner" {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}
pub async fn get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut view = pg.mail_config().await?.view();
    let test:Option<Value>=sqlx::query_scalar(&format!("SELECT jsonb_build_object('status',status,'error',error,'config_revision',config_revision,'updated_at',updated_at) FROM {} WHERE purpose='test' ORDER BY created_at DESC LIMIT 1",pg.t("mail_outbox"))).fetch_optional(&pg.pool).await.map_err(db_error)?;
    view["last_test"] = json!(test);
    Ok(Json(view))
}
pub async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Save>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_configuration_admin(&state, &headers).await?;
    state.limit_account_auth(&actor).await?;
    if input.revision < 0
        || !crate::mail_transport::server(&input.host, input.port)
        || !crate::mail_transport::address(&input.from)
        || !["implicit", "starttls"].contains(&input.tls.as_str())
        || input.username.trim().is_empty()
        || input.username.len() > 254
        || input.password.len() > 4096
        || input.current_password.len() > 512
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    locked_owner(pg, &mut tx, &actor, &headers).await?;
    let hash: Option<String> = sqlx::query_scalar(&format!(
        "SELECT password_hash FROM {} WHERE email=$1",
        pg.t("accounts")
    ))
    .bind(&actor)
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;
    if !hash.is_some_and(|h| crate::verify_password(&input.current_password, &h)) {
        return Err(StatusCode::FORBIDDEN);
    }
    let old: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} WHERE id=1 FOR UPDATE",
        pg.t("mail_configuration")
    ))
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;
    let old: Config = serde_json::from_value(old).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if old.revision != input.revision {
        return Err(StatusCode::CONFLICT);
    }
    let encrypted_secret = if input.password.is_empty() {
        old.encrypted_secret
    } else {
        crate::oauth_admin::seal(&state.oauth_config.key, "smtp", &input.password)?
    };
    if input.enabled && encrypted_secret.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let config = Config {
        revision: old.revision + 1,
        enabled: input.enabled,
        host: input.host,
        port: input.port,
        tls: input.tls,
        from: input.from,
        username: input.username,
        encrypted_secret,
    };
    sqlx::query(&format!(
        "UPDATE {} SET data=$1 WHERE id=1",
        pg.t("mail_configuration")
    ))
    .bind(json!(config))
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    audit(pg,&mut tx,&actor,"mail_configuration_saved",json!({"revision":config.revision,"enabled":config.enabled,"host":config.host,"port":config.port,"tls":config.tls,"from":config.from,"secret_rotated":!input.password.is_empty()})).await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(config.view()))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Test {
    revision: i64,
    to: String,
}
pub async fn test(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Test>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let actor = require_configuration_admin(&state, &headers).await?;
    state.limit_account_auth(&actor).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    locked_owner(pg, &mut tx, &actor, &headers).await?;
    let value: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} WHERE id=1 FOR SHARE",
        pg.t("mail_configuration")
    ))
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;
    let config: Config =
        serde_json::from_value(value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if config.revision != input.revision {
        return Err(StatusCode::CONFLICT);
    }
    if config.encrypted_secret.is_empty() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let id=pg.queue_mail(&mut tx,&state.oauth_config.key,config.revision,"test",&input.to,"PromptArk 邮件服务测试","这是一封由站点所有者主动发起的测试邮件。收到本邮件表示当前 SMTP 已完成这次投递。无需回复。",&actor,3600).await?;
    audit(
        pg,
        &mut tx,
        &actor,
        "mail_test_requested",
        json!({"mail_id":id,"recipient":input.to,"revision":config.revision}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id":id,"status":"queued"})),
    ))
}
#[derive(Deserialize, Default)]
pub struct Filter {
    #[serde(default)]
    status: String,
    #[serde(default)]
    q: String,
    #[serde(default)]
    offset: i64,
}
pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(filter): Query<Filter>,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    if filter.q.len() > 254
        || !(0..=100000).contains(&filter.offset)
        || ![
            "",
            "queued",
            "sending",
            "accepted",
            "failed",
            "expired",
            "config_changed",
        ]
        .contains(&filter.status.as_str())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let clause="($1='' OR status=$1) AND ($2='' OR strpos(lower(recipient),lower($2))>0 OR strpos(id,$2)>0)";
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {} WHERE {clause}",
        pg.t("mail_outbox")
    ))
    .bind(&filter.status)
    .bind(&filter.q)
    .fetch_one(&pg.pool)
    .await
    .map_err(db_error)?;
    let items:Vec<Value>=sqlx::query_scalar(&format!("SELECT to_jsonb(m)-'payload'-'claim'-'lease_until' FROM {} m WHERE {clause} ORDER BY created_at DESC,id LIMIT 25 OFFSET $3",pg.t("mail_outbox"))).bind(&filter.status).bind(&filter.q).bind(filter.offset).fetch_all(&pg.pool).await.map_err(db_error)?;
    Ok(Json(
        json!({"items":items,"total":total,"offset":filter.offset,"limit":25}),
    ))
}
pub async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let item: Value = sqlx::query_scalar(&format!(
        "SELECT to_jsonb(m)-'payload'-'claim'-'lease_until' FROM {} m WHERE id=$1",
        pg.t("mail_outbox")
    ))
    .bind(&id)
    .fetch_optional(&pg.pool)
    .await
    .map_err(db_error)?
    .ok_or(StatusCode::NOT_FOUND)?;
    let attempts: Vec<Value> = sqlx::query_scalar(&format!(
        "SELECT to_jsonb(a) FROM {} a WHERE mail_id=$1 ORDER BY id",
        pg.t("mail_attempts")
    ))
    .bind(&id)
    .fetch_all(&pg.pool)
    .await
    .map_err(db_error)?;
    Ok(Json(json!({"item":item,"attempts":attempts})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retry {
    revision: i64,
}
pub async fn retry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Retry>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_configuration_admin(&state, &headers).await?;
    state.limit_account_auth(&actor).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    locked_owner(pg, &mut tx, &actor, &headers).await?;
    let changed=sqlx::query(&format!("UPDATE {} SET status='queued',next_at=now(),revision=revision+1,updated_at=now() WHERE id=$1 AND revision=$2 AND status='failed' AND attempts<3 AND expires_at>now()",pg.t("mail_outbox"))).bind(&id).bind(input.revision).execute(&mut *tx).await.map_err(db_error)?;
    if changed.rows_affected() != 1 {
        return Err(StatusCode::CONFLICT);
    }
    audit(
        pg,
        &mut tx,
        &actor,
        "mail_retry_requested",
        json!({"mail_id":id,"revision":input.revision+1}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(json!({"id":id,"status":"queued"})))
}
pub async fn deliver_one(state: &AppState) -> Result<bool, StatusCode> {
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let config = pg.mail_config().await?;
    let Some(job) = pg.claim_mail(config.enabled).await? else {
        return Ok(false);
    };
    if job.config_revision != config.revision {
        sqlx::query(&format!("UPDATE {} SET status='config_changed',error='SMTP 配置已变化，请重新发起',payload='',claim=NULL,revision=revision+1,updated_at=now() WHERE id=$1 AND claim=$2",pg.t("mail_outbox"))).bind(&job.id).bind(&job.claim).execute(&pg.pool).await.map_err(db_error)?;
        return Ok(true);
    }
    let result = async {
        let secret =
            crate::oauth_admin::unseal(&state.oauth_config.key, "smtp", &config.encrypted_secret)
                .map_err(|_| "SMTP 密钥无法解密".to_owned())?;
        let raw = crate::oauth_admin::unseal(
            &state.oauth_config.key,
            &format!("mail:{}", job.id),
            &job.payload,
        )
        .map_err(|_| "邮件载荷无法解密".to_owned())?;
        let value: Value = serde_json::from_str(&raw).map_err(|_| "邮件载荷无效".to_owned())?;
        crate::mail_transport::send(
            &config.host,
            config.port,
            &config.tls,
            &config.username,
            &secret,
            &config.from,
            &job.recipient,
            value["subject"].as_str().ok_or("缺少邮件主题")?,
            value["body"].as_str().ok_or("缺少邮件正文")?,
            &job.id,
        )
        .await
    }
    .await;
    pg.complete_mail(&job.id, &job.claim, result).await?;
    Ok(true)
}
impl AppState {
    pub fn start_mail_worker(&self) {
        let state = self.clone();
        tokio::spawn(async move {
            loop {
                let did = deliver_one(&state).await.unwrap_or(false);
                tokio::time::sleep(std::time::Duration::from_secs(if did { 1 } else { 5 })).await;
            }
        });
    }
}
