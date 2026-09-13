use crate::{
    admin_risk::audit, bearer_token, oauth_verification::owner_lock, postgres::Pg,
    require_configuration_admin, AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Postgres, Transaction};
fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub revision: i64,
    pub email_enabled: bool,
    pub recipient: String,
    pub webhook_enabled: bool,
    pub encrypted_endpoint: String,
    pub encrypted_secret: String,
    pub threshold: u8,
    pub daily_limit: i64,
    pub active_since: Option<String>,
    pub retention_enabled: bool,
    pub retention_days: i64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            revision: 0,
            email_enabled: false,
            recipient: String::new(),
            webhook_enabled: false,
            encrypted_endpoint: String::new(),
            encrypted_secret: String::new(),
            threshold: 70,
            daily_limit: 50,
            active_since: None,
            retention_enabled: false,
            retention_days: 90,
        }
    }
}
impl Config {
    fn view(&self, key: &[u8; 32]) -> Result<Value, StatusCode> {
        let endpoint =
            crate::oauth_admin::unseal(key, "notification:endpoint", &self.encrypted_endpoint)?;
        let host = url::Url::parse(&endpoint)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned));
        Ok(
            json!({"revision":self.revision,"email_enabled":self.email_enabled,"recipient":self.recipient,"webhook_enabled":self.webhook_enabled,"endpoint_configured":!endpoint.is_empty(),"webhook_host":host,"secret_configured":!self.encrypted_secret.is_empty(),"threshold":self.threshold,"daily_limit":self.daily_limit,"active_since":self.active_since,"retention_enabled":self.retention_enabled,"retention_days":self.retention_days}),
        )
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Save {
    revision: i64,
    current_password: String,
    email_enabled: bool,
    recipient: String,
    webhook_enabled: bool,
    #[serde(default)]
    endpoint: String,
    #[serde(default)]
    secret: String,
    threshold: u8,
    daily_limit: i64,
    retention_enabled: bool,
    retention_days: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Test {
    revision: i64,
    channel: String,
}
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Filter {
    status: String,
    offset: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retry {
    revision: i64,
}
impl Pg {
    pub async fn init_notifications(&self) -> Result<(), sqlx::Error> {
        for sql in [
            format!("CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY CHECK(id=1),data JSONB NOT NULL)",self.t("notification_configuration")),
            format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY,event_id TEXT NOT NULL,channel TEXT NOT NULL,metadata JSONB NOT NULL,config_revision BIGINT NOT NULL,mail_id TEXT,status TEXT NOT NULL,attempts INT NOT NULL DEFAULT 0,revision BIGINT NOT NULL DEFAULT 0,claim TEXT,lease_until TIMESTAMPTZ,next_at TIMESTAMPTZ NOT NULL DEFAULT now(),expires_at TIMESTAMPTZ NOT NULL DEFAULT now()+interval '24 hours',created_at TIMESTAMPTZ NOT NULL DEFAULT now(),updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),error TEXT,UNIQUE(event_id,channel))",self.t("notification_deliveries")),
            format!("CREATE TABLE IF NOT EXISTS {} (id BIGSERIAL PRIMARY KEY,delivery_id TEXT NOT NULL,attempt INT NOT NULL,status TEXT NOT NULL,error TEXT,created_at TIMESTAMPTZ NOT NULL DEFAULT now())",self.t("notification_attempts")),
        ]{sqlx::query(&sql).execute(&self.pool).await?;}
        sqlx::query(&format!(
            "INSERT INTO {} (id,data) VALUES (1,$1) ON CONFLICT DO NOTHING",
            self.t("notification_configuration")
        ))
        .bind(json!(Config::default()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn notification_config(&self) -> Result<Config, StatusCode> {
        let value: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1",
            self.t("notification_configuration")
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;
        serde_json::from_value(value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
    pub async fn has_notification_secrets(&self) -> Result<bool, StatusCode> {
        sqlx::query_scalar(&format!("SELECT data->>'encrypted_endpoint'<>'' OR data->>'encrypted_secret'<>'' FROM {} WHERE id=1",self.t("notification_configuration"))).fetch_one(&self.pool).await.map_err(db_error)
    }
}
async fn locked_config(pg: &Pg, tx: &mut Transaction<'_, Postgres>) -> Result<Config, StatusCode> {
    let value: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} WHERE id=1 FOR UPDATE",
        pg.t("notification_configuration")
    ))
    .fetch_one(&mut **tx)
    .await
    .map_err(db_error)?;
    serde_json::from_value(value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
pub async fn get_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    Ok(Json(
        state
            .db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
            .notification_config()
            .await?
            .view(&state.oauth_config.key)?,
    ))
}
pub async fn save_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Save>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_configuration_admin(&state, &headers).await?;
    state.limit_account_auth(&actor).await?;
    if input.current_password.len() > 512
        || !(1..=100).contains(&input.threshold)
        || !(1..=1000).contains(&input.daily_limit)
        || !(7..=3650).contains(&input.retention_days)
        || input.secret.len() > 4096
        || (!input.secret.is_empty() && input.secret.len() < 16)
        || (!input.recipient.is_empty() && !crate::mail_transport::address(input.recipient.trim()))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !input.endpoint.is_empty() {
        crate::outbound::validate_url(input.endpoint.trim())
            .map_err(|_| StatusCode::BAD_REQUEST)?;
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    owner_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
    )
    .await?;
    pg.verify_login(&actor, &input.current_password)
        .await
        .map_err(|_| StatusCode::FORBIDDEN)?;
    let previous = locked_config(pg, &mut tx).await?;
    if input.revision != previous.revision {
        return Err(StatusCode::CONFLICT);
    }
    let config = Config {
        revision: previous.revision + 1,
        email_enabled: input.email_enabled,
        recipient: input.recipient.trim().into(),
        webhook_enabled: input.webhook_enabled,
        encrypted_endpoint: if input.endpoint.is_empty() {
            previous.encrypted_endpoint
        } else {
            crate::oauth_admin::seal(
                &state.oauth_config.key,
                "notification:endpoint",
                input.endpoint.trim(),
            )?
        },
        encrypted_secret: if input.secret.is_empty() {
            previous.encrypted_secret
        } else {
            crate::oauth_admin::seal(
                &state.oauth_config.key,
                "notification:secret",
                &input.secret,
            )?
        },
        threshold: input.threshold,
        daily_limit: input.daily_limit,
        active_since: if input.email_enabled || input.webhook_enabled {
            previous
                .active_since
                .or_else(|| Some(chrono::Utc::now().to_rfc3339()))
        } else {
            None
        },
        retention_enabled: input.retention_enabled,
        retention_days: input.retention_days,
    };
    if config.email_enabled {
        let mail = pg.mail_config().await?;
        if config.recipient.is_empty() || !mail.enabled || mail.encrypted_secret.is_empty() {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    }
    if config.webhook_enabled
        && (config.encrypted_endpoint.is_empty() || config.encrypted_secret.is_empty())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    sqlx::query(&format!(
        "UPDATE {} SET data=$1 WHERE id=1",
        pg.t("notification_configuration")
    ))
    .bind(json!(config))
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    sqlx::query(&format!("UPDATE {} SET status='config_changed',payload='',claim=NULL,revision=revision+1,updated_at=now() WHERE id IN (SELECT mail_id FROM {}) AND status IN ('queued','failed','sending')",pg.t("mail_outbox"),pg.t("notification_deliveries"))).execute(&mut *tx).await.map_err(db_error)?;
    sqlx::query(&format!("UPDATE {} SET status='config_changed',claim=NULL,error='通知配置已变化，请重新发起测试',revision=revision+1,updated_at=now() WHERE status IN ('queued','failed','sending','mail_queued')",pg.t("notification_deliveries"))).execute(&mut *tx).await.map_err(db_error)?;
    audit(pg,&mut tx,&actor,"notification_configuration_saved",json!({"revision":config.revision,"email_enabled":config.email_enabled,"webhook_enabled":config.webhook_enabled,"retention_enabled":config.retention_enabled,"retention_days":config.retention_days})).await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(config.view(&state.oauth_config.key)?))
}
async fn enqueue(
    state: &AppState,
    tx: &mut Transaction<'_, Postgres>,
    config: &Config,
    event_id: &str,
    channel: &str,
    metadata: Value,
    actor: &str,
) -> Result<Value, StatusCode> {
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let exists: bool = sqlx::query_scalar(&format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE event_id=$1 AND channel=$2)",
        pg.t("notification_deliveries")
    ))
    .bind(event_id)
    .bind(channel)
    .fetch_one(&mut **tx)
    .await
    .map_err(db_error)?;
    if exists {
        return Ok(json!({"duplicate":true}));
    }
    let count:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {} WHERE created_at>=date_trunc('day',now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC' AND status<>'quota_limited'",pg.t("notification_deliveries"))).fetch_one(&mut **tx).await.map_err(db_error)?;
    let id = uuid::Uuid::new_v4().to_string();
    let mut status = if count >= config.daily_limit {
        "quota_limited"
    } else {
        "queued"
    };
    let mut mail_id = None;
    if channel == "email" && status == "queued" {
        let mail = pg.mail_config().await?;
        if !mail.enabled || mail.encrypted_secret.is_empty() {
            status = "unavailable"
        } else {
            mail_id = Some(
                pg.queue_mail(
                    tx,
                    &state.oauth_config.key,
                    mail.revision,
                    "risk_notification",
                    &config.recipient,
                    "CueTuck 风险通知",
                    &format!(
                        "公开投稿风险事件（不含正文）。\n{}\n请登录管理台核查。",
                        metadata
                    ),
                    actor,
                    86400,
                )
                .await?,
            );
            status = "mail_queued"
        }
    }
    sqlx::query(&format!("INSERT INTO {} (id,event_id,channel,metadata,config_revision,mail_id,status) VALUES ($1,$2,$3,$4,$5,$6,$7)",pg.t("notification_deliveries"))).bind(&id).bind(event_id).bind(channel).bind(&metadata).bind(config.revision).bind(mail_id).bind(status).execute(&mut **tx).await.map_err(db_error)?;
    Ok(json!({"id":id,"status":status}))
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
    owner_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
    )
    .await?;
    let config = locked_config(pg, &mut tx).await?;
    if config.revision != input.revision {
        return Err(StatusCode::CONFLICT);
    }
    if !matches!(input.channel.as_str(), "email" | "webhook")
        || !(input.channel == "email" && config.email_enabled
            || input.channel == "webhook" && config.webhook_enabled)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let event_id = uuid::Uuid::new_v4().to_string();
    let result=enqueue(&state,&mut tx,&config,&event_id,&input.channel,json!({"event_id":event_id,"source":"explicit_test","test":true,"created_at":chrono::Utc::now()}),&actor).await?;
    audit(
        pg,
        &mut tx,
        &actor,
        "notification_test_requested",
        json!({"id":result["id"],"channel":input.channel,"status":result["status"]}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok((StatusCode::ACCEPTED, Json(result)))
}
pub async fn collect(state: &AppState) -> Result<usize, StatusCode> {
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    let config = locked_config(pg, &mut tx).await?;
    if !config.email_enabled && !config.webhook_enabled {
        return Ok(0);
    }
    let events:Vec<Value>=sqlx::query_scalar(&format!("SELECT json_build_object('event_id',a.id,'source',a.action,'publication_id',a.details->>'publication_id','score',COALESCE(a.details->'score',a.details->'result'->'score'),'created_at',a.created_at,'test',false) FROM {} a WHERE a.created_at >= $1::timestamptz AND a.action IN ('publication_auto_screened','publication_ai_screened') AND COALESCE(a.details->>'score',a.details->'result'->>'score')::int >= $2 AND (($3 AND NOT EXISTS(SELECT 1 FROM {1} n WHERE n.event_id=a.id AND n.channel='email')) OR ($4 AND NOT EXISTS(SELECT 1 FROM {1} n WHERE n.event_id=a.id AND n.channel='webhook'))) ORDER BY a.created_at,a.id LIMIT 20",pg.t("security_audit"),pg.t("notification_deliveries"))).bind(&config.active_since).bind(i32::from(config.threshold)).bind(config.email_enabled).bind(config.webhook_enabled).fetch_all(&mut *tx).await.map_err(db_error)?;
    for event in &events {
        for (channel, enabled) in [
            ("email", config.email_enabled),
            ("webhook", config.webhook_enabled),
        ] {
            if enabled {
                enqueue(
                    state,
                    &mut tx,
                    &config,
                    event["event_id"]
                        .as_str()
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?,
                    channel,
                    event.clone(),
                    "system:notifications",
                )
                .await?;
            }
        }
    }
    tx.commit().await.map_err(db_error)?;
    Ok(events.len())
}
pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(filter): Query<Filter>,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    if !(0..=100000).contains(&filter.offset) || filter.status.len() > 40 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let sql=format!("WITH rows AS (SELECT n.id,n.event_id,n.channel,n.metadata,n.created_at,n.expires_at,COALESCE(m.status,n.status) AS status,COALESCE(m.attempts,n.attempts) AS attempts,COALESCE(m.revision,n.revision) AS revision,COALESCE(m.error,n.error) AS error,n.mail_id FROM {} n LEFT JOIN {} m ON n.mail_id=m.id),filtered AS (SELECT * FROM rows WHERE $1='' OR status=$1) SELECT json_build_object('items',COALESCE((SELECT json_agg(p) FROM (SELECT * FROM filtered ORDER BY created_at DESC,id LIMIT 25 OFFSET $2) p),'[]'::json),'total',(SELECT count(*) FROM filtered),'offset',$2::bigint,'limit',25)",pg.t("notification_deliveries"),pg.t("mail_outbox"));
    Ok(Json(
        sqlx::query_scalar(&sql)
            .bind(&filter.status)
            .bind(filter.offset)
            .fetch_one(&pg.pool)
            .await
            .map_err(db_error)?,
    ))
}

pub async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let row:Value=sqlx::query_scalar(&format!("SELECT json_build_object('id',n.id,'channel',n.channel,'metadata',n.metadata,'status',COALESCE(m.status,n.status),'attempts',COALESCE(m.attempts,n.attempts),'error',COALESCE(m.error,n.error),'mail_id',n.mail_id) FROM {} n LEFT JOIN {} m ON m.id=n.mail_id WHERE n.id=$1",pg.t("notification_deliveries"),pg.t("mail_outbox"))).bind(&id).fetch_optional(&pg.pool).await.map_err(db_error)?.ok_or(StatusCode::NOT_FOUND)?;
    let mail = row["mail_id"].as_str();
    let (table, key, value) = if let Some(mail) = mail {
        ("mail_attempts", "mail_id", mail)
    } else {
        ("notification_attempts", "delivery_id", id.as_str())
    };
    let attempts:Vec<Value>=sqlx::query_scalar(&format!("SELECT json_build_object('attempt',attempt,'status',status,'error',error,'created_at',created_at) FROM {} WHERE {key}=$1 ORDER BY id",pg.t(table))).bind(value).fetch_all(&pg.pool).await.map_err(db_error)?;
    Ok(Json(json!({"delivery":row,"history":attempts})))
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
    owner_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
    )
    .await?;
    let config = locked_config(pg, &mut tx).await?;
    let row: Option<(Option<String>, i64)> = sqlx::query_as(&format!(
        "SELECT mail_id,config_revision FROM {} WHERE id=$1",
        pg.t("notification_deliveries")
    ))
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_error)?;
    let (mail_id, revision) = row.ok_or(StatusCode::NOT_FOUND)?;
    if revision != config.revision {
        return Err(StatusCode::CONFLICT);
    }
    let (table, target) = if let Some(mail_id) = mail_id {
        if !config.email_enabled {
            return Err(StatusCode::CONFLICT);
        }
        ("mail_outbox", mail_id)
    } else {
        if !config.webhook_enabled {
            return Err(StatusCode::CONFLICT);
        }
        ("notification_deliveries", id.clone())
    };
    let changed=sqlx::query(&format!("UPDATE {} SET status='queued',next_at=now(),revision=revision+1,updated_at=now() WHERE id=$1 AND revision=$2 AND status='failed' AND attempts<3 AND expires_at>now()+interval '25 seconds'",pg.t(table))).bind(&target).bind(input.revision).execute(&mut *tx).await.map_err(db_error)?.rows_affected();
    if changed != 1 {
        return Err(StatusCode::CONFLICT);
    }
    audit(
        pg,
        &mut tx,
        &actor,
        "notification_retry_requested",
        json!({"id":id,"revision":input.revision}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(json!({"id":id,"status":"queued"})))
}
impl Pg {
    pub(crate) async fn claim_notification(&self) -> Result<Option<Value>, StatusCode> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        sqlx::query(&format!("UPDATE {} SET status='expired',claim=NULL,revision=revision+1,updated_at=now() WHERE channel='webhook' AND status IN ('queued','failed','sending') AND expires_at<=now()",self.t("notification_deliveries"))).execute(&mut *tx).await.map_err(db_error)?;
        sqlx::query(&format!("UPDATE {} SET status='failed',claim=NULL,error='租约超时，结果未知且重试次数已耗尽',revision=revision+1,updated_at=now() WHERE channel='webhook' AND status='sending' AND lease_until<now() AND attempts>=3",self.t("notification_deliveries"))).execute(&mut *tx).await.map_err(db_error)?;
        let id:Option<String>=sqlx::query_scalar(&format!("SELECT id FROM {} WHERE channel='webhook' AND attempts<3 AND expires_at>now()+interval '25 seconds' AND ((status IN ('queued','failed') AND next_at<=now()) OR (status='sending' AND lease_until<now())) ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT 1",self.t("notification_deliveries"))).fetch_optional(&mut *tx).await.map_err(db_error)?;
        let Some(id) = id else {
            tx.commit().await.map_err(db_error)?;
            return Ok(None);
        };
        let claim = uuid::Uuid::new_v4().to_string();
        let job:Value=sqlx::query_scalar(&format!("UPDATE {} SET status='sending',claim=$2,lease_until=now()+interval '45 seconds',attempts=attempts+1,revision=revision+1,updated_at=now() WHERE id=$1 RETURNING json_build_object('id',id,'claim',claim,'metadata',metadata,'config_revision',config_revision)",self.t("notification_deliveries"))).bind(&id).bind(&claim).fetch_one(&mut *tx).await.map_err(db_error)?;
        sqlx::query(&format!("INSERT INTO {} (delivery_id,attempt,status) SELECT id,attempts,'sending' FROM {} WHERE id=$1",self.t("notification_attempts"),self.t("notification_deliveries"))).bind(&id).execute(&mut *tx).await.map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(Some(job))
    }
    pub(crate) async fn complete_notification(
        &self,
        id: &str,
        claim: &str,
        result: Result<(), String>,
    ) -> Result<bool, StatusCode> {
        let status = if result.is_ok() { "accepted" } else { "failed" };
        let error = result.err();
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        let attempt:Option<i32>=sqlx::query_scalar(&format!("UPDATE {} SET status=$3,error=$4,claim=NULL,lease_until=NULL,next_at=now()+attempts*interval '30 seconds',revision=revision+1,updated_at=now() WHERE id=$1 AND claim=$2 AND status='sending' AND lease_until>now() RETURNING attempts",self.t("notification_deliveries"))).bind(id).bind(claim).bind(status).bind(&error).fetch_optional(&mut *tx).await.map_err(db_error)?;
        let Some(attempt) = attempt else {
            return Ok(false);
        };
        sqlx::query(&format!(
            "INSERT INTO {} (delivery_id,attempt,status,error) VALUES ($1,$2,$3,$4)",
            self.t("notification_attempts")
        ))
        .bind(id)
        .bind(attempt)
        .bind(status)
        .bind(error)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(true)
    }
}
pub(crate) fn signature(secret: &str, body: &str) -> String {
    use hmac::{Hmac, Mac};
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(body.as_bytes());
    format!("sha256={:x}", mac.finalize().into_bytes())
}
pub(crate) async fn send_webhook(
    client: &reqwest::Client,
    endpoint: &str,
    secret: &str,
    id: &str,
    body: &str,
) -> Result<(), String> {
    let response = client
        .post(endpoint)
        .header("content-type", "application/json")
        .header("X-PromptArk-Delivery", id)
        .header("X-PromptArk-Signature", signature(secret, body))
        .body(body.to_owned())
        .send()
        .await
        .map_err(|_| "Webhook 连接失败或超时，结果可能未知".to_owned())?;
    if !response.status().is_success() {
        return Err(format!("Webhook 返回 HTTP {}", response.status().as_u16()));
    }
    Ok(())
}
pub async fn deliver_one(state: &AppState) -> Result<bool, StatusCode> {
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let Some(job) = pg.claim_notification().await? else {
        return Ok(false);
    };
    let config = pg.notification_config().await?;
    let id = job["id"]
        .as_str()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let claim = job["claim"]
        .as_str()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    if !config.webhook_enabled || job["config_revision"] != config.revision {
        sqlx::query(&format!("UPDATE {} SET status='config_changed',claim=NULL,error='通知配置已变化',revision=revision+1,updated_at=now() WHERE id=$1 AND claim=$2",pg.t("notification_deliveries"))).bind(id).bind(claim).execute(&pg.pool).await.map_err(db_error)?;
        return Ok(true);
    }
    let result = async {
        let endpoint = crate::oauth_admin::unseal(
            &state.oauth_config.key,
            "notification:endpoint",
            &config.encrypted_endpoint,
        )
        .map_err(|_| "Webhook 地址无法解密".to_owned())?;
        let secret = crate::oauth_admin::unseal(
            &state.oauth_config.key,
            "notification:secret",
            &config.encrypted_secret,
        )
        .map_err(|_| "Webhook 签名密钥无法解密".to_owned())?;
        let client = crate::outbound::client(&endpoint, 12).await?;
        send_webhook(
            &client,
            &endpoint,
            &secret,
            id,
            &job["metadata"].to_string(),
        )
        .await
    }
    .await;
    pg.complete_notification(id, claim, result).await?;
    Ok(true)
}
pub async fn cleanup(state: &AppState) -> Result<Value, StatusCode> {
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    let config = locked_config(pg, &mut tx).await?;
    if !config.retention_enabled {
        return Ok(json!({"enabled":false}));
    }
    let mut counts = serde_json::Map::new();
    // Mail is the source of truth; preserve its terminal status before pruning mail logs.
    sqlx::query(&format!("UPDATE {} n SET status=m.status,attempts=m.attempts,error=m.error FROM {} m WHERE n.mail_id=m.id AND m.status NOT IN ('queued','sending') AND (m.status<>'failed' OR m.attempts>=3 OR m.expires_at<=now())",pg.t("notification_deliveries"),pg.t("mail_outbox"))).execute(&mut *tx).await.map_err(db_error)?;
    for (table,predicate,attempts,fk) in [
        ("request_failures","TRUE",None,""),
        ("notification_deliveries","status NOT IN ('queued','sending','mail_queued') AND (status<>'failed' OR attempts>=3 OR expires_at<=now())",Some("notification_attempts"),"delivery_id"),
        ("mail_outbox","status NOT IN ('queued','sending') AND (status<>'failed' OR attempts>=3 OR expires_at<=now())",Some("mail_attempts"),"mail_id"),
        ("oauth_verifications","status NOT IN ('pending','verifying') OR expires_at<=now()",None,""),
    ] {
        let ids:Vec<String>=sqlx::query_scalar(&format!("SELECT id FROM {} WHERE created_at<now()-$1::bigint*interval '1 day' AND ({predicate}) ORDER BY created_at LIMIT 500 FOR UPDATE SKIP LOCKED",pg.t(table))).bind(config.retention_days).fetch_all(&mut *tx).await.map_err(db_error)?;
        if let Some(attempts)=attempts{sqlx::query(&format!("DELETE FROM {} WHERE {fk}=ANY($1)",pg.t(attempts))).bind(&ids).execute(&mut *tx).await.map_err(db_error)?;}
        let count=sqlx::query(&format!("DELETE FROM {} WHERE id=ANY($1)",pg.t(table))).bind(&ids).execute(&mut *tx).await.map_err(db_error)?.rows_affected();counts.insert(table.into(),json!(count));
    }
    if counts.values().any(|v| v.as_u64().unwrap_or(0) > 0) {
        audit(
            pg,
            &mut tx,
            "system:retention",
            "runtime_logs_pruned",
            json!({"retention_days":config.retention_days,"counts":counts}),
        )
        .await?;
    }
    tx.commit().await.map_err(db_error)?;
    Ok(json!({"enabled":true,"counts":counts}))
}
impl AppState {
    pub fn start_notification_worker(&self) {
        let state = self.clone();
        tokio::spawn(async move {
            let mut ticks = 0u32;
            loop {
                let _ = collect(&state).await;
                let _ = deliver_one(&state).await;
                ticks += 1;
                if ticks % 12 == 0 {
                    let _ = cleanup(&state).await;
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }
}
