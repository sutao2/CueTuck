use crate::{
    admin_risk::{actor_lock, audit},
    bearer_token,
    postgres::Pg,
    require_admin, require_configuration_admin, AppState,
};
use axum::{
    extract::{MatchedPath, Query, Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;
fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
impl Pg {
    pub async fn init_operations(&self) -> Result<(), sqlx::Error> {
        for query in [
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ",self.t("accounts")),
            format!("ALTER TABLE {} ALTER COLUMN created_at SET DEFAULT now()",self.t("accounts")),
            format!("CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY CHECK(id=1),started_at TIMESTAMPTZ NOT NULL DEFAULT now())",self.t("operations_metadata")),
            format!("INSERT INTO {} (id) VALUES (1) ON CONFLICT DO NOTHING",self.t("operations_metadata")),
            format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY,actor_email TEXT,route TEXT NOT NULL,method TEXT NOT NULL,status INT NOT NULL,created_at TIMESTAMPTZ NOT NULL DEFAULT now())",self.t("request_failures")),
        ] {sqlx::query(&query).execute(&self.pool).await?;}
        Ok(())
    }
}
#[derive(Deserialize)]
pub struct OverviewQuery {
    days: Option<i64>,
}
pub async fn overview(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(input): Query<OverviewQuery>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&state, &headers).await?;
    let days = input.days.unwrap_or(7);
    if ![7, 30, 90].contains(&days) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let sql=format!("SELECT json_build_object('days',$1::bigint,'updated_at',now(),'collection_started_at',(SELECT started_at FROM {metadata} WHERE id=1),'accounts',(SELECT count(*) FROM {accounts}),'active_accounts',(SELECT count(*) FROM {accounts} WHERE NOT disabled),'new_accounts',(SELECT count(*) FROM {accounts} WHERE created_at>=now()-make_interval(days=>$1::int)),'unknown_account_dates',(SELECT count(*) FROM {accounts} WHERE created_at IS NULL),'online_content',(SELECT count(*) FROM {items} WHERE visibility='online'),'pending',(SELECT count(*) FROM {publications} WHERE status='pending'),'new_publications',(SELECT count(*) FROM {publications} WHERE created_at>=now()-make_interval(days=>$1::int)),'unknown_publication_dates',(SELECT count(*) FROM {publications} WHERE created_at IS NULL),'recorded_downloads',(SELECT COALESCE(sum(download_count),0) FROM {items}),'favorites',(SELECT count(*) FROM {favorites}),'mock_orders',(SELECT count(*) FROM {orders}),'new_mock_orders',(SELECT count(*) FROM {orders} WHERE created_at>=now()-make_interval(days=>$1::int)),'mock_success',(SELECT count(*) FROM {orders} WHERE outcome='success'),'mock_enabled',$2::boolean)",metadata=pg.t("operations_metadata"),accounts=pg.t("accounts"),items=pg.t("square_items"),publications=pg.t("publications"),favorites=pg.t("favorites"),orders=pg.t("mock_orders"));
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        sqlx::query_scalar::<_, Value>(&sql)
            .bind(days)
            .bind(state.billing_mock)
            .fetch_one(&pg.pool),
    )
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    .map_err(db_error)?;
    Ok(Json(result))
}
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct AuditQuery {
    actor: String,
    action: String,
    from: String,
    to: String,
    offset: i64,
}
impl AuditQuery {
    fn validate(&self) -> Result<(), StatusCode> {
        if self.actor.len() > 254
            || self.action.len() > 100
            || !(0..=100000).contains(&self.offset)
            || [&self.from, &self.to].iter().any(|v| {
                !v.is_empty()
                    && (v.len() != 10 || chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").is_err())
            })
            || (!self.from.is_empty() && !self.to.is_empty() && self.from > self.to)
        {
            Err(StatusCode::BAD_REQUEST)
        } else {
            Ok(())
        }
    }
}
fn safe_details(value: &Value, depth: usize) -> Value {
    if depth > 3 {
        return Value::Null;
    }
    let Some(object) = value.as_object() else {
        return Value::Null;
    };
    let fields = [
        "id",
        "kind",
        "target",
        "children",
        "provider",
        "publication_id",
        "order_id",
        "challenge_id",
        "email",
        "target_email",
        "purpose",
        "role",
        "status",
        "decision",
        "revision",
        "enabled",
        "name",
        "count",
        "uses_per_code",
        "expires_at",
        "mock",
        "outcome",
        "reason",
        "source",
        "method",
        "route",
        "http_status",
        "request_id",
        "square_public_before",
        "square_public_after",
        "publishing_open",
        "registration_open",
        "email_enabled",
        "webhook_enabled",
        "retention_enabled",
        "retention_days",
        "counts",
        "request_failures",
        "notification_deliveries",
        "mail_outbox",
        "oauth_verifications",
        "channel",
        "before",
        "after",
    ];
    let mut out = serde_json::Map::new();
    for (key, value) in object {
        if !fields.contains(&key.as_str()) {
            continue;
        }
        let safe = if value.is_object() {
            safe_details(value, depth + 1)
        } else if let Some(text) = value.as_str() {
            json!(text.chars().take(500).collect::<String>())
        } else if value.is_boolean() || value.is_number() || value.is_null() {
            value.clone()
        } else {
            continue;
        };
        out.insert(key.clone(), safe);
    }
    Value::Object(out)
}
async fn audit_rows(pg: &Pg, input: &AuditQuery, limit: i64) -> Result<Value, StatusCode> {
    input.validate()?;
    let sql=format!("WITH events AS (SELECT id,actor_email,action,target_email,details,created_at,'success' AS result FROM {} UNION ALL SELECT id,actor_email,'request_failed',NULL,jsonb_build_object('method',method,'route',route,'http_status',status,'request_id',id),created_at,'failure' FROM {}),filtered AS (SELECT * FROM events WHERE ($1='' OR strpos(lower(COALESCE(actor_email,'')),lower($1))>0) AND ($2='' OR strpos(action,$2)>0) AND (NULLIF($3,'')::date IS NULL OR created_at>=NULLIF($3,'')::date AT TIME ZONE 'UTC') AND (NULLIF($4,'')::date IS NULL OR created_at<(NULLIF($4,'')::date+1) AT TIME ZONE 'UTC')) SELECT json_build_object('items',COALESCE((SELECT json_agg(p) FROM (SELECT * FROM filtered ORDER BY created_at DESC,id LIMIT $5 OFFSET $6) p),'[]'::json),'total',(SELECT count(*) FROM filtered),'limit',$5::bigint,'offset',$6::bigint)",pg.t("security_audit"),pg.t("request_failures"));
    let mut result: Value = sqlx::query_scalar(&sql)
        .bind(input.actor.trim())
        .bind(input.action.trim())
        .bind(&input.from)
        .bind(&input.to)
        .bind(limit)
        .bind(input.offset)
        .fetch_one(&pg.pool)
        .await
        .map_err(db_error)?;
    for row in result["items"]
        .as_array_mut()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
    {
        row["details"] = safe_details(&row["details"], 0);
    }
    Ok(result)
}
pub async fn audit_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(input): Query<AuditQuery>,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    Ok(Json(
        audit_rows(
            state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?,
            &input,
            25,
        )
        .await?,
    ))
}
pub async fn audit_export(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(mut input): Query<AuditQuery>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_configuration_admin(&state, &headers).await?;
    state.limit_account_auth(&actor).await?;
    input.offset = 0;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let result = audit_rows(pg, &input, 500).await?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    actor_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
        true,
    )
    .await?;
    let owner: bool = sqlx::query_scalar(&format!(
        "SELECT role='owner' FROM {} WHERE email=$1",
        pg.t("accounts")
    ))
    .bind(&actor)
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;
    if !owner {
        return Err(StatusCode::FORBIDDEN);
    }
    audit(pg,&mut tx,&actor,"audit_exported",json!({"count":result["items"].as_array().map_or(0,Vec::len),"from":input.from,"to":input.to})).await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(result))
}
pub async fn failure_log(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let path = request.uri().path();
    let tracked = request.method() != Method::GET
        && request.method() != Method::OPTIONS
        && (path.starts_with("/v1/admin/")
            || path == "/v1/session"
            || path.starts_with("/v1/session/identity/")
            || path == "/v1/publications");
    if !tracked || state.db.is_none() {
        return next.run(request).await;
    }
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|| "unmatched".into());
    let method = request.method().to_string();
    let actor = if let Some(token) = bearer_token(request.headers()) {
        state.email_for_access(&token).await.ok().flatten()
    } else {
        None
    };
    let mut response = next.run(request).await;
    if response.status().is_client_error() || response.status().is_server_error() {
        let id = uuid::Uuid::new_v4().to_string();
        if let Ok(header) = HeaderValue::from_str(&id) {
            response.headers_mut().insert("x-request-id", header);
        }
        let pg = state.db.as_ref().unwrap();
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            sqlx::query(&format!(
                "INSERT INTO {} (id,actor_email,route,method,status) VALUES ($1,$2,$3,$4,$5)",
                pg.t("request_failures")
            ))
            .bind(&id)
            .bind(actor)
            .bind(route)
            .bind(method)
            .bind(i32::from(response.status().as_u16()))
            .execute(&pg.pool),
        )
        .await;
        if !matches!(result, Ok(Ok(_))) {
            eprintln!("request failure audit unavailable; request_id={id}");
        }
    }
    response
}
async fn probe(configured: bool, future: impl std::future::Future<Output = bool>) -> Value {
    if !configured {
        return json!({"status":"not_configured"});
    }
    let start = std::time::Instant::now();
    let ok = tokio::time::timeout(Duration::from_secs(3), future)
        .await
        .unwrap_or(false);
    json!({"status":if ok{"healthy"}else{"unavailable"},"latency_ms":start.elapsed().as_millis()})
}
pub async fn system(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    let (postgres, redis, media) = tokio::join!(
        probe(state.db.is_some(), state.ping_db()),
        probe(state.redis.is_some(), state.ping_redis()),
        probe(state.media.is_some(), async {
            if let Some(media) = &state.media {
                media.ping().await
            } else {
                false
            }
        })
    );
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mail=tokio::time::timeout(Duration::from_secs(2),sqlx::query_scalar::<_,Value>(&format!("SELECT COALESCE(json_object_agg(status,count),'{{}}'::json) FROM (SELECT status,count(*) FROM {} GROUP BY status) q",pg.t("mail_outbox"))).fetch_one(&pg.pool)).await.ok().and_then(Result::ok);
    let notifications=tokio::time::timeout(Duration::from_secs(2),sqlx::query_scalar::<_,Value>(&format!("SELECT COALESCE(json_object_agg(status,count),'{{}}'::json) FROM (SELECT COALESCE(m.status,n.status) AS status,count(*) FROM {} n LEFT JOIN {} m ON n.mail_id=m.id GROUP BY COALESCE(m.status,n.status)) q",pg.t("notification_deliveries"),pg.t("mail_outbox"))).fetch_one(&pg.pool)).await.ok().and_then(Result::ok);
    Ok(Json(
        json!({"version":env!("CARGO_PKG_VERSION"),"checked_at":chrono::Utc::now(),"postgres":postgres,"redis":redis,"media":media,"mail_queue":mail,"notification_queue":notifications,"encryption_key_loaded":true,"mock_billing":state.billing_mock,"recovery":"恢复必须同时验证数据库、加密密钥和对象存储。请按仓库恢复手册在隔离环境演练，本页面不执行服务器命令。"}),
    ))
}
