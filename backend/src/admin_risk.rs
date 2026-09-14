use crate::{bearer_token, postgres::Pg, require_admin, require_user, AppState};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Postgres, Row, Transaction};

fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
fn category(value: &str) -> bool {
    matches!(
        value,
        "privacy" | "medical" | "financial" | "copyright" | "danger" | "spam" | "other"
    )
}
pub(crate) async fn actor_lock(
    pg: &Pg,
    tx: &mut Transaction<'_, Postgres>,
    actor: &str,
    token: &str,
    admin: bool,
) -> Result<String, StatusCode> {
    let role:Option<String> = sqlx::query_scalar(&format!("SELECT role FROM {} a WHERE email=$1 AND NOT disabled AND EXISTS(SELECT 1 FROM {} WHERE email=a.email AND token=$2) FOR UPDATE",pg.t("accounts"),pg.t("access_tokens"))).bind(actor).bind(token).fetch_optional(&mut **tx).await.map_err(db_error)?;
    match role {
        None => Err(StatusCode::UNAUTHORIZED),
        Some(role) if !admin || matches!(role.as_str(), "owner" | "admin") => Ok(role),
        _ => Err(StatusCode::FORBIDDEN),
    }
}
pub(crate) async fn audit(
    pg: &Pg,
    tx: &mut Transaction<'_, Postgres>,
    actor: &str,
    action: &str,
    details: Value,
) -> Result<(), StatusCode> {
    sqlx::query(&format!(
        "INSERT INTO {} (id,actor_email,action,details) VALUES ($1,$2,$3,$4)",
        pg.t("security_audit")
    ))
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(actor)
    .bind(action)
    .bind(details)
    .execute(&mut **tx)
    .await
    .map_err(db_error)?;
    Ok(())
}
impl Pg {
    pub async fn init_risk(&self) -> Result<(), sqlx::Error> {
        for statement in [
            format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY,target_id TEXT NOT NULL,reporter TEXT NOT NULL,category TEXT NOT NULL,reason TEXT NOT NULL,status TEXT NOT NULL DEFAULT 'pending',priority TEXT NOT NULL DEFAULT 'normal',assignee TEXT,revision BIGINT NOT NULL DEFAULT 0,resolution TEXT NOT NULL DEFAULT '',created_at TIMESTAMPTZ NOT NULL DEFAULT now(),updated_at TIMESTAMPTZ NOT NULL DEFAULT now())",self.t("reports")),
            format!("CREATE UNIQUE INDEX IF NOT EXISTS reports_open_target ON {}(reporter,target_id) WHERE status IN ('pending','processing')",self.t("reports")),
            format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY,report_id TEXT NOT NULL,actor TEXT NOT NULL,action TEXT NOT NULL,reason TEXT NOT NULL,created_at TIMESTAMPTZ NOT NULL DEFAULT now(),details JSONB NOT NULL)",self.t("report_events")),
            format!("CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY CHECK(id=1),data JSONB NOT NULL)",self.t("safety_rules")),
            format!("INSERT INTO {} (id,data) VALUES (1,'{{\"revision\":0,\"rules\":[]}}') ON CONFLICT DO NOTHING",self.t("safety_rules")),
        ] { sqlx::query(&statement).execute(&self.pool).await?; }
        Ok(())
    }
    pub async fn risk_rules(&self) -> Result<Rules, StatusCode> {
        let value: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1",
            self.t("safety_rules")
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;
        serde_json::from_value(value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub category: String,
    pub words: Vec<String>,
    pub score: u8,
    pub enabled: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub revision: i64,
    pub rules: Vec<Rule>,
}
impl Rules {
    fn valid(&self) -> bool {
        let mut ids = std::collections::HashSet::new();
        self.rules.len() <= 100
            && self.rules.iter().all(|r| {
                !r.id.is_empty()
                    && r.id.len() <= 80
                    && ids.insert(&r.id)
                    && !r.name.trim().is_empty()
                    && r.name.chars().count() <= 80
                    && category(&r.category)
                    && r.score <= 100
                    && !r.words.is_empty()
                    && r.words.len() <= 100
                    && r.words
                        .iter()
                        .all(|w| !w.trim().is_empty() && w.chars().count() <= 80)
            })
    }
    pub fn evaluate(&self, text: &str, selected: Option<&str>) -> Value {
        let text = text.to_lowercase();
        let hits:Vec<Value>=self.rules.iter().filter(|r|r.enabled && selected.map_or(true,|c|c==r.category)).filter_map(|r| {
            let words:Vec<&String>=r.words.iter().filter(|w|text.contains(&w.trim().to_lowercase())).collect();
            (!words.is_empty()).then(||json!({"id":r.id,"name":r.name,"category":r.category,"score":r.score,"words":words}))
        }).collect();
        let score = hits
            .iter()
            .filter_map(|h| h["score"].as_u64())
            .max()
            .unwrap_or(0);
        json!({"source":"local_rules","revision":self.revision,"score":score,"hits":hits,"notice":"仅检测当前字面规则，无命中不代表内容安全；未调用 AI。"})
    }
}
pub async fn rules(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Rules>, StatusCode> {
    require_admin(&state, &headers).await?;
    Ok(Json(
        state
            .db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
            .risk_rules()
            .await?,
    ))
}
pub async fn save_rules(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut input): Json<Rules>,
) -> Result<Json<Rules>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    if !input.valid() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    actor_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
        true,
    )
    .await?;
    let before: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} WHERE id=1 FOR UPDATE",
        pg.t("safety_rules")
    ))
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;
    if before["revision"].as_i64() != Some(input.revision) {
        return Err(StatusCode::CONFLICT);
    }
    input.revision += 1;
    sqlx::query(&format!(
        "UPDATE {} SET data=$1 WHERE id=1",
        pg.t("safety_rules")
    ))
    .bind(json!(input))
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    audit(
        pg,
        &mut tx,
        &actor,
        "safety_rules_updated",
        json!({"before":before,"after":input}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(input))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestRules {
    text: String,
    category: Option<String>,
}
pub async fn test_rules(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<TestRules>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&state, &headers).await?;
    if input.text.len() > 200_000 || input.category.as_deref().is_some_and(|c| !category(c)) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(Json(
        state
            .db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
            .risk_rules()
            .await?
            .evaluate(&input.text, input.category.as_deref()),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewReport {
    target_id: String,
    category: String,
    reason: String,
}
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<NewReport>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_user(&state, &headers).await?;
    if !category(&input.category)
        || input.reason.trim().is_empty()
        || input.reason.chars().count() > 1000
        || input.target_id.len() > 200
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    let role = actor_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
        false,
    )
    .await?;
    let existing:Option<Value>=sqlx::query_scalar(&format!("SELECT jsonb_build_object('id',id,'status',status) FROM {} WHERE reporter=$1 AND target_id=$2 AND status IN ('pending','processing')",pg.t("reports"))).bind(&actor).bind(&input.target_id).fetch_optional(&mut *tx).await.map_err(db_error)?;
    if let Some(value) = existing {
        return Ok(Json(value));
    }
    let target: Option<String> = sqlx::query_scalar(&format!(
        "SELECT id FROM {} WHERE id=$1 AND visibility='online' FOR SHARE",
        pg.t("square_items")
    ))
    .bind(&input.target_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_error)?;
    if target.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let count: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {} WHERE reporter=$1 AND created_at>=now()-interval '24 hours'",
        pg.t("reports")
    ))
    .bind(&actor)
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;
    if count >= 20 && !matches!(role.as_str(), "admin" | "owner") {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(&format!(
        "INSERT INTO {} (id,target_id,reporter,category,reason) VALUES ($1,$2,$3,$4,$5)",
        pg.t("reports")
    ))
    .bind(&id)
    .bind(&input.target_id)
    .bind(&actor)
    .bind(&input.category)
    .bind(input.reason.trim())
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    audit(
        pg,
        &mut tx,
        &actor,
        "report_created",
        json!({"id":id,"target_id":input.target_id,"category":input.category}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(json!({"id":id,"status":"pending"})))
}
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Filter {
    q: String,
    status: String,
    category: String,
    priority: String,
    assignee: String,
    offset: i64,
}
impl Filter {
    fn valid(&self) -> bool {
        self.q.len() <= 200
            && self.offset >= 0
            && self.offset <= 100_000
            && matches!(
                self.status.as_str(),
                "" | "pending" | "processing" | "dismissed" | "closed"
            )
            && (self.category.is_empty() || category(&self.category))
            && matches!(self.priority.as_str(), "" | "normal" | "high")
            && self.assignee.len() <= 254
    }
}
const FILTER:&str="($1='' OR strpos(lower(r.target_id || ' ' || r.reason || ' ' || r.reporter),lower($1))>0) AND ($2='' OR r.status=$2) AND ($3='' OR r.category=$3) AND ($4='' OR r.priority=$4) AND ($5='' OR r.assignee=$5)";
async fn list_rows(
    pg: &Pg,
    input: &Filter,
    limit: i64,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Value, StatusCode> {
    if !input.valid() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {} r WHERE {FILTER}",
        pg.t("reports")
    ))
    .bind(&input.q)
    .bind(&input.status)
    .bind(&input.category)
    .bind(&input.priority)
    .bind(&input.assignee)
    .fetch_one(&mut **tx)
    .await
    .map_err(db_error)?;
    let items:Vec<Value>=sqlx::query_scalar(&format!("SELECT to_jsonb(r) FROM {} r WHERE {FILTER} ORDER BY created_at DESC,id LIMIT $6 OFFSET $7",pg.t("reports"))).bind(&input.q).bind(&input.status).bind(&input.category).bind(&input.priority).bind(&input.assignee).bind(limit).bind(input.offset).fetch_all(&mut **tx).await.map_err(db_error)?;
    Ok(json!({"items":items,"total":total,"offset":input.offset,"limit":limit}))
}
pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(input): Query<Filter>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    let mut result = list_rows(pg, &input, 25, &mut tx).await?;
    let assignees: Vec<String> = sqlx::query_scalar(&format!(
        "SELECT email FROM {} WHERE NOT disabled AND role IN ('owner','admin') ORDER BY email",
        pg.t("accounts")
    ))
    .fetch_all(&mut *tx)
    .await
    .map_err(db_error)?;
    result["assignees"] = json!(assignees);
    tx.commit().await.map_err(db_error)?;
    Ok(Json(result))
}
pub async fn mine(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(input): Query<Filter>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_user(&state, &headers).await?;
    if !input.valid() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let items:Vec<Value>=sqlx::query_scalar(&format!("SELECT jsonb_build_object('id',id,'target_id',target_id,'category',category,'reason',reason,'status',status,'resolution',resolution,'created_at',created_at,'updated_at',updated_at) FROM {} WHERE reporter=$1 ORDER BY created_at DESC,id LIMIT 25 OFFSET $2",pg.t("reports"))).bind(&actor).bind(input.offset).fetch_all(&pg.pool).await.map_err(db_error)?;
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM {} WHERE reporter=$1",
        pg.t("reports")
    ))
    .bind(actor)
    .fetch_one(&pg.pool)
    .await
    .map_err(db_error)?;
    Ok(Json(
        json!({"items":items,"total":total,"offset":input.offset,"limit":25}),
    ))
}
pub async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut value: Value = sqlx::query_scalar(&format!(
        "SELECT to_jsonb(r) FROM {} r WHERE id=$1",
        pg.t("reports")
    ))
    .bind(&id)
    .fetch_optional(&pg.pool)
    .await
    .map_err(db_error)?
    .ok_or(StatusCode::NOT_FOUND)?;
    let target: Option<Value> = sqlx::query_scalar(&format!(
        "SELECT to_jsonb(s) FROM {} s WHERE id=$1",
        pg.t("square_items")
    ))
    .bind(value["target_id"].as_str())
    .fetch_optional(&pg.pool)
    .await
    .map_err(db_error)?;
    let target = target.unwrap_or(Value::Null);
    let text = format!(
        "{}\n{}\n{}",
        target["title"].as_str().unwrap_or(""),
        target["content"].as_str().unwrap_or(""),
        target["members"]
    );
    value["rules"] = pg.risk_rules().await?.evaluate(&text, None);
    value["target"] = target;
    let events: Vec<Value> = sqlx::query_scalar(&format!(
        "SELECT to_jsonb(e) FROM {} e WHERE report_id=$1 ORDER BY created_at,id",
        pg.t("report_events")
    ))
    .bind(id)
    .fetch_all(&pg.pool)
    .await
    .map_err(db_error)?;
    value["events"] = json!(events);
    Ok(Json(value))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    revision: i64,
    action: String,
    reason: String,
    priority: String,
    assignee: Option<String>,
}
pub async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Action>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    if !matches!(
        input.action.as_str(),
        "start" | "assign" | "dismiss" | "offline"
    ) || !matches!(input.priority.as_str(), "normal" | "high")
        || input.reason.trim().is_empty()
        || input.reason.chars().count() > 1000
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    actor_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
        true,
    )
    .await?;
    let row = sqlx::query(&format!(
        "SELECT * FROM {} WHERE id=$1 FOR UPDATE",
        pg.t("reports")
    ))
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_error)?
    .ok_or(StatusCode::NOT_FOUND)?;
    if row.get::<i64, _>("revision") != input.revision
        || !matches!(row.get::<&str, _>("status"), "pending" | "processing")
    {
        return Err(StatusCode::CONFLICT);
    }
    let assignee = if input.action == "assign" {
        input
            .assignee
            .as_deref()
            .filter(|a| !a.is_empty())
            .ok_or(StatusCode::BAD_REQUEST)?
    } else {
        &actor
    };
    // Use a non-locking assignment check: role changes revoke action rights immediately; no lock-order inversion across administrators.
    let valid:bool=sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE email=$1 AND NOT disabled AND role IN ('owner','admin'))",pg.t("accounts"))).bind(assignee).fetch_one(&mut *tx).await.map_err(db_error)?;
    if !valid {
        return Err(StatusCode::BAD_REQUEST);
    }
    let status = match input.action.as_str() {
        "dismiss" => "dismissed",
        "offline" => "closed",
        _ => "processing",
    };
    if input.action == "offline" {
        let target = row.get::<String, _>("target_id");
        let visibility: Option<String> = sqlx::query_scalar(&format!(
            "SELECT visibility FROM {} WHERE id=$1 FOR UPDATE",
            pg.t("square_items")
        ))
        .bind(&target)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?;
        if visibility.is_none() {
            return Err(StatusCode::NOT_FOUND);
        }
        if visibility.as_deref() == Some("online") {
            let revision:i64=sqlx::query_scalar(&format!(
                "UPDATE {} SET visibility='offline',revision=revision+1 WHERE id=$1 RETURNING revision",
                pg.t("square_items")
            ))
            .bind(&target)
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;
            audit(pg,&mut tx,&actor,"content_updated",json!({"item_id":target,"revision":revision,"report_id":id,"reason":input.reason.trim(),"before":{"visibility":"online"},"after":{"visibility":"offline"}})).await?;
        }
    }
    let result:Value=sqlx::query_scalar(&format!("UPDATE {} r SET status=$2,priority=$3,assignee=$4,revision=revision+1,resolution=$5,updated_at=now() WHERE id=$1 RETURNING to_jsonb(r)",pg.t("reports"))).bind(&id).bind(status).bind(&input.priority).bind(assignee).bind(if matches!(status,"closed"|"dismissed"){input.reason.trim()}else{""}).fetch_one(&mut *tx).await.map_err(db_error)?;
    sqlx::query(&format!(
        "INSERT INTO {} (id,report_id,actor,action,reason,details) VALUES ($1,$2,$3,$4,$5,$6)",
        pg.t("report_events")
    ))
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&id)
    .bind(&actor)
    .bind(&input.action)
    .bind(input.reason.trim())
    .bind(json!({"revision":input.revision+1,"priority":input.priority,"assignee":assignee}))
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    audit(pg,&mut tx,&actor,&format!("report_{}",input.action),json!({"id":id,"target_id":row.get::<String,_>("target_id"),"revision":input.revision+1,"reason":input.reason.trim()})).await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(result))
}
pub async fn export(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(mut input): Query<Filter>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    actor_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
        true,
    )
    .await?;
    input.offset = 0;
    let mut value = list_rows(pg, &input, 500, &mut tx).await?;
    if let Some(items) = value["items"].as_array_mut() {
        for item in items {
            if let Some(map) = item.as_object_mut() {
                map.remove("reason");
                map.remove("resolution");
                map.remove("reporter");
            }
        }
    }
    audit(
        pg,
        &mut tx,
        &actor,
        "reports_exported",
        json!({"count":value["items"].as_array().map(Vec::len),"limit":500}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(value))
}
