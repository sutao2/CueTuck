use crate::{bearer_token, postgres::Pg, require_admin, AppState};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Postgres, Row, Transaction};

fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub icon: String,
    pub color: String,
    pub vendor: String,
    pub region: String,
    pub group: String,
    pub enabled: bool,
    pub sort_index: i32,
    pub revision: i64,
}
impl Default for Entry {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            parent_id: None,
            icon: "folder".into(),
            color: "#728080".into(),
            vendor: String::new(),
            region: String::new(),
            group: "language".into(),
            enabled: true,
            sort_index: 0,
            revision: 0,
        }
    }
}
pub fn seed_categories() -> Vec<Entry> {
    crate::admin_content::CATEGORIES
        .iter()
        .enumerate()
        .flat_map(|(order, (id, name, children))| {
            let mut rows = vec![Entry {
                id: id.to_string(),
                name: name.to_string(),
                sort_index: order as i32,
                ..Entry::default()
            }];
            rows.extend(children.iter().enumerate().map(|(index, name)| Entry {
                id: format!("{id}-{index}"),
                name: name.to_string(),
                parent_id: Some(id.to_string()),
                sort_index: index as i32,
                ..Entry::default()
            }));
            rows
        })
        .collect()
}
fn seed_models() -> Vec<Entry> {
    [
        ("GPT", "OpenAI", "language"),
        ("ChatGPT", "OpenAI", "language"),
        ("Claude", "Anthropic", "language"),
        ("Gemini", "Google", "language"),
        ("DeepSeek", "DeepSeek", "language"),
        ("Qwen", "Alibaba", "language"),
        ("Flux", "Black Forest Labs", "image"),
        ("Midjourney", "Midjourney", "image"),
        ("Stable Diffusion", "Stability AI", "image"),
        ("DALL-E", "OpenAI", "image"),
        ("Sora", "OpenAI", "video"),
        ("Veo", "Google", "video"),
        ("Kling", "Kuaishou", "video"),
        ("GPT-5", "OpenAI", "language"),
        ("DeepSeek V4", "DeepSeek", "language"),
        ("Doubao", "ByteDance", "language"),
        ("Kimi", "Moonshot", "language"),
        ("GLM", "Zhipu", "language"),
        ("GPT Image", "OpenAI", "image"),
        ("Nano Banana", "Google", "image"),
        ("Doubao Image", "ByteDance", "image"),
        ("Qwen Image", "Alibaba", "image"),
        ("Runway", "Runway", "video"),
        ("Hailuo", "MiniMax", "video"),
        ("Vidu", "Shengshu", "video"),
        ("Wan", "Alibaba", "video"),
    ]
    .iter()
    .enumerate()
    .map(|(i, (id, vendor, group))| Entry {
        id: id.to_string(),
        name: id.to_string(),
        vendor: vendor.to_string(),
        region: if matches!(
            *vendor,
            "DeepSeek"
                | "Alibaba"
                | "ByteDance"
                | "Moonshot"
                | "Zhipu"
                | "Kuaishou"
                | "MiniMax"
                | "Shengshu"
        ) {
            "国内"
        } else {
            "国外"
        }
        .into(),
        group: group.to_string(),
        sort_index: i as i32,
        ..Entry::default()
    })
    .collect()
}
fn valid_kind(kind: &str) -> Result<(), StatusCode> {
    if matches!(kind, "categories" | "models") {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

impl Pg {
    pub async fn seed_catalog(&self) -> Result<(), sqlx::Error> {
        sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (kind TEXT NOT NULL, id TEXT NOT NULL, data JSONB NOT NULL, deleted BOOLEAN NOT NULL DEFAULT FALSE, PRIMARY KEY(kind,id))",self.t("catalog"))).execute(&self.pool).await?;
        for (kind, entries) in [("categories", seed_categories()), ("models", seed_models())] {
            for entry in entries {
                sqlx::query(&format!(
                    "INSERT INTO {} (kind,id,data) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING",
                    self.t("catalog")
                ))
                .bind(kind)
                .bind(&entry.id)
                .bind(json!(entry))
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }
    pub async fn catalog_lock(&self, tx: &mut Transaction<'_, Postgres>) -> Result<(), StatusCode> {
        // All new references and dictionary changes share this transaction lock.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!("{}:catalog", self.schema))
            .execute(&mut **tx)
            .await
            .map_err(db_error)?;
        Ok(())
    }
    pub async fn catalog_entries(
        &self,
        kind: &str,
        active_only: bool,
    ) -> Result<Vec<Entry>, StatusCode> {
        let values: Vec<Value> = sqlx::query_scalar(&format!("SELECT c.data FROM {} c WHERE kind=$1 AND NOT deleted AND (NOT $2 OR ((c.data->>'enabled')::boolean AND (c.data->>'parent_id' IS NULL OR EXISTS(SELECT 1 FROM {} p WHERE p.kind='categories' AND p.id=c.data->>'parent_id' AND NOT p.deleted AND (p.data->>'enabled')::boolean)))) ORDER BY (c.data->>'sort_index')::int,c.id",self.t("catalog"),self.t("catalog"))).bind(kind).bind(active_only).fetch_all(&self.pool).await.map_err(db_error)?;
        values
            .into_iter()
            .map(|v| serde_json::from_value(v).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR))
            .collect()
    }
    pub async fn validate_catalog_refs(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        category: Option<&str>,
        model: Option<&str>,
    ) -> Result<(), StatusCode> {
        if let Some(id) = category {
            let valid: bool = sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} c WHERE kind='categories' AND id=$1 AND NOT deleted AND (data->>'enabled')::boolean AND (data->>'parent_id' IS NULL OR EXISTS(SELECT 1 FROM {} p WHERE p.kind='categories' AND p.id=c.data->>'parent_id' AND NOT p.deleted AND (p.data->>'enabled')::boolean)))",self.t("catalog"),self.t("catalog"))).bind(id).fetch_one(&mut **tx).await.map_err(db_error)?;
            if !valid {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        // Existing clients support free-form model names. Known disabled/deleted IDs are blocked.
        if let Some(id) = model {
            let blocked: bool = sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE kind='models' AND id=$1 AND (deleted OR NOT (data->>'enabled')::boolean))",self.t("catalog"))).bind(id).fetch_one(&mut **tx).await.map_err(db_error)?;
            if blocked {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        Ok(())
    }
    async fn catalog_references(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        kind: &str,
        id: &str,
    ) -> Result<i64, StatusCode> {
        let field = if kind == "categories" {
            "category_id"
        } else {
            "model"
        };
        let mut count:i64=sqlx::query_scalar(&format!("SELECT (SELECT count(*) FROM {} WHERE {field}=$1 OR EXISTS(SELECT 1 FROM jsonb_array_elements(members) m WHERE m->>'{field}'=$1)) + (SELECT count(*) FROM {} WHERE {field}=$1 OR EXISTS(SELECT 1 FROM jsonb_array_elements(members) m WHERE m->>'{field}'=$1))",self.t("square_items"),self.t("publications"))).bind(id).fetch_one(&mut **tx).await.map_err(db_error)?;
        count+=sqlx::query_scalar::<_,i64>(&format!("SELECT count(*) FROM {} WHERE kind=$1 AND target=$2",self.t("catalog_redirects"))).bind(kind).bind(id).fetch_one(&mut **tx).await.map_err(db_error)?;
        if kind == "categories" {
            count+=sqlx::query_scalar::<_,i64>(&format!("SELECT count(*) FROM (SELECT data FROM {} ORDER BY revision DESC LIMIT 1) c,jsonb_array_elements(c.data->'skills') s WHERE s->'categories' ? $1",self.t("ai_configuration"))).bind(id).fetch_one(&mut **tx).await.map_err(db_error)?;
        }
        Ok(count)
    }
    async fn catalog_list(&self, kind: &str) -> Result<Value, StatusCode> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        // Same lock gives a consistent list and reference snapshot.
        self.catalog_lock(&mut tx).await?;
        let values: Vec<Value> = sqlx::query_scalar(&format!("SELECT data FROM {} WHERE kind=$1 AND NOT deleted ORDER BY (data->>'sort_index')::int,id",self.t("catalog"))).bind(kind).fetch_all(&mut *tx).await.map_err(db_error)?;
        let mut rows = vec![];
        for mut value in values {
            value["references"] = json!(
                self.catalog_references(
                    &mut tx,
                    kind,
                    value["id"]
                        .as_str()
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
                )
                .await?
            );
            rows.push(value);
        }
        tx.commit().await.map_err(db_error)?;
        Ok(json!({"items":rows}))
    }
    async fn change_catalog(
        &self,
        actor: &str,
        token: &str,
        kind: &str,
        id: &str,
        input: Option<Entry>,
        revision: i64,
        create: bool,
    ) -> Result<Value, StatusCode> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        let role: Option<String> = sqlx::query_scalar(&format!("SELECT role FROM {} a WHERE email=$1 AND NOT disabled AND EXISTS(SELECT 1 FROM {} WHERE email=a.email AND token=$2) FOR SHARE",self.t("accounts"),self.t("access_tokens"))).bind(actor).bind(token).fetch_optional(&mut *tx).await.map_err(db_error)?;
        match role.as_deref() {
            Some("owner" | "admin") => {}
            None => return Err(StatusCode::UNAUTHORIZED),
            _ => return Err(StatusCode::FORBIDDEN),
        }
        self.catalog_lock(&mut tx).await?;
        let before = sqlx::query(&format!(
            "SELECT data,deleted FROM {} WHERE kind=$1 AND id=$2",
            self.t("catalog")
        ))
        .bind(kind)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?;
        if create && before.is_some() {
            return Err(StatusCode::CONFLICT);
        }
        if !create {
            let row = before
                .as_ref()
                .filter(|r| !r.get::<bool, _>("deleted"))
                .ok_or(StatusCode::NOT_FOUND)?;
            if row.get::<Value, _>("data")["revision"].as_i64() != Some(revision) {
                return Err(StatusCode::CONFLICT);
            }
        }
        let value = if let Some(mut entry) = input {
            entry.name = entry.name.trim().to_string();
            entry.vendor = entry.vendor.trim().to_string();
            if entry.id != id
                || id.is_empty()
                || id.chars().count() > 100
                || id.trim() != id
                || id
                    .chars()
                    .any(|c| c.is_control() || matches!(c, '/' | '\\'))
                || entry.name.is_empty()
                || entry.name.chars().count() > 80
                || entry.vendor.chars().count() > 100
                || !matches!(entry.region.as_str(), "" | "国内" | "国外")
                || entry.sort_index.abs_diff(0) > 1_000_000
                || entry.icon.len() > 40
                || !entry
                    .icon
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-')
                || entry.color.len() != 7
                || !entry.color.starts_with('#')
                || !entry.color[1..].chars().all(|c| c.is_ascii_hexdigit())
                || !matches!(entry.group.as_str(), "language" | "image" | "video")
            {
                return Err(StatusCode::BAD_REQUEST);
            }
            if kind == "models" && entry.parent_id.is_some() {
                return Err(StatusCode::BAD_REQUEST);
            }
            if let Some(parent) = &entry.parent_id {
                if parent == id {
                    return Err(StatusCode::BAD_REQUEST);
                }
                let valid: bool = sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE kind='categories' AND id=$1 AND NOT deleted AND data->>'parent_id' IS NULL)",self.t("catalog"))).bind(parent).fetch_one(&mut *tx).await.map_err(db_error)?;
                let children: bool = sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE kind='categories' AND NOT deleted AND data->>'parent_id'=$1)",self.t("catalog"))).bind(id).fetch_one(&mut *tx).await.map_err(db_error)?;
                if !valid || children {
                    return Err(StatusCode::BAD_REQUEST);
                }
            }
            let duplicate: bool = sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE kind=$1 AND id<>$2 AND NOT deleted AND lower(data->>'name')=lower($3) AND (data->>'parent_id') IS NOT DISTINCT FROM $4)",self.t("catalog"))).bind(kind).bind(id).bind(&entry.name).bind(&entry.parent_id).fetch_one(&mut *tx).await.map_err(db_error)?;
            if duplicate {
                return Err(StatusCode::CONFLICT);
            }
            entry.revision = if create { 0 } else { revision + 1 };
            let value = json!(entry);
            sqlx::query(&format!("INSERT INTO {} (kind,id,data) VALUES ($1,$2,$3) ON CONFLICT(kind,id) DO UPDATE SET data=EXCLUDED.data",self.t("catalog"))).bind(kind).bind(id).bind(&value).execute(&mut *tx).await.map_err(db_error)?;
            value
        } else {
            let children: bool = sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE kind='categories' AND NOT deleted AND data->>'parent_id'=$1)",self.t("catalog"))).bind(id).fetch_one(&mut *tx).await.map_err(db_error)?;
            if (kind == "categories" && children)
                || self.catalog_references(&mut tx, kind, id).await? > 0
            {
                return Err(StatusCode::CONFLICT);
            }
            sqlx::query(&format!(
                "UPDATE {} SET deleted=TRUE WHERE kind=$1 AND id=$2",
                self.t("catalog")
            ))
            .bind(kind)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
            json!({"deleted":true,"id":id})
        };
        sqlx::query(&format!("INSERT INTO {} (id,actor_email,action,details) VALUES ($1,$2,$3,$4)",self.t("security_audit"))).bind(uuid::Uuid::new_v4().to_string()).bind(actor).bind(if create {"catalog_created"} else if value["deleted"]==true {"catalog_deleted"} else {"catalog_updated"}).bind(json!({"kind":kind,"id":id,"before":before.map(|r|r.get::<Value,_>("data")),"after":value})).execute(&mut *tx).await.map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(value)
    }
}

