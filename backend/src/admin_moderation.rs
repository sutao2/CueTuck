use crate::{
    admin_risk::{actor_lock, audit, Rules},
    bearer_token,
    postgres::Pg,
    require_configuration_admin, AppState, Publication,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Policy {
    pub revision: i64,
    pub enabled: bool,
    pub daily_limit: i64,
    pub auto_approve: bool,
    pub approve_below: u8,
    pub manual_at: u8,
    pub check_duplicates: bool,
    pub check_structure: bool,
    pub check_sensitive: bool,
    pub check_images: bool,
    pub require_ai: bool,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            revision: 0,
            enabled: false,
            daily_limit: 20,
            auto_approve: false,
            approve_below: 30,
            manual_at: 70,
            check_duplicates: true,
            check_structure: true,
            check_sensitive: true,
            check_images: true,
            require_ai: true,
        }
    }
}
impl Policy {
    fn valid(&self) -> bool {
        self.revision >= 0
            && (1..=1000).contains(&self.daily_limit)
            && self.approve_below > 0
            && self.approve_below < self.manual_at
            && self.manual_at <= 100
    }
}
impl Pg {
    pub async fn init_moderation(&self) -> Result<(), sqlx::Error> {
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY CHECK(id=1),data JSONB NOT NULL)",
            self.t("moderation_policy")
        ))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!(
            "INSERT INTO {} (id,data) VALUES (1,$1) ON CONFLICT DO NOTHING",
            self.t("moderation_policy")
        ))
        .bind(json!(Policy::default()))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!(
            "ALTER TABLE {} ADD COLUMN IF NOT EXISTS moderation JSONB",
            self.t("publications")
        ))
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn moderate_publication(
        &self,
        publication: &Publication,
        token: &str,
    ) -> Result<Publication, StatusCode> {
        let author = publication
            .author_email
            .as_deref()
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        self.check_publishing(&mut tx).await?;
        actor_lock(self, &mut tx, author, token, false).await?;
        let data: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1 FOR SHARE",
            self.t("moderation_policy")
        ))
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        let config: Policy =
            serde_json::from_value(data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let mut result = publication.clone();
        if !config.enabled {
            self.insert_publication_in(&mut tx, publication).await?;
            tx.commit().await.map_err(db_error)?;
            return Ok(result);
        }
        let count:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {} WHERE author_email=$1 AND created_at>=now()-interval '24 hours'",self.t("publications"))).bind(author).fetch_one(&mut *tx).await.map_err(db_error)?;
        if count >= config.daily_limit {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        let data: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1 FOR SHARE",
            self.t("safety_rules")
        ))
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        let rules: Rules =
            serde_json::from_value(data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        // All publication inserts use this lock, so concurrent authors cannot both pass duplicate checks.
        self.catalog_lock(&mut tx).await?;
        let duplicate: bool = if config.check_duplicates {
            sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE kind=$1 AND btrim(COALESCE(content,''))=$2 AND members=$3)",self.t("publications"))).bind(&publication.kind).bind(publication.content.as_deref().unwrap_or("").trim()).bind(json!(publication.members)).fetch_one(&mut *tx).await.map_err(db_error)?
        } else {
            false
        };
        let screen = screen(&config, &rules, publication, duplicate);
        if screen["decision"] == "approved" {
            result.status = "approved".into();
        }
        self.insert_publication_in(&mut tx, &result).await?;
        sqlx::query(&format!(
            "UPDATE {} SET moderation=$2 WHERE id=$1",
            self.t("publications")
        ))
        .bind(&result.id)
        .bind(&screen)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        if result.status == "pending" && (config.require_ai || (config.check_images && !publication.asset_refs.is_empty())) {
            sqlx::query(&format!("INSERT INTO {} (publication_id) VALUES ($1) ON CONFLICT DO NOTHING",self.t("ai_jobs"))).bind(&result.id).execute(&mut *tx).await.map_err(db_error)?;
        }
        if result.status == "approved" {
            let item = result
                .square_item()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            sqlx::query(&format!("INSERT INTO {} (id,title,kind,excerpt,model,member_count,content,category_id,members,sort_index) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,COALESCE((SELECT max(sort_index)+1 FROM {0}),0))",self.t("square_items"))).bind(&item.id).bind(&item.title).bind(&item.kind).bind(&item.excerpt).bind(&item.model).bind(item.member_count).bind(&item.content).bind(&item.category_id).bind(json!(item.members)).execute(&mut *tx).await.map_err(db_error)?;
            sqlx::query(&format!("INSERT INTO {} (publication_id,actor_email,status,reason) VALUES ($1,'system:local-rules','approved',$2)",self.t("review_events"))).bind(&result.id).bind("按站点策略由本地规则自动通过（不是 AI 安全保证）").execute(&mut *tx).await.map_err(db_error)?;
        }
        audit(
            self,
            &mut tx,
            "system:local-rules",
            "publication_auto_screened",
            json!({"publication_id":result.id,"result":screen}),
        )
        .await?;
        tx.commit().await.map_err(db_error)?;
        Ok(result)
    }
}
fn balanced_variables(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut opened = false;
    while i + 1 < bytes.len() {
        match &bytes[i..i + 2] {
            b"{{" => {
                if opened {
                    return false;
                }
                opened = true;
                i += 2;
            }
            b"}}" => {
                if !opened {
                    return false;
                }
                opened = false;
                i += 2;
            }
            _ => i += 1,
        }
    }
    !opened
}
fn screen(config: &Policy, rules: &Rules, publication: &Publication, duplicate: bool) -> Value {
    let mut parts = vec![
        publication.title.as_deref().unwrap_or(""),
        publication.content.as_deref().unwrap_or(""),
    ];
    for member in &publication.members {
        parts.push(&member.title);
        parts.push(&member.content);
    }
    let text = parts.join("\n");
    let mut reasons = Vec::<String>::new();
    if !publication.asset_refs.is_empty() { reasons.push("稿件含文件附件，必须人工查看，文本审核不能替代文件审核".into()); }
    let mut score = 0u64;
    let oversized = text.len() > 200_000;
    let usable = publication.square_item().is_some()
        && (publication.kind != "prompt"
            || !publication
                .content
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty());
    let structure = usable && !oversized && parts.iter().all(|p| balanced_variables(p));
    // Missing usable content is always manual, even if the optional structure policy is disabled.
    if !usable || oversized || (config.check_structure && !structure) {
        reasons.push("快照不完整、正文过长或变量标记不配对，需人工核查".into());
        score = 100;
    }
    if config.check_duplicates && duplicate {
        reasons.push("与已提交公开快照正文或合集成员完全重复".into());
        score = score.max(80);
    }
    let local = if config.check_sensitive && !oversized {
        rules.evaluate(&text, None)
    } else {
        json!({"revision":rules.revision,"hits":[],"score":0,"source":"local_rules"})
    };
    score = score.max(local["score"].as_u64().unwrap_or(0));
    if config.check_sensitive && local["hits"].as_array().is_some_and(|h| !h.is_empty()) {
        reasons.push("命中已启用安全规则，需人工核查".into());
    }
    let image =
        text.contains("![") || text.to_lowercase().contains("<img") || text.contains("data:image/");
    if config.check_images && image {
        reasons.push("检测到图片引用，图片安全服务尚未配置，转人工".into());
    }
    if score >= u64::from(config.manual_at) {
        reasons.push("达到高风险转人工阈值".into());
    }
    let local_reasons = reasons.clone();
    if config.require_ai {
        reasons.push("策略要求 AI 审核，等待模型结果；未配置或失败时转人工".into());
    }
    let approved =
        config.auto_approve && score < u64::from(config.approve_below) && reasons.is_empty();
    if !approved && reasons.is_empty() {
        reasons.push("按站点策略进入人工审核".into());
    }
    json!({"source":"local_rules","policy_revision":config.revision,"rules_revision":rules.revision,"score":score,"decision":if approved{"approved"}else{"manual"},"reasons":reasons,"local_reasons":local_reasons,"rules":local,"checks":{"duplicates":config.check_duplicates,"structure":config.check_structure,"sensitive":config.check_sensitive,"images":config.check_images,"require_ai":config.require_ai},"notice":"本地规则初筛，不是 AI 安全保证"})
}
pub async fn get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(
        sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1",
            pg.t("moderation_policy")
        ))
        .fetch_one(&pg.pool)
        .await
        .map_err(db_error)?,
    ))
}
pub async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut input): Json<Policy>,
) -> Result<Json<Policy>, StatusCode> {
    let actor = require_configuration_admin(&state, &headers).await?;
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
    let role: String = sqlx::query_scalar(&format!(
        "SELECT role FROM {} WHERE email=$1",
        pg.t("accounts")
    ))
    .bind(&actor)
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;
    if role != "owner" {
        return Err(StatusCode::FORBIDDEN);
    }
    let before: Value = sqlx::query_scalar(&format!(
        "SELECT data FROM {} WHERE id=1 FOR UPDATE",
        pg.t("moderation_policy")
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
        pg.t("moderation_policy")
    ))
    .bind(json!(input))
    .execute(&mut *tx)
    .await
    .map_err(db_error)?;
    audit(
        pg,
        &mut tx,
        &actor,
        "moderation_policy_updated",
        json!({"before":before,"after":input}),
    )
    .await?;
    tx.commit().await.map_err(db_error)?;
    Ok(Json(input))
}
