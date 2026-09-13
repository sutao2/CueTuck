use crate::{bearer_token, postgres::Pg, require_staff, AppState};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;

fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReviewQuery {
    q: String,
    author: String,
    status: Option<String>,
    from: String,
    to: String,
    offset: i64,
    limit: Option<i64>,
}

fn valid_date(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    if value.len() != 10
        || value.as_bytes()[4] != b'-'
        || value.as_bytes()[7] != b'-'
        || !value
            .bytes()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return false;
    }
    let year: u32 = value[..4].parse().unwrap_or(0);
    let month: usize = value[5..7].parse().unwrap_or(0);
    let day: u32 = value[8..].parse().unwrap_or(0);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        0,
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    year > 0 && (1..=12).contains(&month) && day > 0 && day <= days[month]
}
impl ReviewQuery {
    fn validate(&self) -> Result<(), StatusCode> {
        if self.q.len() > 300
            || self.author.len() > 254
            || !matches!(
                self.status.as_deref().unwrap_or("pending"),
                "" | "pending" | "approved" | "rejected"
            )
            || self.offset < 0
            || !(1..=100).contains(&self.limit.unwrap_or(25))
            || !valid_date(&self.from)
            || !valid_date(&self.to)
            || (!self.from.is_empty() && !self.to.is_empty() && self.from > self.to)
        {
            return Err(StatusCode::BAD_REQUEST);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectReason {
    pub reason: String,
}
pub fn validate_reason(reason: &str) -> Result<&str, StatusCode> {
    let value = reason.trim();
    if !(1..=1000).contains(&value.chars().count()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(value)
}

impl Pg {
    pub async fn review_list(&self, query: &ReviewQuery) -> Result<Value, StatusCode> {
        query.validate()?;
        let sql = format!("WITH filtered AS (SELECT * FROM {} WHERE ($1='' OR strpos(lower(COALESCE(title,'') || ' ' || source_id),lower($1))>0) AND ($2='' OR strpos(lower(COALESCE(author_email,'')),lower($2))>0) AND ($3='' OR status=$3) AND ($4='' OR (created_at AT TIME ZONE 'UTC')::date>=NULLIF($4,'')::date) AND ($5='' OR (created_at AT TIME ZONE 'UTC')::date<=NULLIF($5,'')::date)), page AS (SELECT * FROM filtered ORDER BY created_at DESC NULLS LAST,id LIMIT $6 OFFSET $7) SELECT json_build_object('items',COALESCE((SELECT json_agg(to_jsonb(p) || jsonb_build_object('history',COALESCE((SELECT jsonb_agg(to_jsonb(e) ORDER BY e.id) FROM {} e WHERE e.publication_id=p.id),'[]'::jsonb)) ORDER BY p.created_at DESC NULLS LAST,p.id) FROM page p),'[]'::json),'total',(SELECT count(*) FROM filtered),'offset',$7::bigint,'limit',$6::bigint)", self.t("publications"), self.t("review_events"));
        sqlx::query_scalar(&sql)
            .bind(query.q.trim())
            .bind(query.author.trim())
            .bind(query.status.as_deref().unwrap_or("pending"))
            .bind(&query.from)
            .bind(&query.to)
            .bind(query.limit.unwrap_or(25))
            .bind(query.offset)
            .fetch_one(&self.pool)
            .await
            .map_err(db_error)
    }

    pub async fn author_reviews(&self, email: &str) -> Result<Value, StatusCode> {
        // The author sees the decision, never the moderator's private email.
        sqlx::query_scalar(&format!("SELECT json_build_object('items',COALESCE(json_agg(to_jsonb(p) || jsonb_build_object('visibility',s.visibility,'download_count',COALESCE(s.download_count,0),'favorite_count',(SELECT count(*) FROM {} f WHERE f.item_id=p.id),'history',COALESCE((SELECT jsonb_agg(to_jsonb(e) - 'actor_email' ORDER BY e.id) FROM {} e WHERE e.publication_id=p.id),'[]'::jsonb)) ORDER BY p.created_at DESC NULLS LAST,p.id),'[]'::json)) FROM {} p LEFT JOIN {} s ON s.id=p.id WHERE p.author_email=$1", self.t("favorites"), self.t("review_events"), self.t("publications"), self.t("square_items")))
            .bind(email).fetch_one(&self.pool).await.map_err(db_error)
    }
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReviewQuery>,
) -> Result<Json<Value>, StatusCode> {
    require_staff(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(pg.review_list(&query).await?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchReview {
    ids: Vec<String>,
    status: String,
    reason: Option<String>,
}
pub async fn batch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<BatchReview>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_staff(&state, &headers).await?;
    let token = bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    if body.ids.is_empty()
        || body.ids.len() > 50
        || body.ids.iter().any(|id| id.is_empty() || id.len() > 200)
        || body.ids.iter().collect::<HashSet<_>>().len() != body.ids.len()
        || !matches!(body.status.as_str(), "approved" | "rejected")
        || (body.status == "approved" && body.reason.is_some())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let reason = if body.status == "rejected" {
        Some(validate_reason(body.reason.as_deref().unwrap_or(""))?)
    } else {
        None
    };
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut results = Vec::new();
    for id in &body.ids {
        let result = pg
            .review_publication(id, &body.status, reason, Some((&actor, &token)))
            .await;
        results.push(match result {
            Ok(_) => json!({"id":id,"ok":true,"status":200}),
            Err(status) => json!({"id":id,"ok":false,"status":status.as_u16(),"message":match status { StatusCode::CONFLICT => "已存在其他审核结果", StatusCode::NOT_FOUND => "投稿不存在", StatusCode::UNAUTHORIZED => "会话已失效", StatusCode::FORBIDDEN => "审核权限已撤销", _ => "审核失败，请刷新确认后重试" }}),
        });
    }
    Ok(Json(json!({"results":results})))
}