pub async fn public(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let (categories, models) = if let Some(pg) = &state.db {
        (
            pg.catalog_entries("categories", true).await?,
            pg.catalog_entries("models", true).await?,
        )
    } else {
        (seed_categories(), seed_models())
    };
    let hierarchy = if let Some(pg) = &state.db {
        pg.catalog_entries("categories", false).await?
    } else {
        seed_categories()
    };
    let parents = hierarchy
        .into_iter()
        .map(|entry| (entry.id, json!(entry.parent_id)))
        .collect::<serde_json::Map<_, _>>();
    let site = match &state.db {
        Some(pg) => pg.site().await?,
        None => crate::admin_site::Site::default(),
    }
    .public(chrono::Utc::now());
    Ok(Json(
        json!({"categories":categories,"models":models,"category_parents":parents,"site":site}),
    ))
}
pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(kind): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&state, &headers).await?;
    valid_kind(&kind)?;
    Ok(Json(
        state
            .db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
            .catalog_list(&kind)
            .await?,
    ))
}
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(kind): Path<String>,
    Json(input): Json<Entry>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    valid_kind(&kind)?;
    let token = bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let id = input.id.clone();
    Ok(Json(
        state
            .db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
            .change_catalog(&actor, &token, &kind, &id, Some(input), 0, true)
            .await?,
    ))
}
pub async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
    Json(input): Json<Entry>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    valid_kind(&kind)?;
    let token = bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let revision = input.revision;
    Ok(Json(
        state
            .db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
            .change_catalog(&actor, &token, &kind, &id, Some(input), revision, false)
            .await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Delete {
    revision: i64,
}
pub async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
    Json(input): Json<Delete>,
) -> Result<Json<Value>, StatusCode> {
    let actor = require_admin(&state, &headers).await?;
    valid_kind(&kind)?;
    let token = bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(
        state
            .db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
            .change_catalog(&actor, &token, &kind, &id, None, input.revision, false)
            .await?,
    ))
}
