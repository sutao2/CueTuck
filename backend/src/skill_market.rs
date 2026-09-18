use crate::{
    admin_risk::{actor_lock, audit},
    bearer_token,
    postgres::Pg,
    require_staff, require_user,
    skill_bundle::Bundle,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
fn db(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
const CATEGORIES: &[&str] = &[
    "development",
    "ai",
    "cloud",
    "security",
    "office",
    "design",
    "marketing",
    "science",
    "tools",
    "uncategorized",
];
impl Pg {
    pub async fn init_skill_market(&self) -> Result<(), sqlx::Error> {
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, author_email TEXT NOT NULL REFERENCES {}(email), request_id TEXT NOT NULL, title TEXT NOT NULL, description TEXT NOT NULL, category TEXT NOT NULL, license TEXT NOT NULL, bundle JSONB NOT NULL, digest TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','approved','rejected','withdrawn')), reason TEXT, history JSONB NOT NULL DEFAULT '[]', created_at TIMESTAMPTZ NOT NULL DEFAULT now(), UNIQUE(author_email,request_id))",self.t("skill_publications"),self.t("accounts"))).execute(&self.pool).await?;
        sqlx::query(&format!(
            "CREATE INDEX IF NOT EXISTS {} ON {} (status,category,created_at DESC,id)",
            "skill_publications_browse",
            self.t("skill_publications")
        ))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS moderation JSONB", self.t("skill_publications"))).execute(&self.pool).await?;
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submission {
    request_id: String,
    title: String,
    description: String,
    category: String,
    license: String,
    confirm_public: bool,
    bundle: Bundle,
}
pub async fn submit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Submission>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_user(&state, &headers).await?;
    let token = bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    if uuid::Uuid::parse_str(&body.request_id).is_err()
        || !body.confirm_public
        || body.title.trim().is_empty()
        || body.title.chars().count() > 120
        || body.description.trim().is_empty()
        || body.description.chars().count() > 2000
        || body.license.trim().is_empty()
        || body.license.chars().count() > 120
        || !CATEGORIES.contains(&body.category.as_str())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    body.bundle
        .validate()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db)?;
    let role = actor_lock(pg, &mut tx, &actor, &token, false).await?;
    let existing:Option<(String,Value,String,String,String,String)>=sqlx::query_as(&format!("SELECT id,bundle,title,description,category,license FROM {} WHERE author_email=$1 AND request_id=$2",pg.t("skill_publications"))).bind(&actor).bind(&body.request_id).fetch_optional(&mut *tx).await.map_err(db)?;
    if let Some((id, bundle, title, description, category, license)) = existing {
        if bundle != json!(body.bundle)
            || title != body.title.trim()
            || description != body.description.trim()
            || category != body.category
            || license != body.license.trim()
        {
            return Err(StatusCode::CONFLICT);
        }
        tx.commit().await.map_err(db)?;
        return detail(State(state), headers, Path(id)).await;
    }
    pg.check_publishing(&mut tx).await?;
    let profile: Option<String> = sqlx::query_scalar(&format!(
        "SELECT display_name FROM {} WHERE email=$1",
        pg.t("accounts")
    ))
    .bind(&actor)
    .fetch_one(&mut *tx)
    .await
    .map_err(db)?;
    if profile.as_deref().unwrap_or("").trim().is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    if !matches!(role.as_str(), "admin" | "owner") {
        let daily: i64 = sqlx::query_scalar(&format!(
            "SELECT COALESCE((data->>'daily_limit')::bigint,20) FROM {} WHERE id=1",
            pg.t("moderation_policy")
        ))
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?;
        let count:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {} WHERE author_email=$1 AND created_at>now()-interval '24 hours'",pg.t("skill_publications"))).bind(&actor).fetch_one(&mut *tx).await.map_err(db)?;
        if count >= daily {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    let digest = body.bundle.digest();
    sqlx::query(&format!("INSERT INTO {} (id,author_email,request_id,title,description,category,license,bundle,digest) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",pg.t("skill_publications"))).bind(&id).bind(&actor).bind(&body.request_id).bind(body.title.trim()).bind(body.description.trim()).bind(&body.category).bind(body.license.trim()).bind(json!(body.bundle)).bind(digest).execute(&mut *tx).await.map_err(db)?;
    let policy: Value = sqlx::query_scalar(&format!("SELECT data FROM {} WHERE id=1 FOR SHARE", pg.t("moderation_policy"))).fetch_one(&mut *tx).await.map_err(db)?;
    if policy["enabled"] == true && policy["ai_decides"] == true {
        sqlx::query(&format!("UPDATE {} SET moderation=$2 WHERE id=$1",pg.t("skill_publications"))).bind(&id).bind(json!({"policy_revision":policy["revision"],"decision":"queued"})).execute(&mut *tx).await.map_err(db)?;
        sqlx::query(&format!("INSERT INTO {} (publication_id) VALUES ($1)",pg.t("skill_ai_jobs"))).bind(&id).execute(&mut *tx).await.map_err(db)?;
    }
    audit(pg, &mut tx, &actor, "skill_submitted", json!({"id":id})).await?;
    tx.commit().await.map_err(db)?;
    detail(State(state), headers, Path(id)).await
}
#[derive(Deserialize, Default)]
pub struct Browse {
    #[serde(default)]
    q: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    offset: i64,
    #[serde(default)]
    status: String,
}
fn summary() -> &'static str {
    "jsonb_build_object('id',p.id,'title',p.title,'description',p.description,'category',p.category,'license',p.license,'name',p.bundle->>'name','digest',p.digest,'status',p.status,'reason',p.reason,'created_at',p.created_at,'publisher',jsonb_build_object('display_name',COALESCE(NULLIF(a.display_name,''),'社区作者')))"
}
async fn listing(
    state: &AppState,
    q: Browse,
    mode: &str,
    actor: &str,
) -> Result<Json<Value>, StatusCode> {
    if q.offset < 0
        || q.offset > 100000
        || q.q.len() > 500
        || (!q.category.is_empty() && !CATEGORIES.contains(&q.category.as_str()))
        || (!q.status.is_empty()
            && !matches!(
                q.status.as_str(),
                "pending" | "approved" | "rejected" | "withdrawn"
            ))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let result:Value=sqlx::query_scalar(&format!("WITH available AS (SELECT p.*,a.display_name FROM {} p JOIN {} a ON a.email=p.author_email WHERE ($1='admin' OR ($1='mine' AND p.author_email=$2) OR ($1='public' AND p.status='approved' AND NOT a.disabled))), filtered AS (SELECT * FROM available WHERE ($3='' OR category=$3) AND ($4='' OR strpos(lower(title||' '||description||' '||(bundle->>'name')),lower($4))>0) AND ($6='' OR status=$6)), page AS (SELECT * FROM filtered ORDER BY created_at DESC,id LIMIT 24 OFFSET $5) SELECT jsonb_build_object('items',COALESCE((SELECT jsonb_agg({} ORDER BY p.created_at DESC,p.id) FROM page p JOIN {} a ON a.email=p.author_email),'[]'::jsonb),'total',(SELECT count(*) FROM filtered),'offset',$5::bigint,'category_counts',COALESCE((SELECT jsonb_object_agg(category,n) FROM (SELECT category,count(*) n FROM available GROUP BY category) c),'{{}}'::jsonb))",pg.t("skill_publications"),pg.t("accounts"),summary(),pg.t("accounts"))).bind(mode).bind(actor).bind(&q.category).bind(q.q.trim()).bind(q.offset).bind(&q.status).fetch_one(&pg.pool).await.map_err(db)?;
    Ok(Json(result))
}
pub async fn browse(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<Browse>,
) -> Result<Json<Value>, StatusCode> {
    if !state.square_public().await? {
        require_user(&state, &headers).await?;
    }
    listing(&state, q, "public", "").await
}
pub async fn mine(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<Browse>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_user(&state, &headers).await?;
    listing(&state, q, "mine", &actor).await
}
pub async fn admin_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<Browse>,
) -> Result<Json<Value>, StatusCode> {
    require_staff(&state, &headers).await?;
    listing(&state, q, "admin", "").await
}
async fn visible(state: &AppState, headers: &HeaderMap, id: &str) -> Result<Value, StatusCode> {
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let row=sqlx::query(&format!("SELECT {},p.author_email,p.bundle,p.history, a.disabled FROM {} p JOIN {} a ON a.email=p.author_email WHERE p.id=$1",summary(),pg.t("skill_publications"),pg.t("accounts"))).bind(id).fetch_optional(&pg.pool).await.map_err(db)?.ok_or(StatusCode::NOT_FOUND)?;
    let mut value: Value = row.get(0);
    let author: String = row.get(1);
    let disabled: bool = row.get(4);
    let public = value["status"] == "approved" && !disabled && state.square_public().await?;
    if !public {
        let actor = require_user(state, headers).await?;
        let staff = crate::admin_users::staff(&state.role_of(&actor).await?);
        if actor != author && !staff && !(value["status"] == "approved" && !disabled) {
            return Err(StatusCode::NOT_FOUND);
        }
    }
    value["bundle"] = row.get(2);
    value["history"] = row.get(3);
    Ok(value)
}
pub async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let mut value = visible(&state, &headers, &id).await?;
    let bundle: Bundle = serde_json::from_value(value["bundle"].take())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let decoded = bundle
        .validate()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    value.as_object_mut().unwrap().remove("bundle");
    value["files"] = json!(bundle
        .files
        .iter()
        .zip(&decoded)
        .map(|(f, b)| json!({"path":f.path,"size":b.len(),"executable":f.executable}))
        .collect::<Vec<_>>());
    value["body"] = json!(bundle
        .files
        .iter()
        .zip(&decoded)
        .find(|(f, _)| f.path == "SKILL.md")
        .map(|(_, b)| String::from_utf8_lossy(b).to_string()));
    Ok(Json(value))
}
pub async fn package(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let value = visible(&state, &headers, &id).await?;
    Ok(Json(
        json!({"bundle":value["bundle"],"digest":value["digest"]}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    status: String,
    #[serde(default)]
    reason: String,
    expected_status: String,
}
async fn transition(
    state: AppState,
    headers: HeaderMap,
    id: String,
    body: Decision,
    owner: bool,
) -> Result<Json<Value>, StatusCode> {
    let actor = if owner {
        require_user(&state, &headers).await?
    } else {
        require_staff(&state, &headers).await?
    };
    let token = bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    if body.reason.chars().count() > 2000
        || (!owner && !matches!(body.status.as_str(), "approved" | "rejected" | "withdrawn"))
        || (!owner && body.status != "approved" && body.reason.trim().is_empty())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pg.pool.begin().await.map_err(db)?;
    let role = actor_lock(pg, &mut tx, &actor, &token, false).await?;
    if !owner && !crate::admin_users::staff(&role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let (author, current): (String, String) = sqlx::query_as(&format!(
        "SELECT author_email,status FROM {} WHERE id=$1 FOR UPDATE",
        pg.t("skill_publications")
    ))
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db)?
    .ok_or(StatusCode::NOT_FOUND)?;
    if owner && author != actor {
        return Err(StatusCode::NOT_FOUND);
    }
    if current != body.expected_status {
        return Err(StatusCode::CONFLICT);
    }
    let status = if owner {
        "withdrawn"
    } else {
        body.status.as_str()
    };
    if current == "withdrawn"
        || (!owner && matches!(status, "approved" | "rejected") && current != "pending")
    {
        return Err(StatusCode::CONFLICT);
    }
    let event = json!({"status":status,"reason":body.reason.trim(),"at":chrono::Utc::now()});
    sqlx::query(&format!(
        "UPDATE {} SET status=$2,reason=$3,history=history||$4::jsonb WHERE id=$1",
        pg.t("skill_publications")
    ))
    .bind(&id)
    .bind(status)
    .bind(body.reason.trim())
    .bind(json!([event]))
    .execute(&mut *tx)
    .await
    .map_err(db)?;
    audit(
        pg,
        &mut tx,
        &actor,
        "skill_reviewed",
        json!({"id":id,"status":status}),
    )
    .await?;
    tx.commit().await.map_err(db)?;
    Ok(Json(json!({"id":id,"status":status})))
}
pub async fn review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Decision>,
) -> Result<Json<Value>, StatusCode> {
    transition(state, headers, id, body, false).await
}
pub async fn withdraw(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Decision>,
) -> Result<Json<Value>, StatusCode> {
    transition(state, headers, id, body, true).await
}
pub async fn admin_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    require_staff(&state, &headers).await?;
    detail(State(state), headers, Path(id)).await
}
pub async fn admin_package(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    require_staff(&state, &headers).await?;
    package(State(state), headers, Path(id)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_security_tests::{request, state};
    use base64::{engine::general_purpose::STANDARD, Engine};
    async fn account(s: &AppState, email: &str, role: &str) -> String {
        let pg = s.db.as_ref().unwrap();
        pg.upsert_account(email, Some("test-password"), role)
            .await
            .unwrap();
        pg.put_profile(email, Some("公开昵称"), None).await.unwrap();
        s.issue_session(email.into()).await.unwrap().access_token
    }
    fn submission() -> Value {
        json!({"request_id":uuid::Uuid::new_v4().to_string(),"title":"代码审查","description":"审阅变更","category":"development","license":"MIT","confirm_public":true,"bundle":{"name":"code-review","files":[{"path":"SKILL.md","content":STANDARD.encode("---\nname: code-review\n---\n# Review"),"executable":false},{"path":"scripts/check.sh","content":STANDARD.encode("echo test"),"executable":true}]}})
    }
    #[tokio::test]
    async fn publication_review_install_visibility_and_idempotency() {
        let s = state().await;
        let author = account(&s, "author@test.local", "user").await;
        let other = account(&s, "other@test.local", "user").await;
        let reviewer = account(&s, "reviewer@test.local", "reviewer").await;
        let body = submission();
        assert_eq!(
            request(&s, "POST", "/v1/skills", "", body.clone()).await.0,
            StatusCode::UNAUTHORIZED
        );
        let (status, created) = request(&s, "POST", "/v1/skills", &author, body.clone()).await;
        assert_eq!(status, StatusCode::OK, "{created}");
        let id = created["id"].as_str().unwrap();
        assert_eq!(created["status"], "pending");
        assert!(!created.to_string().contains("author@test.local"));
        assert_eq!(created["files"].as_array().unwrap().len(), 2);
        assert_eq!(
            request(&s, "POST", "/v1/skills", &author, body.clone())
                .await
                .1["id"],
            id
        );
        let mut different = body.clone();
        different["title"] = json!("changed");
        assert_eq!(
            request(&s, "POST", "/v1/skills", &author, different)
                .await
                .0,
            StatusCode::CONFLICT
        );
        assert_eq!(
            request(&s, "GET", "/v1/skills", "", json!({})).await.1["total"],
            0
        );
        assert_eq!(
            request(
                &s,
                "GET",
                &format!("/v1/skills/{id}/bundle"),
                &other,
                json!({})
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            request(&s, "GET", "/v1/skills/mine", &author, json!({}))
                .await
                .1["total"],
            1
        );
        assert_eq!(
            request(&s, "GET", "/v1/admin/skills", &author, json!({}))
                .await
                .0,
            StatusCode::FORBIDDEN
        );
        let decision = json!({"status":"approved","reason":"Checked","expected_status":"pending"});
        assert_eq!(
            request(
                &s,
                "POST",
                &format!("/v1/admin/skills/{id}/review"),
                &reviewer,
                decision.clone()
            )
            .await
            .0,
            StatusCode::OK
        );
        assert_eq!(
            request(
                &s,
                "POST",
                &format!("/v1/admin/skills/{id}/review"),
                &reviewer,
                decision
            )
            .await
            .0,
            StatusCode::CONFLICT
        );
        let (_, list) = request(
            &s,
            "GET",
            "/v1/skills?category=development&q=代码",
            "",
            json!({}),
        )
        .await;
        assert_eq!(list["total"], 1);
        assert_eq!(list["category_counts"]["development"], 1);
        assert!(list["items"][0].get("bundle").is_none());
        let (_, package) =
            request(&s, "GET", &format!("/v1/skills/{id}/bundle"), "", json!({})).await;
        let bundle: Bundle = serde_json::from_value(package["bundle"].clone()).unwrap();
        assert_eq!(bundle.digest(), package["digest"].as_str().unwrap());
        assert_eq!(bundle.validate().unwrap()[1], b"echo test");
        let withdraw =
            json!({"status":"withdrawn","reason":"作者撤回","expected_status":"approved"});
        assert_eq!(
            request(
                &s,
                "POST",
                &format!("/v1/skills/{id}/withdraw"),
                &other,
                withdraw.clone()
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            request(
                &s,
                "POST",
                &format!("/v1/skills/{id}/withdraw"),
                &author,
                withdraw
            )
            .await
            .0,
            StatusCode::OK
        );
        assert_ne!(
            request(&s, "GET", &format!("/v1/skills/{id}/bundle"), "", json!({}))
                .await
                .0,
            StatusCode::OK
        );
    }
    #[tokio::test]
    async fn invalid_upload_and_nickname_are_rejected() {
        let s = state().await;
        let author = account(&s, "author@test.local", "user").await;
        let mut body = submission();
        body["bundle"]["files"][1]["path"] = json!("../escape");
        assert_eq!(
            request(&s, "POST", "/v1/skills", &author, body).await.0,
            StatusCode::BAD_REQUEST
        );
        s.db.as_ref()
            .unwrap()
            .put_profile("author@test.local", None, None)
            .await
            .unwrap();
        assert_eq!(
            request(&s, "POST", "/v1/skills", &author, submission())
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
}
