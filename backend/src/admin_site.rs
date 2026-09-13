use crate::{
    admin_risk::{actor_lock, audit},
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
fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    pub revision: i64,
    pub name: String,
    pub description: String,
    pub logo_url: String,
    pub support_email: String,
    pub publishing_open: bool,
    pub announcement: String,
    pub announcement_start: Option<chrono::DateTime<chrono::Utc>>,
    pub announcement_end: Option<chrono::DateTime<chrono::Utc>>,
}
impl Default for Site {
    fn default() -> Self {
        Self {
            revision: 0,
            name: "唤词".into(),
            description: "分享好用的提示词，从社区创作者的实践中寻找灵感。".into(),
            logo_url: String::new(),
            support_email: String::new(),
            publishing_open: true,
            announcement: String::new(),
            announcement_start: None,
            announcement_end: None,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(flatten)]
    pub site: Site,
    pub square_public: bool,
}
impl Site {
    fn validate(&mut self) -> Result<(), StatusCode> {
        self.name = self.name.trim().into();
        self.logo_url = self.logo_url.trim().into();
        self.support_email = self.support_email.trim().into();
        if self.name.is_empty()
            || self.name.chars().count() > 60
            || self.description.chars().count() > 300
            || self.announcement.chars().count() > 2000
            || self.logo_url.len() > 2048
            || (!self.support_email.is_empty()
                && !crate::mail_transport::address(&self.support_email))
            || matches!((self.announcement_start,self.announcement_end),(Some(a),Some(b)) if a>=b)
        {
            return Err(StatusCode::BAD_REQUEST);
        }
        if !self.logo_url.is_empty() {
            let url = reqwest::Url::parse(&self.logo_url).map_err(|_| StatusCode::BAD_REQUEST)?;
            if url.scheme() != "https"
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        Ok(())
    }
    pub fn public(mut self, now: chrono::DateTime<chrono::Utc>) -> Self {
        if self.announcement_start.is_some_and(|v| v > now)
            || self.announcement_end.is_some_and(|v| v <= now)
        {
            self.announcement.clear();
        }
        self
    }
}
impl Pg {
    pub async fn init_site(&self) -> Result<(), sqlx::Error> {
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY CHECK(id=1),data JSONB NOT NULL)",
            self.t("site_configuration")
        ))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!(
            "INSERT INTO {} (id,data) VALUES (1,$1) ON CONFLICT DO NOTHING",
            self.t("site_configuration")
        ))
        .bind(json!(Site::default()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn site(&self) -> Result<Site, StatusCode> {
        let data: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1",
            self.t("site_configuration")
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;
        serde_json::from_value(data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
    pub async fn check_publishing(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), StatusCode> {
        let enabled: bool = sqlx::query_scalar(&format!(
            "SELECT (data->>'publishing_open')::boolean FROM {} WHERE id=1 FOR SHARE",
            self.t("site_configuration")
        ))
        .fetch_one(&mut **tx)
        .await
        .map_err(db_error)?;
        if enabled {
            Ok(())
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
    pub async fn save_site(
        &self,
        actor: &str,
        token: &str,
        input: Option<Configuration>,
        square_public: bool,
    ) -> Result<Configuration, StatusCode> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        let data: Value = sqlx::query_scalar(&format!(
            "SELECT data FROM {} WHERE id=1 FOR UPDATE",
            self.t("site_configuration")
        ))
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        let before: Site =
            serde_json::from_value(data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        actor_lock(self, &mut tx, actor, token, true).await?;
        let role: String = sqlx::query_scalar(&format!(
            "SELECT role FROM {} WHERE email=$1",
            self.t("accounts")
        ))
        .bind(actor)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        if role != "owner" && self.has_owner().await? {
            return Err(StatusCode::FORBIDDEN);
        }
        let mut site = if let Some(input) = input {
            if input.site.revision != before.revision {
                return Err(StatusCode::CONFLICT);
            }
            input.site
        } else {
            before.clone()
        };
        site.validate()?;
        site.revision = before.revision + 1;
        let old_public = self.square_public().await?;
        sqlx::query(&format!(
            "UPDATE {} SET data=$1 WHERE id=1",
            self.t("site_configuration")
        ))
        .bind(json!(site))
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        sqlx::query(&format!("INSERT INTO {} (key,value) VALUES ('square_public',$1) ON CONFLICT(key) DO UPDATE SET value=EXCLUDED.value",self.t("settings"))).bind(square_public.to_string()).execute(&mut *tx).await.map_err(db_error)?;
        audit(self,&mut tx,actor,"site_configuration_saved",json!({"before":before,"after":site,"square_public_before":old_public,"square_public_after":square_public})).await?;
        tx.commit().await.map_err(db_error)?;
        Ok(Configuration {
            site,
            square_public,
        })
    }
}
pub async fn get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Configuration>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(Configuration {
        site: pg.site().await?,
        square_public: pg.square_public().await?,
    }))
}
pub async fn put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Configuration>,
) -> Result<Json<Configuration>, StatusCode> {
    let actor = require_configuration_admin(&state, &headers).await?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let public = input.square_public;
    Ok(Json(
        pg.save_site(
            &actor,
            &bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?,
            Some(input),
            public,
        )
        .await?,
    ))
}
pub async fn public(State(state): State<AppState>) -> Result<Json<Site>, StatusCode> {
    Ok(Json(
        match &state.db {
            Some(pg) => pg.site().await?,
            None => Site::default(),
        }
        .public(chrono::Utc::now()),
    ))
}
