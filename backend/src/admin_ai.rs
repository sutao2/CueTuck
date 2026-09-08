use crate::{
    admin_risk::{actor_lock, audit},
    ai_transport::{self, Verdict},
    bearer_token,
    postgres::Pg,
    require_configuration_admin, AppState,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
async fn owner_lock(
    pg: &Pg,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &str,
    token: &str,
) -> Result<(), StatusCode> {
    actor_lock(pg, tx, actor, token, true).await?;
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub model: String,
    pub enabled: bool,
    pub json_mode: bool,
    pub redact: bool,
    pub timeout_seconds: u64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub instruction: String,
    pub enabled: bool,
    pub kinds: Vec<String>,
    pub categories: Vec<String>,
    pub route: String,
    pub models: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredModel {
    pub config: Model,
    encrypted_secret: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub revision: i64,
    pub models: Vec<StoredModel>,
    pub skills: Vec<Skill>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            revision: 0,
            models: vec![],
            skills: vec![Skill {
                id: "general".into(),
                name: "通用内容安全".into(),
                instruction:
                    "核查隐私泄露、违法危险行为、恶意诱导和明显欺诈；区分讨论风险与鼓励风险。"
                        .into(),
                enabled: true,
                kinds: vec!["prompt".into(), "collection".into()],
                categories: vec![],
                route: "fallback".into(),
                models: vec![],
            }],
        }
    }
}
impl Config {
    fn view(&self) -> Value {
        json!({"revision":self.revision,"models":self.models.iter().map(|m| {let mut value=json!(m.config);value["secret_configured"]=json!(!m.encrypted_secret.is_empty());value}).collect::<Vec<_>>(),"skills":self.skills})
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Save {
    revision: i64,
    models: Vec<Model>,
    skills: Vec<Skill>,
    #[serde(default)]
    secrets: HashMap<String, String>,
    current_password: String,
}
fn identifier(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}
fn validate(input: &Save) -> bool {
    let ids: HashSet<_> = input.models.iter().map(|m| m.id.as_str()).collect();
    let skills: HashSet<_> = input.skills.iter().map(|s| s.id.as_str()).collect();
    input.revision >= 0
        && input.models.len() <= 20
        && input.skills.len() <= 30
        && ids.len() == input.models.len()
        && skills.len() == input.skills.len()
        && input.current_password.len() <= 512
        && input.secrets.iter().all(|(id, key)| {
            ids.contains(id.as_str()) && key.len() <= 4096 && !key.contains(['\r', '\n'])
        })
        && input.models.iter().all(|m| {
            identifier(&m.id)
                && !m.name.trim().is_empty()
                && m.name.chars().count() <= 80
                && !m.model.trim().is_empty()
                && m.model.len() <= 200
                && (1..=15).contains(&m.timeout_seconds)
                && crate::outbound::validate_url(&m.endpoint).is_ok()
        })
        && input.skills.iter().all(|s| {
            identifier(&s.id)
                && !s.name.trim().is_empty()
                && s.name.chars().count() <= 80
                && !s.instruction.trim().is_empty()
                && s.instruction.len() <= 10000
                && !s.kinds.is_empty()
                && s.kinds.len() <= 2
                && s.kinds
                    .iter()
                    .all(|k| ["prompt", "collection"].contains(&k.as_str()))
                && s.categories.len() <= 50
                && s.categories.iter().all(|id| id.len() <= 200)
                && ["fallback", "consensus", "majority"].contains(&s.route.as_str())
                && s.models.len() <= 3
                && s.models.iter().collect::<HashSet<_>>().len() == s.models.len()
                && s.models.iter().all(|id| ids.contains(id.as_str()))
        })
}
impl Pg {
    pub async fn init_ai(&self) -> Result<(), sqlx::Error> {
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (revision BIGINT PRIMARY KEY,data JSONB NOT NULL,actor TEXT NOT NULL,created_at TIMESTAMPTZ NOT NULL DEFAULT now())",self.t("ai_configuration"))).execute(&self.pool).await?;
        sqlx::query(&format!(
            "INSERT INTO {} (revision,data,actor) VALUES (0,$1,'system') ON CONFLICT DO NOTHING",
            self.t("ai_configuration")
        ))
        .bind(json!(Config::default()))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (id BIGSERIAL PRIMARY KEY,revision BIGINT NOT NULL,actor TEXT NOT NULL,model_id TEXT,skill_id TEXT NOT NULL,success BOOLEAN NOT NULL,error TEXT,created_at TIMESTAMPTZ NOT NULL DEFAULT now())",self.t("ai_tests"))).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn ai_config(&self) -> Result<Config, StatusCode> {
        let data: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} ORDER BY revision DESC LIMIT 1",
            self.t("ai_configuration")
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;
        serde_json::from_value(data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
    pub async fn has_ai_secrets(&self) -> Result<bool, StatusCode> {
        // History also needs the original encryption key, even if current models were removed.
        sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {},jsonb_array_elements(data->'models') m WHERE m->>'encrypted_secret'<>'')",self.t("ai_configuration"))).fetch_one(&self.pool).await.map_err(db_error)
    }
}
pub async fn get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut view = pg.ai_config().await?.view();
    let tests: Vec<Value> = sqlx::query_scalar(&format!(
        "SELECT to_jsonb(t) FROM {} t ORDER BY id DESC LIMIT 50",
        pg.t("ai_tests")
    ))
    .fetch_all(&pg.pool)
    .await
    .map_err(db_error)?;
    view["tests"] = json!(tests);
    Ok(Json(view))
}
pub async fn history(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let rows:Vec<Value>=sqlx::query_scalar(&format!("SELECT jsonb_build_object('revision',revision,'actor',actor,'created_at',created_at,'data',data) FROM {} ORDER BY revision DESC LIMIT 50",pg.t("ai_configuration"))).fetch_all(&pg.pool).await.map_err(db_error)?;
    let mut views = vec![];
    for mut row in rows {
        let config: Config = serde_json::from_value(row["data"].take())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        row["data"] = config.view();
        views.push(row);
    }
    Ok(Json(json!({"items":views})))
}
pub async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Save>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_configuration_admin(&state, &headers).await?;
    state.limit_account_auth(&actor).await?;
    if !validate(&input) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let token = bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    owner_lock(pg, &mut tx, &actor, &token).await?;
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
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
        .bind(format!("{}:ai-config", pg.schema))
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
    let old: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} ORDER BY revision DESC LIMIT 1",
        pg.t("ai_configuration")
    ))
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;
    let old: Config = serde_json::from_value(old).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if input.revision != old.revision {
        return Err(StatusCode::CONFLICT);
    }
    // Keep category deletion and Skill references consistent with the dictionary.
    pg.catalog_lock(&mut tx).await?;
    for skill in &input.skills {
        for id in &skill.categories {
            let exists: bool = sqlx::query_scalar(&format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE kind='categories' AND id=$1 AND NOT deleted)",
                pg.t("catalog")
            ))
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;
            if !exists {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }
    let mut models = vec![];
    for config in input.models {
        let encrypted_secret = match input.secrets.get(&config.id).filter(|s| !s.is_empty()) {
            Some(secret) => crate::oauth_admin::seal(
                &state.oauth_config.key,
                &format!("ai:{}", config.id),
                secret,
            )?,
            None => old
                .models
                .iter()
                .find(|m| m.config.id == config.id)
                .map(|m| m.encrypted_secret.clone())
                .unwrap_or_default(),
        };
        if config.enabled && encrypted_secret.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
        models.push(StoredModel {
            config,
            encrypted_secret,
        });
    }
    let config = Config {
        revision: old.revision + 1,
        models,
        skills: input.skills,
    };
    sqlx::query(&format!(
        "INSERT INTO {} (revision,data,actor) VALUES ($1,$2,$3)",
        pg.t("ai_configuration")
    ))
    .bind(config.revision)
    .bind(json!(config))
    .bind(&actor)
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    audit(pg,&mut tx,&actor,"ai_configuration_saved",json!({"revision":config.revision,"models":config.models.iter().map(|m|&m.config.id).collect::<Vec<_>>(),"skills":config.skills.iter().map(|s|&s.id).collect::<Vec<_>>(),"rotated_keys":input.secrets.keys().collect::<Vec<_>>()})).await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(config.view()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Test {
    revision: i64,
    skill_id: String,
    model_id: Option<String>,
    text: String,
}
#[derive(Serialize, Clone)]
pub struct Run {
    pub revision: i64,
    pub skill_id: String,
    pub verdict: Option<Verdict>,
    pub models: Vec<Value>,
    pub error: Option<String>,
}

pub fn combine(route: &str, results: &[Verdict]) -> Option<Verdict> {
    let required = match route {
        "fallback" => 1,
        "consensus" => 2,
        "majority" => 3,
        _ => return None,
    };
    if results.len() != required {
        return None;
    }
    let decision = if route == "fallback" {
        results[0].decision.clone()
    } else {
        ["approve", "reject", "manual"]
            .iter()
            .find(|d| results.iter().filter(|r| r.decision == **d).count() >= 2)?
            .to_string()
    };
    Some(Verdict {
        risk_score: results.iter().map(|r| r.risk_score).max().unwrap_or(100),
        decision,
        reasons: results
            .iter()
            .flat_map(|r| r.reasons.clone())
            .take(10)
            .collect(),
        matched_rules: results
            .iter()
            .flat_map(|r| r.matched_rules.clone())
            .take(50)
            .collect(),
    })
}
pub async fn run(
    config: &Config,
    skill: &Skill,
    key: &[u8; 32],
    text: &str,
    only: Option<&str>,
) -> Run {
    let mut run = Run {
        revision: config.revision,
        skill_id: skill.id.clone(),
        verdict: None,
        models: vec![],
        error: None,
    };
    let selected: Vec<_> = if let Some(id) = only {
        config
            .models
            .iter()
            .filter(|m| m.config.id == id && m.config.enabled)
            .collect()
    } else {
        skill
            .models
            .iter()
            .filter_map(|id| {
                config
                    .models
                    .iter()
                    .find(|m| m.config.id == *id && m.config.enabled)
            })
            .collect()
    };
    let route = if only.is_some() {
        "fallback"
    } else {
        &skill.route
    };
    let required = match route {
        "consensus" => 2,
        "majority" => 3,
        _ => 1,
    };
    if route != "fallback"
        && selected
            .iter()
            .map(|m| (&m.config.endpoint, &m.config.model))
            .collect::<HashSet<_>>()
            .len()
            != selected.len()
    {
        run.error = Some("一致/多数路由不能重复使用同一接口模型".into());
        return run;
    }
    if selected.len() < required || (route != "fallback" && selected.len() != required) {
        run.error = Some("启用的独立模型数量不足，转人工".into());
        return run;
    }
    let mut results = vec![];
    let work = async {
        for stored in selected {
            let m = &stored.config;
            let result = async {
                let secret = crate::oauth_admin::unseal(
                    key,
                    &format!("ai:{}", m.id),
                    &stored.encrypted_secret,
                )
                .map_err(|_| "无法解密模型密钥".to_owned())?;
                if secret.is_empty() {
                    return Err("未配置模型密钥".into());
                }
                let client = crate::outbound::client(&m.endpoint, m.timeout_seconds).await?;
                let input = if m.redact {
                    ai_transport::redact(text)
                } else {
                    text.to_owned()
                };
                ai_transport::send(
                    &client,
                    &m.endpoint,
                    &secret,
                    &m.model,
                    &skill.instruction,
                    &input,
                    m.json_mode,
                )
                .await
            }
            .await;
            match result {
                Ok(verdict) => {
                    run.models
                        .push(json!({"id":m.id,"model":m.model,"result":verdict}));
                    results.push(verdict);
                    if route == "fallback" {
                        break;
                    }
                }
                Err(error) => {
                    run.models.push(json!({"id":m.id,"error":error}));
                    if route != "fallback" {
                        break;
                    }
                }
            }
        }
    };
    if tokio::time::timeout(std::time::Duration::from_secs(25), work)
        .await
        .is_err()
    {
        run.error = Some("审核链路超过 25 秒，转人工".into());
        return run;
    }
    run.verdict = combine(route, &results);
    if run.verdict.is_none() {
        run.error = Some("必要模型失败或结论不一致，转人工".into())
    }
    run
}
pub async fn test(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Test>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_configuration_admin(&state, &headers).await?;
    state.limit_account_auth(&actor).await?;
    if input.text.trim().is_empty() || input.text.len() > 8000 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let config = pg.ai_config().await?;
    if config.revision != input.revision {
        return Err(StatusCode::CONFLICT);
    }
    let skill = config
        .skills
        .iter()
        .find(|s| s.id == input.skill_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let result = run(
        &config,
        skill,
        &state.oauth_config.key,
        &input.text,
        input.model_id.as_deref(),
    )
    .await;
    let mut tx = pg.pool.begin().await.map_err(db_error)?;
    owner_lock(
        pg,
        &mut tx,
        &actor,
        &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
    )
    .await?;
    sqlx::query(&format!("INSERT INTO {} (revision,actor,model_id,skill_id,success,error) VALUES ($1,$2,$3,$4,$5,$6)",pg.t("ai_tests"))).bind(config.revision).bind(&actor).bind(&input.model_id).bind(&input.skill_id).bind(result.verdict.is_some()).bind(&result.error).execute(&mut *tx).await.map_err(db_error)?;
    audit(pg,&mut tx,&actor,"ai_configuration_tested",json!({"revision":config.revision,"skill_id":input.skill_id,"model_id":input.model_id,"success":result.verdict.is_some()})).await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(json!(result)))
}

pub async fn screen_publication(
    state: &AppState,
    publication: &crate::Publication,
) -> Result<crate::Publication, StatusCode> {
    if !publication.asset_refs.is_empty() { return Ok(publication.clone()); }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let local: Option<Value> = sqlx::query_scalar(&format!(
        "SELECT moderation FROM {} WHERE id=$1",
        pg.t("publications")
    ))
    .bind(&publication.id)
    .fetch_one(&pg.pool)
    .await
    .map_err(db_error)?;
    let Some(local) = local else {
        return Ok(publication.clone());
    };
    if publication.status != "pending" || local["checks"]["require_ai"] != true {
        return Ok(publication.clone());
    }
    let config = pg.ai_config().await?;
    let mut parts = vec![
        publication.title.as_deref().unwrap_or(""),
        publication.content.as_deref().unwrap_or(""),
    ];
    for member in &publication.members {
        parts.push(&member.title);
        parts.push(&member.content);
    }
    let text = parts.join("\n");
    let mut runs = vec![];
    let mut error = None;
    if text.len() > 32000 {
        error = Some("投稿超出 AI 单次 32 KB 文本范围，转人工".to_owned())
    } else if local["local_reasons"]
        .as_array()
        .is_some_and(|v| !v.is_empty())
    {
        error = Some("本地检查已要求人工复核，未发送模型".into())
    } else {
        let work = async {
            for skill in config.skills.iter().filter(|s| {
                s.enabled
                    && s.kinds.contains(&publication.kind)
                    && (s.categories.is_empty()
                        || publication
                            .category_id
                            .as_ref()
                            .is_some_and(|id| s.categories.contains(id)))
            }) {
                runs.push(run(&config, skill, &state.oauth_config.key, &text, None).await);
            }
        };
        if tokio::time::timeout(std::time::Duration::from_secs(25), work)
            .await
            .is_err()
        {
            error = Some("全部 Skills 审核超过 25 秒，转人工".into())
        }
        if runs.is_empty() && error.is_none() {
            error = Some("没有匹配的启用 Skill，转人工".into())
        }
    }
    pg.finish_ai(publication, &local, config.revision, &runs, error)
        .await
}
impl Pg {
    pub(crate) async fn finish_ai(
        &self,
        publication: &crate::Publication,
        local: &Value,
        revision: i64,
        runs: &[Run],
        mut error: Option<String>,
    ) -> Result<crate::Publication, StatusCode> {
        use crate::admin_moderation::Policy;
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        // Same ordering as configuration writers. No locks are held while calling providers.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(format!("{}:ai-config", self.schema))
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        let current: i64 = sqlx::query_scalar(&format!(
            "SELECT max(revision) FROM {}",
            self.t("ai_configuration")
        ))
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        let policy: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1 FOR SHARE",
            self.t("moderation_policy")
        ))
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        let policy: Policy =
            serde_json::from_value(policy).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let rules: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1 FOR SHARE",
            self.t("safety_rules")
        ))
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        if current != revision
            || local["policy_revision"] != policy.revision
            || local["rules_revision"] != rules["revision"]
            || !policy.enabled
            || !policy.require_ai
        {
            error = Some("审核期间策略、词库或模型/Skill 版本变化，转人工".into())
        }
        self.catalog_lock(&mut tx).await?;
        if let Some(mut item)=publication.square_item() {
            if self.resolve_catalog_item(&mut tx,&mut item).await? {
                error=Some("审核期间分类或模型已迁移，转人工复核".into());
            }
        }
        let status: String = sqlx::query_scalar(&format!(
            "SELECT status FROM {} WHERE id=$1 FOR UPDATE",
            self.t("publications")
        ))
        .bind(&publication.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        let mut result = publication.clone();
        result.status = status;
        if result.status != "pending" {
            return Ok(result);
        } // A human decision always wins.
        let local_clear = local["local_reasons"]
            .as_array()
            .is_some_and(|r| r.is_empty());
        let score = runs
            .iter()
            .filter_map(|r| r.verdict.as_ref().map(|v| u64::from(v.risk_score)))
            .chain([local["score"].as_u64().unwrap_or(100)])
            .max()
            .unwrap_or(100);
        let passed = !runs.is_empty()
            && runs.iter().all(|r| {
                r.revision == revision
                    && r.error.is_none()
                    && r.verdict.as_ref().is_some_and(|v| v.decision == "approve")
            });
        let mut approved = error.is_none()
            && local_clear
            && passed
            && policy.auto_approve
            && score < u64::from(policy.approve_below);
        if approved
            && self
                .validate_catalog_refs(
                    &mut tx,
                    publication.category_id.as_deref(),
                    publication.model.as_deref(),
                )
                .await
                .is_err()
        {
            approved = false;
            error = Some("投稿引用字典已停用，转人工".into())
        }
        for member in &publication.members {
            if approved
                && self
                    .validate_catalog_refs(
                        &mut tx,
                        member.category_id.as_deref(),
                        member.model.as_deref(),
                    )
                    .await
                    .is_err()
            {
                approved = false;
                error = Some("合集成员引用字典已停用，转人工".into())
            }
        }
        let mut moderation = local.clone();
        moderation["source"] = json!("local_rules_and_ai");
        moderation["score"] = json!(score);
        moderation["ai"] = json!({"revision":revision,"runs":runs,"error":error});
        moderation["decision"] = json!(if approved { "approved" } else { "manual" });
        moderation["notice"] = json!("AI 结果仅辅助内容审核，不是安全保证");
        moderation["reasons"] = if approved {
            json!([])
        } else {
            json!([error.unwrap_or_else(|| "按策略、模型结论或本地检查结果转人工".into())])
        };
        if approved {
            result.status = "approved".into();
            let item = result
                .square_item()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            sqlx::query(&format!("INSERT INTO {} (id,title,kind,excerpt,model,member_count,content,category_id,members,sort_index) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,COALESCE((SELECT max(sort_index)+1 FROM {0}),0))",self.t("square_items"))).bind(&item.id).bind(&item.title).bind(&item.kind).bind(&item.excerpt).bind(&item.model).bind(item.member_count).bind(&item.content).bind(&item.category_id).bind(json!(item.members)).execute(&mut *tx).await.map_err(db_error)?;
            sqlx::query(&format!("INSERT INTO {} (publication_id,actor_email,status,reason) VALUES ($1,'system:ai','approved',$2)",self.t("review_events"))).bind(&result.id).bind(format!("按站点策略及 AI 配置版本 {revision} 自动通过")).execute(&mut *tx).await.map_err(db_error)?;
        }
        sqlx::query(&format!(
            "UPDATE {} SET status=$2,moderation=$3 WHERE id=$1",
            self.t("publications")
        ))
        .bind(&result.id)
        .bind(&result.status)
        .bind(&moderation)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        audit(self,&mut tx,"system:ai","publication_ai_screened",json!({"publication_id":result.id,"status":result.status,"revision":revision,"score":score})).await?;
        tx.commit().await.map_err(db_error)?;
        Ok(result)
    }
}
