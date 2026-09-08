use crate::{
    admin_risk::{actor_lock, audit},
    bearer_token,
    postgres::Pg,
    require_admin, require_user, AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
fn request_id(value: &str) -> Result<(), StatusCode> {
    uuid::Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| StatusCode::BAD_REQUEST)
}
fn reason(value: &str) -> bool {
    !value.trim().is_empty() && value.chars().count() <= 500
}
impl Pg {
    pub async fn init_mock_billing(&self) -> Result<(), sqlx::Error> {
        for sql in [
            format!("CREATE TABLE IF NOT EXISTS {} (email TEXT PRIMARY KEY,pro BOOLEAN NOT NULL DEFAULT false,revision BIGINT NOT NULL DEFAULT 0)",self.t("mock_entitlements")),
            format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY,request_id TEXT NOT NULL,actor TEXT NOT NULL,email TEXT NOT NULL,outcome TEXT NOT NULL,reason TEXT NOT NULL,code_id TEXT,created_at TIMESTAMPTZ NOT NULL DEFAULT now(),UNIQUE(actor,request_id))",self.t("mock_orders")),
            format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY,name TEXT NOT NULL,actor TEXT NOT NULL,request_id TEXT NOT NULL UNIQUE,enabled BOOLEAN NOT NULL DEFAULT true,revision BIGINT NOT NULL DEFAULT 0,expires_at TIMESTAMPTZ NOT NULL,uses_per_code INT NOT NULL,created_at TIMESTAMPTZ NOT NULL DEFAULT now())",self.t("mock_code_batches")),
            format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY,batch_id TEXT NOT NULL,code_hash TEXT NOT NULL UNIQUE,hint TEXT NOT NULL)",self.t("mock_codes")),
            format!("CREATE TABLE IF NOT EXISTS {} (code_id TEXT NOT NULL,email TEXT NOT NULL,order_id TEXT NOT NULL,created_at TIMESTAMPTZ NOT NULL DEFAULT now(),PRIMARY KEY(code_id,email))",self.t("mock_code_uses")),
        ] {sqlx::query(&sql).execute(&self.pool).await?;}
        Ok(())
    }
    async fn mock_lock(&self, tx: &mut Transaction<'_, Postgres>) -> Result<(), StatusCode> {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(format!("{}:mock-billing", self.schema))
            .execute(&mut **tx)
            .await
            .map_err(db_error)?;
        Ok(())
    }
    pub async fn mock_pro(&self, email: &str) -> Result<bool, StatusCode> {
        Ok(sqlx::query_scalar::<_, bool>(&format!(
            "SELECT pro FROM {} WHERE email=$1",
            self.t("mock_entitlements")
        ))
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?
        .unwrap_or(false))
    }
    async fn mock_order(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        actor: &str,
        email: &str,
        outcome: &str,
        reason: &str,
        request: &str,
        code_id: Option<&str>,
    ) -> Result<Value, StatusCode> {
        request_id(request)?;
        if let Some(row) = sqlx::query_scalar::<_, Value>(&format!(
            "SELECT to_jsonb(o) FROM {} o WHERE actor=$1 AND request_id=$2",
            self.t("mock_orders")
        ))
        .bind(actor)
        .bind(request)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db_error)?
        {
            if row["email"] != email
                || row["outcome"] != outcome
                || row["reason"] != reason
                || row["code_id"] != json!(code_id)
            {
                return Err(StatusCode::CONFLICT);
            }
            return Ok(row);
        }
        let active: Option<bool> = sqlx::query_scalar(&format!(
            "SELECT NOT disabled FROM {} WHERE email=$1 FOR SHARE",
            self.t("accounts")
        ))
        .bind(email)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db_error)?;
        if active != Some(true) {
            return Err(if active.is_none() {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::FORBIDDEN
            });
        }
        if matches!(outcome, "success" | "reset" | "redeem") {
            sqlx::query(&format!("INSERT INTO {} AS e (email,pro,revision) VALUES ($1,$2,1) ON CONFLICT(email) DO UPDATE SET pro=EXCLUDED.pro,revision=e.revision+1",self.t("mock_entitlements"))).bind(email).bind(outcome!="reset").execute(&mut **tx).await.map_err(db_error)?;
        }
        let id = uuid::Uuid::new_v4().to_string();
        let row:Value=sqlx::query_scalar(&format!("INSERT INTO {} AS o (id,request_id,actor,email,outcome,reason,code_id) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING to_jsonb(o)",self.t("mock_orders"))).bind(&id).bind(request).bind(actor).bind(email).bind(outcome).bind(reason).bind(code_id).fetch_one(&mut **tx).await.map_err(db_error)?;
        audit(
            self,
            tx,
            actor,
            "mock_billing_changed",
            json!({"mock":true,"order_id":id,"email":email,"outcome":outcome,"reason":reason}),
        )
        .await?;
        Ok(row)
    }
    pub async fn change_mock(
        &self,
        actor: &str,
        token: &str,
        email: &str,
        outcome: &str,
        reason: &str,
        request: &str,
        admin: bool,
    ) -> Result<Value, StatusCode> {
        if !matches!(outcome, "success" | "failure" | "cancel" | "reset") || !self::reason(reason) {
            return Err(StatusCode::BAD_REQUEST);
        }
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        self.mock_lock(&mut tx).await?;
        actor_lock(self, &mut tx, actor, token, admin).await?;
        let row = self
            .mock_order(&mut tx, actor, email, outcome, reason, request, None)
            .await?;
        tx.commit().await.map_err(db_error)?;
        Ok(row)
    }
}
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Page {
    q: String,
    offset: i64,
}
impl Page {
    fn validate(&self) -> Result<(), StatusCode> {
        if self.q.len() > 254 || !(0..=100000).contains(&self.offset) {
            Err(StatusCode::BAD_REQUEST)
        } else {
            Ok(())
        }
    }
}
pub async fn list_batches(
    state: State<AppState>,
    headers: HeaderMap,
    page: Query<Page>,
) -> Result<Json<Value>, StatusCode> {
    list(state, headers, Path("batches".into()), page).await
}
pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(kind): Path<String>,
    Query(page): Query<Page>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&state, &headers).await?;
    page.validate()?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let source=match kind.as_str(){
        "orders"=>format!("SELECT o.*,true AS mock FROM {} o WHERE $1='' OR strpos(lower(email),lower($1))>0 OR strpos(id,$1)>0 OR outcome=$1",pg.t("mock_orders")),
        "entitlements"=>format!("SELECT a.email,a.pro AS real_pro,COALESCE(m.pro,false) AS mock_pro,COALESCE(m.revision,0) AS revision,true AS mock,a.email AS id FROM {} a LEFT JOIN {} m ON m.email=a.email WHERE $1='' OR strpos(lower(a.email),lower($1))>0",pg.t("accounts"),pg.t("mock_entitlements")),
        "batches"=>format!("SELECT b.*,true AS mock,(SELECT count(*) FROM {} c WHERE c.batch_id=b.id) AS code_count,(SELECT count(*) FROM {} u JOIN {} c ON c.id=u.code_id WHERE c.batch_id=b.id) AS used_count FROM {} b WHERE $1='' OR strpos(lower(name),lower($1))>0 OR strpos(id,$1)>0",pg.t("mock_codes"),pg.t("mock_code_uses"),pg.t("mock_codes"),pg.t("mock_code_batches")),
        _=>return Err(StatusCode::NOT_FOUND)
    };
    let sort = if kind == "entitlements" {
        "email"
    } else {
        "created_at DESC,id"
    };
    let result:Value=sqlx::query_scalar(&format!("WITH filtered AS ({source}) SELECT json_build_object('items',COALESCE((SELECT json_agg(p) FROM (SELECT * FROM filtered ORDER BY {sort} LIMIT 25 OFFSET $2) p),'[]'::json),'total',(SELECT count(*) FROM filtered),'offset',$2::bigint,'limit',25,'mock',true,'enabled',$3::boolean)" )).bind(page.q.trim()).bind(page.offset).bind(state.billing_mock).fetch_one(&pg.pool).await.map_err(db_error)?;
    Ok(Json(result))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    email: String,
    outcome: String,
    reason: String,
    request_id: String,
}
pub async fn change(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Change>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    if !state.billing_mock {
        return Err(StatusCode::CONFLICT);
    }
    state.limit_account_auth(&actor).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let order = pg
        .change_mock(
            &actor,
            &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
            &body.email,
            &body.outcome,
            &body.reason,
            &body.request_id,
            true,
        )
        .await?;
    Ok(Json(json!({"mock":true,"order":order})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Batch {
    name: String,
    count: i64,
    uses_per_code: i32,
    expires_at: chrono::DateTime<chrono::Utc>,
    request_id: String,
}
pub async fn create_batch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Batch>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    if !state.billing_mock {
        return Err(StatusCode::CONFLICT);
    }
    state.limit_account_auth(&actor).await?;
    request_id(&body.request_id)?;
    let now = chrono::Utc::now();
    if body.name.trim().is_empty()
        || body.name.chars().count() > 80
        || !(1..=100).contains(&body.count)
        || !(1..=1000).contains(&body.uses_per_code)
        || body.expires_at <= now
        || body.expires_at > now + chrono::Duration::days(365)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    pg.mock_lock(&mut tx).await?;
    actor_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
        true,
    )
    .await?;
    let duplicate: bool = sqlx::query_scalar(&format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE request_id=$1)",
        pg.t("mock_code_batches")
    ))
    .bind(&body.request_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;
    if duplicate {
        return Err(StatusCode::CONFLICT);
    }
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(&format!("INSERT INTO {} (id,name,actor,request_id,expires_at,uses_per_code) VALUES ($1,$2,$3,$4,$5::timestamptz,$6)",pg.t("mock_code_batches"))).bind(&id).bind(body.name.trim()).bind(&actor).bind(&body.request_id).bind(body.expires_at.to_rfc3339()).bind(body.uses_per_code).execute(&mut *tx).await.map_err(db_error)?;
    let mut codes = Vec::new();
    use ring::rand::SecureRandom;
    for _ in 0..body.count {
        let mut bytes = [0u8; 16];
        ring::rand::SystemRandom::new()
            .fill(&mut bytes)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let code = format!(
            "TEST-{}",
            bytes.iter().map(|b| format!("{b:02X}")).collect::<String>()
        );
        sqlx::query(&format!(
            "INSERT INTO {} (id,batch_id,code_hash,hint) VALUES ($1,$2,$3,$4)",
            pg.t("mock_codes")
        ))
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&id)
        .bind(format!("{:x}", Sha256::digest(code.as_bytes())))
        .bind(format!("TEST-…{}", &code[code.len() - 6..]))
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        codes.push(code);
    }
    audit(pg,&mut tx,&actor,"mock_code_batch_created",json!({"mock":true,"id":id,"name":body.name,"count":body.count,"uses_per_code":body.uses_per_code,"expires_at":body.expires_at})).await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(
        json!({"mock":true,"id":id,"codes":codes,"note":"测试码仅显示一次，不授予真实权益。"}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchChange {
    revision: i64,
    enabled: bool,
}
pub async fn update_batch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<BatchChange>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    if !state.billing_mock {
        return Err(StatusCode::CONFLICT);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    pg.mock_lock(&mut tx).await?;
    actor_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
        true,
    )
    .await?;
    let result:Option<Value>=sqlx::query_scalar(&format!("UPDATE {} AS b SET enabled=$1,revision=revision+1 WHERE id=$2 AND revision=$3 AND (NOT $1 OR expires_at>now()) RETURNING to_jsonb(b)",pg.t("mock_code_batches"))).bind(body.enabled).bind(&id).bind(body.revision).fetch_optional(&mut *tx).await.map_err(db_error)?;
    let result = result.ok_or(StatusCode::CONFLICT)?;
    audit(
        pg,
        &mut tx,
        &actor,
        "mock_code_batch_changed",
        json!({"mock":true,"id":id,"enabled":body.enabled,"revision":body.revision+1}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(result))
}
pub async fn batch_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let batch: Value = sqlx::query_scalar(&format!(
        "SELECT to_jsonb(b) FROM {} b WHERE id=$1",
        pg.t("mock_code_batches")
    ))
    .bind(&id)
    .fetch_optional(&pg.pool)
    .await
    .map_err(db_error)?
    .ok_or(StatusCode::NOT_FOUND)?;
    let codes:Vec<Value>=sqlx::query_scalar(&format!("SELECT json_build_object('id',c.id,'hint',c.hint,'uses',(SELECT count(*) FROM {} WHERE code_id=c.id)) FROM {} c WHERE batch_id=$1 ORDER BY id",pg.t("mock_code_uses"),pg.t("mock_codes"))).bind(&id).fetch_all(&pg.pool).await.map_err(db_error)?;
    let uses:Vec<Value>=sqlx::query_scalar(&format!("SELECT to_jsonb(u) FROM {} u JOIN {} c ON c.id=u.code_id WHERE c.batch_id=$1 ORDER BY u.created_at DESC LIMIT 100",pg.t("mock_code_uses"),pg.t("mock_codes"))).bind(id).fetch_all(&pg.pool).await.map_err(db_error)?;
    Ok(Json(
        json!({"mock":true,"batch":batch,"codes":codes,"recent_uses":uses,"uses_limit":100}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Redeem {
    code: String,
    request_id: Option<String>,
}
pub async fn redeem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Redeem>,
) -> Result<Json<crate::billing::BillingStatus>, StatusCode> {
    let email = require_user(&state, &headers).await?;
    if !state.billing_mock {
        return Err(StatusCode::CONFLICT);
    }
    state.limit_account_auth(&email).await?;
    let code = body.code.trim().to_ascii_uppercase();
    if code.len() != 37
        || !code.starts_with("TEST-")
        || !code[5..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let request = body
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    request_id(&request)?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    pg.mock_lock(&mut tx).await?;
    actor_lock(
        pg,
        &mut tx,
        &email,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
        false,
    )
    .await?;
    let row=sqlx::query(&format!("SELECT c.id,b.enabled,b.expires_at>now() AS valid,b.uses_per_code,(SELECT count(*) FROM {} WHERE code_id=c.id) AS used FROM {} c JOIN {} b ON b.id=c.batch_id WHERE code_hash=$1",pg.t("mock_code_uses"),pg.t("mock_codes"),pg.t("mock_code_batches"))).bind(format!("{:x}",Sha256::digest(code.as_bytes()))).fetch_optional(&mut *tx).await.map_err(db_error)?.ok_or(StatusCode::BAD_REQUEST)?;
    let id: String = row.get("id");
    let existing: Option<Value> = sqlx::query_scalar(&format!(
        "SELECT to_jsonb(o) FROM {} o WHERE actor=$1 AND request_id=$2",
        pg.t("mock_orders")
    ))
    .bind(&email)
    .bind(&request)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_error)?;
    if let Some(existing) = existing {
        if existing["code_id"] != id || existing["outcome"] != "redeem" {
            return Err(StatusCode::CONFLICT);
        }
    } else {
        let used: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE code_id=$1 AND email=$2)",
            pg.t("mock_code_uses")
        ))
        .bind(&id)
        .bind(&email)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        if used
            || !row.get::<bool, _>("enabled")
            || !row.get::<bool, _>("valid")
            || row.get::<i64, _>("used") >= i64::from(row.get::<i32, _>("uses_per_code"))
        {
            return Err(StatusCode::CONFLICT);
        }
        let order = pg
            .mock_order(
                &mut tx,
                &email,
                &email,
                "redeem",
                "测试码兑换",
                &request,
                Some(&id),
            )
            .await?;
        sqlx::query(&format!(
            "INSERT INTO {} (code_id,email,order_id) VALUES ($1,$2,$3)",
            pg.t("mock_code_uses")
        ))
        .bind(id)
        .bind(&email)
        .bind(order["id"].as_str())
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
    }
    tx.commit().await.map_err(db_error)?;
    Ok(Json(crate::billing::status_for(&state, &email).await?))
}
