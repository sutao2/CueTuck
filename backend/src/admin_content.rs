use crate::{bearer_token, postgres::Pg, require_admin, AppState};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;

fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
fn visibility(value: &str) -> bool {
    matches!(value, "online" | "offline" | "trashed")
}
pub const CATEGORIES: &[(&str, &str, &[&str])] = &[
    (
        "cat-software",
        "软件开发",
        &["网站开发", "前端工程", "后端与数据库", "测试与审查"],
    ),
    (
        "cat-image",
        "图片生成",
        &["人像摄影", "商品视觉", "插画与海报"],
    ),
    ("cat-video", "视频创作", &["分镜脚本", "短视频"]),
    (
        "cat-office",
        "办公效率",
        &["PPT 制作", "数据表格", "会议与邮件"],
    ),
    ("cat-writing", "内容写作", &["社交媒体", "长文写作", "SEO"]),
    (
        "cat-product",
        "产品设计",
        &["PRD 与需求", "竞品分析", "用户研究"],
    ),
    (
        "cat-marketing",
        "市场营销",
        &["品牌与广告", "增长运营", "销售话术"],
    ),
    (
        "cat-data",
        "数据分析",
        &["SQL 与清洗", "业务洞察", "可视化"],
    ),
    (
        "cat-education",
        "教育学习",
        &["课程与教案", "私人导师", "论文与研究"],
    ),
    (
        "cat-life",
        "生活助手",
        &["旅行规划", "饮食与健身", "求职成长"],
    ),
];
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContentQuery {
    q: String,
    visibility: String,
    offset: i64,
    limit: Option<i64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentEdit {
    revision: i64,
    excerpt: String,
    category_id: Option<String>,
    model: Option<String>,
    recommended: bool,
    sort_index: i32,
    visibility: String,
    reason: Option<String>,
}
impl Pg {
    pub async fn should_seed_square(&self) -> Result<bool, StatusCode> {
        sqlx::query_scalar(&format!(
            "SELECT NOT EXISTS(SELECT 1 FROM {}) AND NOT EXISTS(SELECT 1 FROM {} WHERE key='square_seed_disabled' AND value='true')",
            self.t("square_items"), self.t("settings")
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)
    }
    #[cfg(test)]
    pub async fn has_square_records(&self) -> Result<bool, StatusCode> {
        sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM {})",
            self.t("square_items")
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)
    }
    async fn content_list(&self, query: &ContentQuery) -> Result<Value, StatusCode> {
        if query.q.len() > 300
            || (!query.visibility.is_empty() && !visibility(&query.visibility))
            || query.offset < 0
            || !(1..=100).contains(&query.limit.unwrap_or(25))
        {
            return Err(StatusCode::BAD_REQUEST);
        }
        let mut result: Value = sqlx::query_scalar(&format!("WITH filtered AS (SELECT id,title,kind,excerpt,category_id,model,visibility,revision,recommended,sort_index,download_count,listed_at FROM {} WHERE ($1='' OR strpos(lower(title || ' ' || id),lower($1))>0) AND ($2='' OR visibility=$2)) SELECT json_build_object('items',COALESCE((SELECT json_agg(p) FROM (SELECT * FROM filtered ORDER BY sort_index,id LIMIT $3 OFFSET $4) p),'[]'::json),'total',(SELECT count(*) FROM filtered),'offset',$4::bigint,'limit',$3::bigint)", self.t("square_items")))
            .bind(query.q.trim()).bind(&query.visibility).bind(query.limit.unwrap_or(25)).bind(query.offset).fetch_one(&self.pool).await.map_err(db_error)?;
        let categories = self.catalog_entries("categories",true).await?;
        result["categories"] = json!(categories.iter().map(|entry| {
            let name = categories.iter().find(|parent| Some(&parent.id)==entry.parent_id.as_ref()).map(|parent|format!("{} / {}",parent.name,entry.name)).unwrap_or_else(||entry.name.clone());
            json!({"id":entry.id,"name":name})
        }).collect::<Vec<_>>());
        result["models"] = json!(self.catalog_entries("models",true).await?);
        Ok(result)
    }
    async fn content_detail(&self, id: &str) -> Result<Value, StatusCode> {
        sqlx::query_scalar(&format!("SELECT to_jsonb(s) || jsonb_build_object('history',COALESCE((SELECT jsonb_agg(jsonb_build_object('id',a.id,'actor_email',a.actor_email,'created_at',a.created_at,'details',a.details) ORDER BY a.created_at DESC,a.id) FROM {} a WHERE a.action='content_updated' AND a.details->>'item_id'=s.id),'[]'::jsonb)) FROM {} s WHERE s.id=$1", self.t("security_audit"), self.t("square_items")))
            .bind(id).fetch_optional(&self.pool).await.map_err(db_error)?.ok_or(StatusCode::NOT_FOUND)
    }
    pub async fn edit_content(
        &self,
        actor: &str,
        token: &str,
        id: &str,
        input: &ContentEdit,
    ) -> Result<i64, StatusCode> {
        if input.revision < 0
            || input.excerpt.chars().count() > 2000
            || input
                .model
                .as_deref()
                .is_some_and(|v| v.trim().is_empty() || v.chars().count() > 100)
            || !visibility(&input.visibility)
            || input.sort_index.abs_diff(0) > 1_000_000
            || input
                .reason
                .as_deref()
                .is_some_and(|v| v.chars().count() > 1000)
        {
            return Err(StatusCode::BAD_REQUEST);
        }
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        let role: Option<String> = sqlx::query_scalar(&format!("SELECT role FROM {} a WHERE email=$1 AND NOT disabled AND EXISTS(SELECT 1 FROM {} WHERE email=a.email AND token=$2) FOR SHARE", self.t("accounts"), self.t("access_tokens"))).bind(actor).bind(token).fetch_optional(&mut *tx).await.map_err(db_error)?;
        let role = role.ok_or(StatusCode::UNAUTHORIZED)?;
        if !matches!(role.as_str(), "owner" | "admin") {
            return Err(StatusCode::FORBIDDEN);
        }
        self.catalog_lock(&mut tx).await?;
        let row = sqlx::query(&format!("SELECT revision,visibility,excerpt,category_id,model,recommended,sort_index FROM {} WHERE id=$1 FOR UPDATE", self.t("square_items"))).bind(id).fetch_optional(&mut *tx).await.map_err(db_error)?.ok_or(StatusCode::NOT_FOUND)?;
        if row.get::<i64, _>("revision") != input.revision {
            return Err(StatusCode::CONFLICT);
        }
        // Existing disabled values can be retained when changing unrelated metadata.
        let old_category: Option<String> = row.get("category_id");
        let old_model: Option<String> = row.get("model");
        self.validate_catalog_refs(&mut tx,
            if old_category==input.category_id {None} else {input.category_id.as_deref()},
            if old_model==input.model {None} else {input.model.as_deref()}).await?;
        let before = json!({"visibility":row.get::<String,_>("visibility"),"excerpt":row.get::<Option<String>,_>("excerpt"),"category_id":row.get::<Option<String>,_>("category_id"),"model":row.get::<Option<String>,_>("model"),"recommended":row.get::<bool,_>("recommended"),"sort_index":row.get::<i32,_>("sort_index")});
        let previous: String = row.get("visibility");
        if previous == "trashed" && input.visibility == "online" {
            return Err(StatusCode::BAD_REQUEST);
        }
        let reason = input.reason.as_deref().unwrap_or("").trim();
        if previous != input.visibility && reason.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
        let revision: i64 = sqlx::query_scalar(&format!("UPDATE {} SET excerpt=$2,category_id=$3,model=$4,recommended=$5,sort_index=$6,visibility=$7,revision=revision+1 WHERE id=$1 RETURNING revision", self.t("square_items")))
            .bind(id).bind(input.excerpt.trim()).bind(&input.category_id).bind(input.model.as_deref().map(str::trim)).bind(input.recommended).bind(input.sort_index).bind(&input.visibility).fetch_one(&mut *tx).await.map_err(db_error)?;
        sqlx::query(&format!("INSERT INTO {} (id,actor_email,action,details) VALUES ($1,$2,'content_updated',$3)", self.t("security_audit")))
            .bind(uuid::Uuid::new_v4().to_string()).bind(actor).bind(json!({"item_id":id,"revision":revision,"reason":reason,"before":before,"after":{"visibility":input.visibility,"excerpt":input.excerpt.trim(),"category_id":input.category_id,"model":input.model.as_deref().map(str::trim),"recommended":input.recommended,"sort_index":input.sort_index}})).execute(&mut *tx).await.map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(revision)
    }
}
pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ContentQuery>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&state, &headers).await?;
    Ok(Json(
        state
            .db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
            .content_list(&query)
            .await?,
    ))
}
pub async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&state, &headers).await?;
    Ok(Json(
        state
            .db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
            .content_detail(&id)
            .await?,
    ))
}
pub async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<ContentEdit>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    let token = bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let revision = state
        .db
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
        .edit_content(&actor, &token, &id, &input)
        .await?;
    Ok(Json(json!({"updated":true,"revision":revision})))
}
