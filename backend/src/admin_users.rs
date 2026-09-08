use crate::{bearer_token, postgres::Pg, require_admin, verify_password, AppState};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

fn db_error(_: sqlx::Error) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
pub fn staff(role: &str) -> bool {
    matches!(role, "owner" | "admin" | "reviewer")
}
fn valid_role(role: &str) -> bool {
    staff(role) || role == "user"
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UserQuery {
    pub q: String,
    pub role: String,
    pub status: String,
    pub offset: i64,
    pub limit: Option<i64>,
}
impl UserQuery {
    fn validate(&self) -> Result<(), StatusCode> {
        if self.q.len() > 254
            || (!self.role.is_empty() && !valid_role(&self.role))
            || !matches!(self.status.as_str(), "" | "active" | "disabled")
            || self.offset < 0
            || !(1..=100).contains(&self.limit.unwrap_or(25))
        {
            return Err(StatusCode::BAD_REQUEST);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserAction {
    action: String,
    current_password: String,
    role: Option<String>,
    reason: Option<String>,
}
impl UserAction {
    fn validate(&self) -> Result<(), StatusCode> {
        if self.current_password.is_empty()
            || self.current_password.len() > 512
            || !matches!(
                self.action.as_str(),
                "disable" | "enable" | "revoke_sessions" | "role"
            )
            || (self.action == "role" && !self.role.as_deref().is_some_and(valid_role))
            || (self.action != "role" && self.role.is_some())
            || (matches!(self.action.as_str(), "disable" | "enable")
                && !self
                    .reason
                    .as_deref()
                    .is_some_and(|r| !r.trim().is_empty() && r.trim().chars().count() <= 1000))
            || self
                .reason
                .as_deref()
                .is_some_and(|r| r.chars().count() > 1000)
        {
            return Err(StatusCode::BAD_REQUEST);
        }
        Ok(())
    }
}

impl Pg {
    pub async fn migrate_owner(&self, email: Option<&str>) -> Result<(), String> {
        let Some(email) = email else { return Ok(()) };
        let mut tx = self.pool.begin().await.map_err(|_| "无法开始所有者迁移")?;
        sqlx::query(&format!(
            "LOCK TABLE {} IN SHARE ROW EXCLUSIVE MODE",
            self.t("accounts")
        ))
        .execute(&mut *tx)
        .await
        .map_err(|_| "无法锁定所有者迁移")?;
        let completed: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE key='owner_initialized')",
            self.t("settings")
        ))
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| "无法检查迁移状态")?;
        if completed {
            return Ok(());
        }
        let has_owner: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE role='owner')",
            self.t("accounts")
        ))
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| "无法检查所有者")?;
        if !has_owner {
            let changed = sqlx::query(&format!(
                "UPDATE {} SET role='owner' WHERE email=$1 AND role='admin' AND NOT disabled",
                self.t("accounts")
            ))
            .bind(email.trim())
            .execute(&mut *tx)
            .await
            .map_err(|_| "无法迁移所有者")?;
            if changed.rows_affected() != 1 {
                return Err(
                    "PROMPTARK_OWNER_EMAIL 必须指定已有且未停用的管理员，不会提升普通用户".into(),
                );
            }
            // A new role must be observed through a fresh session, not stale browser state.
            for table in ["access_tokens", "refresh_tokens"] {
                sqlx::query(&format!("DELETE FROM {} WHERE email=$1", self.t(table)))
                    .bind(email.trim())
                    .execute(&mut *tx)
                    .await
                    .map_err(|_| "无法撤销迁移账号会话")?;
            }
            sqlx::query(&format!("INSERT INTO {} (id,actor_email,action,target_email) VALUES ($1,'server-bootstrap','owner_migrated',$2)", self.t("security_audit")))
                .bind(Uuid::new_v4().to_string()).bind(email.trim()).execute(&mut *tx).await.map_err(|_| "无法记录迁移审计")?;
        }
        sqlx::query(&format!(
            "INSERT INTO {} (key,value) VALUES ('owner_initialized','true') ON CONFLICT DO NOTHING",
            self.t("settings")
        ))
        .execute(&mut *tx)
        .await
        .map_err(|_| "无法记录所有者迁移")?;
        tx.commit().await.map_err(|_| "无法完成所有者迁移".into())
    }

    pub async fn has_owner(&self) -> Result<bool, StatusCode> {
        sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {} WHERE role='owner') OR EXISTS(SELECT 1 FROM {} WHERE key='owner_initialized')", self.t("accounts"), self.t("settings")))
            .fetch_one(&self.pool).await.map_err(db_error)
    }

    pub async fn admin_user_list(&self, query: &UserQuery) -> Result<Value, StatusCode> {
        query.validate()?;
        let filter = "WHERE ($1='' OR strpos(lower(email),lower($1))>0 OR strpos(lower(COALESCE(display_name,'')),lower($1))>0) AND ($2='' OR role=$2) AND ($3='' OR disabled=($3='disabled'))";
        // One statement gives count and rows the same MVCC snapshot even during user changes.
        let sql = format!("WITH filtered AS (SELECT email,display_name,role,disabled FROM {} {}) SELECT json_build_object('items',COALESCE((SELECT json_agg(p) FROM (SELECT * FROM filtered ORDER BY email LIMIT $4 OFFSET $5) p),'[]'::json),'total',(SELECT count(*) FROM filtered),'offset',$5::bigint,'limit',$4::bigint)", self.t("accounts"), filter);
        sqlx::query_scalar(&sql)
            .bind(query.q.trim())
            .bind(&query.role)
            .bind(&query.status)
            .bind(query.limit.unwrap_or(25))
            .bind(query.offset)
            .fetch_one(&self.pool)
            .await
            .map_err(db_error)
    }

    pub async fn admin_user_detail(&self, email: &str) -> Result<Value, StatusCode> {
        let row = sqlx::query(&format!("SELECT email,display_name,bio,role,disabled,pro,(password_hash IS NOT NULL AND password_hash<>'') AS has_password,(SELECT count(*) FROM {} WHERE email=$1) AS active_access_count,(SELECT count(*) FROM {} WHERE author_email=$1) AS publication_count FROM {} WHERE email=$1", self.t("access_tokens"), self.t("publications"), self.t("accounts")))
            .bind(email).fetch_optional(&self.pool).await.map_err(db_error)?.ok_or(StatusCode::NOT_FOUND)?;
        let providers: Vec<String> = sqlx::query_scalar(&format!(
            "SELECT DISTINCT provider FROM {} WHERE email=$1 ORDER BY provider",
            self.t("oauth_accounts")
        ))
        .bind(email)
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(
            json!({"email":row.get::<String,_>("email"),"display_name":row.get::<Option<String>,_>("display_name"),"bio":row.get::<Option<String>,_>("bio"),"role":row.get::<String,_>("role"),"disabled":row.get::<bool,_>("disabled"),"pro":row.get::<bool,_>("pro"),"has_password":row.get::<bool,_>("has_password"),"providers":providers,"active_access_count":row.get::<i64,_>("active_access_count"),"publication_count":row.get::<i64,_>("publication_count")}),
        )
    }

    pub async fn admin_user_action(
        &self,
        actor: &str,
        token: &str,
        target: &str,
        input: &UserAction,
    ) -> Result<(), StatusCode> {
        input.validate()?;
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        self.identity_lock(&mut tx).await?;
        // All role/state writes use this order; session issuance uses account row locks.
        sqlx::query(&format!(
            "LOCK TABLE {} IN SHARE ROW EXCLUSIVE MODE",
            self.t("accounts")
        ))
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        let current = sqlx::query(&format!("SELECT role,password_hash FROM {} a WHERE email=$1 AND NOT disabled AND EXISTS(SELECT 1 FROM {} WHERE email=a.email AND token=$2) FOR UPDATE", self.t("accounts"), self.t("access_tokens")))
            .bind(actor).bind(token).fetch_optional(&mut *tx).await.map_err(db_error)?.ok_or(StatusCode::UNAUTHORIZED)?;
        let actor_role: String = current.get("role");
        if !matches!(actor_role.as_str(), "owner" | "admin") {
            return Err(StatusCode::FORBIDDEN);
        }
        if actor == target {
            return Err(StatusCode::CONFLICT);
        }
        let target_row = sqlx::query(&format!(
            "SELECT role,disabled FROM {} WHERE email=$1 FOR UPDATE",
            self.t("accounts")
        ))
        .bind(target)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?
        .ok_or(StatusCode::NOT_FOUND)?;
        let target_role: String = target_row.get("role");
        if actor_role != "owner" && (target_role != "user" || input.action == "role") {
            return Err(StatusCode::FORBIDDEN);
        }
        let hash: Option<String> = current.get("password_hash");
        if !hash.is_some_and(|h| verify_password(&input.current_password, &h)) {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
        if target_role == "owner"
            && !target_row.get::<bool, _>("disabled")
            && (input.action == "disable"
                || (input.action == "role" && input.role.as_deref() != Some("owner")))
        {
            let owners: i64 = sqlx::query_scalar(&format!(
                "SELECT count(*) FROM {} WHERE role='owner' AND NOT disabled",
                self.t("accounts")
            ))
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;
            if owners <= 1 {
                return Err(StatusCode::CONFLICT);
            }
        }
        match input.action.as_str() {
            "disable" | "enable" => {
                sqlx::query(&format!(
                    "UPDATE {} SET disabled=$2 WHERE email=$1",
                    self.t("accounts")
                ))
                .bind(target)
                .bind(input.action == "disable")
                .execute(&mut *tx)
                .await
                .map_err(db_error)?;
            }
            "role" => {
                sqlx::query(&format!(
                    "UPDATE {} SET role=$2 WHERE email=$1",
                    self.t("accounts")
                ))
                .bind(target)
                .bind(&input.role)
                .execute(&mut *tx)
                .await
                .map_err(db_error)?;
            }
            _ => {}
        }
        if input.action == "disable"
            || (target_role == "owner"
                && input.action == "role"
                && input.role.as_deref() != Some("owner"))
        {
            let clause="status='pending' AND (($2 AND lower(email)=lower($1)) OR (inviter=$1 AND purpose='invitation'))";
            sqlx::query(&format!("UPDATE {} SET status='expired',payload='',claim=NULL,revision=revision+1,updated_at=now() WHERE id IN (SELECT mail_id FROM {} WHERE {clause}) AND status IN ('queued','failed','sending')",self.t("mail_outbox"),self.t("identity_challenges"))).bind(target).bind(input.action=="disable").execute(&mut *tx).await.map_err(db_error)?;
            sqlx::query(&format!(
                "UPDATE {} SET status='revoked',revision=revision+1 WHERE {clause}",
                self.t("identity_challenges")
            ))
            .bind(target)
            .bind(input.action == "disable")
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        }
        if input.action != "enable" {
            for table in ["access_tokens", "refresh_tokens"] {
                sqlx::query(&format!("DELETE FROM {} WHERE email=$1", self.t(table)))
                    .bind(target)
                    .execute(&mut *tx)
                    .await
                    .map_err(db_error)?;
            }
        }
        sqlx::query(&format!("INSERT INTO {} (id,actor_email,action,target_email,details) VALUES ($1,$2,$3,$4,$5)", self.t("security_audit")))
            .bind(Uuid::new_v4().to_string()).bind(actor).bind(format!("user_{}", input.action)).bind(target)
            .bind(json!({"previous_role":target_role,"role":input.role,"previous_disabled":target_row.get::<bool,_>("disabled"),"reason":input.reason.as_deref().map(str::trim)}))
            .execute(&mut *tx).await.map_err(db_error)?;
        tx.commit().await.map_err(db_error)
    }
}

type ApiError = (StatusCode, Json<Value>);
fn api_error(status: StatusCode) -> ApiError {
    let message = match status {
        StatusCode::TOO_MANY_REQUESTS => "密码验证过于频繁，请一分钟后重试",
        StatusCode::FORBIDDEN => "无权管理该账号或修改角色",
        StatusCode::UNAUTHORIZED => "登录已失效，请重新登录",
        StatusCode::CONFLICT => "不能在此操作本人，也不能移除最后一位有效所有者",
        StatusCode::UNPROCESSABLE_ENTITY => "当前密码不正确，或操作者没有本地密码",
        StatusCode::NOT_FOUND => "用户不存在",
        StatusCode::BAD_REQUEST => "筛选或操作参数无效；停用/启用必须填写 1–1000 字操作原因",
        StatusCode::SERVICE_UNAVAILABLE => "用户管理需要数据库连接",
        _ => "用户管理操作未完成，请重试",
    };
    (status, Json(json!({"message":message})))
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<UserQuery>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&state, &headers).await.map_err(api_error)?;
    query.validate().map_err(api_error)?;
    if let Some(pg) = &state.db {
        return Ok(Json(pg.admin_user_list(&query).await.map_err(api_error)?));
    }
    let mut users = state.list_users().await.map_err(api_error)?;
    users.retain(|u| {
        u.email.to_lowercase().contains(&query.q.to_lowercase())
            && (query.role.is_empty() || u.role == query.role)
            && query.status != "disabled"
    });
    users.sort_by(|a, b| a.email.cmp(&b.email));
    let total = users.len();
    Ok(Json(
        json!({"total":total,"items":users.into_iter().skip(query.offset as usize).take(query.limit.unwrap_or(25) as usize).collect::<Vec<_>>()}),
    ))
}
pub async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(email): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&state, &headers).await.map_err(api_error)?;
    let pg = state
        .db
        .as_ref()
        .ok_or_else(|| api_error(StatusCode::SERVICE_UNAVAILABLE))?;
    let mut result = pg.admin_user_detail(&email).await.map_err(api_error)?;
    result["mock_pro"] = json!(state.billing_mock && pg.mock_pro(&email).await.map_err(api_error)?);
    Ok(Json(result))
}
pub async fn action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(email): Path<String>,
    Json(input): Json<UserAction>,
) -> Result<Json<Value>, ApiError> {
    let actor = require_admin(&state, &headers).await.map_err(api_error)?;
    state.limit_account_auth(&actor).await.map_err(api_error)?;
    let pg = state
        .db
        .as_ref()
        .ok_or_else(|| api_error(StatusCode::SERVICE_UNAVAILABLE))?;
    let token = bearer_token(&headers).ok_or_else(|| api_error(StatusCode::UNAUTHORIZED))?;
    pg.admin_user_action(&actor, &token, &email, &input)
        .await
        .map_err(api_error)?;
    Ok(Json(json!({"updated":true})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_security_tests::{request, state};

    async fn seed(state: &AppState, email: &str, role: &str) -> String {
        state
            .db
            .as_ref()
            .unwrap()
            .upsert_account(email, Some("account-password"), role)
            .await
            .unwrap();
        state
            .issue_session(email.into())
            .await
            .unwrap()
            .access_token
    }
    fn operation(action: &str) -> Value {
        json!({"action":action,"current_password":"account-password","reason":"测试运营操作"})
    }

    #[tokio::test]
    async fn user_state_changes_require_a_reason_and_preserve_it_atomically() {
        let state = state().await;
        let owner = seed(&state, "owner@example.com", "owner").await;
        let user = seed(&state, "user@example.com", "user").await;
        let pg = state.db.as_ref().unwrap();
        let path = "/v1/admin/users/user@example.com/actions";
        for reason in [Value::Null, json!("  "), json!("a".repeat(1001))] {
            assert_eq!(request(&state,"POST",path,&owner,json!({"action":"disable","current_password":"account-password","reason":reason})).await.0,StatusCode::BAD_REQUEST);
        }
        assert!(
            !pg.admin_user_detail("user@example.com").await.unwrap()["disabled"]
                .as_bool()
                .unwrap()
        );
        assert!(pg.email_for_access(&user).await.unwrap().is_some());
        assert_eq!(request(&state,"POST",path,&owner,json!({"action":"disable","current_password":"account-password","reason":"  违反规则  "})).await.0,StatusCode::OK);
        let reason: String = sqlx::query_scalar(&format!(
            "SELECT details->>'reason' FROM {} WHERE action='user_disable'",
            pg.t("security_audit")
        ))
        .fetch_one(&pg.pool)
        .await
        .unwrap();
        assert_eq!(reason, "违反规则");
        assert!(pg.email_for_access(&user).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn owner_migration_is_explicit_once_and_preserves_password() {
        let state = state().await;
        let pg = state.db.as_ref().unwrap();
        let admin = seed(&state, "admin@example.com", "admin").await;
        seed(&state, "other@example.com", "admin").await;
        seed(&state, "user@example.com", "user").await;
        pg.migrate_owner(None).await.unwrap();
        assert_eq!(pg.role_of("admin@example.com").await.unwrap(), "admin");
        assert!(pg.migrate_owner(Some("user@example.com")).await.is_err());
        pg.migrate_owner(Some("admin@example.com")).await.unwrap();
        assert_eq!(pg.role_of("admin@example.com").await.unwrap(), "owner");
        assert!(pg.email_for_access(&admin).await.unwrap().is_none());
        pg.password_session("admin@example.com", "account-password")
            .await
            .unwrap();
        pg.migrate_owner(Some("other@example.com")).await.unwrap();
        assert_eq!(pg.role_of("other@example.com").await.unwrap(), "admin");
        pg.upsert_account("admin@example.com", None, "user")
            .await
            .unwrap();
        pg.migrate_owner(Some("admin@example.com")).await.unwrap();
        assert_eq!(pg.role_of("admin@example.com").await.unwrap(), "user");
        assert!(pg.has_owner().await.unwrap()); // migration never reopens legacy admin configuration
    }

    #[tokio::test]
    async fn roles_enforced_on_api_and_identity_permissions() {
        let state = state().await;
        for role in ["owner", "admin", "reviewer", "user"] {
            seed(&state, &format!("{role}@example.com"), role).await;
        }
        for role in ["owner", "admin", "reviewer", "user"] {
            let token = state
                .issue_session(format!("{role}@example.com"))
                .await
                .unwrap()
                .access_token;
            let (code, identity) =
                request(&state, "GET", "/v1/admin/me", &token, json!(null)).await;
            assert_eq!(
                code,
                if role == "user" {
                    StatusCode::FORBIDDEN
                } else {
                    StatusCode::OK
                }
            );
            if role != "user" {
                assert_eq!(identity["permissions"]["configuration"], role == "owner");
            }
            for (path, permitted) in [
                ("/v1/admin/users", matches!(role, "owner" | "admin")),
                ("/v1/admin/oauth", role == "owner"),
                ("/v1/admin/settings", role == "owner"),
                ("/v1/admin/publications", role != "user"),
                ("/v1/admin/security", role != "user"),
                ("/v1/admin/overview", matches!(role, "owner" | "admin")),
                ("/v1/admin/content", matches!(role, "owner" | "admin")),
                (
                    "/v1/admin/catalog/categories",
                    matches!(role, "owner" | "admin"),
                ),
                (
                    "/v1/admin/catalog/models",
                    matches!(role, "owner" | "admin"),
                ),
                ("/v1/admin/reports", matches!(role, "owner" | "admin")),
                ("/v1/admin/safety-rules", matches!(role, "owner" | "admin")),
                (
                    "/v1/admin/mock-billing/entitlements",
                    matches!(role, "owner" | "admin"),
                ),
                ("/v1/admin/reviews", role != "user"),
                ("/v1/admin/site", role == "owner"),
                ("/v1/admin/moderation", role == "owner"),
                ("/v1/admin/ai/config", role == "owner"),
                ("/v1/admin/ai/history", role == "owner"),
                ("/v1/admin/mail/config", role == "owner"),
                ("/v1/admin/mail/deliveries", role == "owner"),
                ("/v1/admin/notifications/config", role == "owner"),
                ("/v1/admin/notifications/deliveries", role == "owner"),
                ("/v1/admin/identity/policy", role == "owner"),
                ("/v1/admin/identity/invitations", role == "owner"),
                ("/v1/admin/audit", role == "owner"),
                ("/v1/admin/system", role == "owner"),
            ] {
                assert_eq!(
                    request(&state, "GET", path, &token, json!(null)).await.0,
                    if permitted {
                        StatusCode::OK
                    } else {
                        StatusCode::FORBIDDEN
                    },
                    "{role}: {path}"
                );
            }
        }
        assert_eq!(
            request(&state, "GET", "/v1/admin/users", "", json!(null))
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn list_filters_paginates_and_detail_never_leaks_private_data() {
        let state = state().await;
        let token = seed(&state, "admin@example.com", "admin").await;
        let pg = state.db.as_ref().unwrap();
        for i in 0..31 {
            sqlx::query(&format!(
                "INSERT INTO {} (email,display_name,disabled) VALUES ($1,$2,$3)",
                pg.t("accounts")
            ))
            .bind(format!("user{i:02}@example.com"))
            .bind(format!("用户 {i}"))
            .bind(i == 30)
            .execute(&pg.pool)
            .await
            .unwrap();
        }
        let (_, first) = request(
            &state,
            "GET",
            "/v1/admin/users?role=user&limit=25&offset=0",
            &token,
            json!(null),
        )
        .await;
        assert_eq!(first["total"], 31);
        assert_eq!(first["items"].as_array().unwrap().len(), 25);
        let (_, next) = request(
            &state,
            "GET",
            "/v1/admin/users?role=user&offset=25&limit=25",
            &token,
            json!(null),
        )
        .await;
        assert_eq!(next["items"][0]["email"], "user25@example.com");
        let (_, disabled) = request(
            &state,
            "GET",
            "/v1/admin/users?status=disabled",
            &token,
            json!(null),
        )
        .await;
        assert_eq!(disabled["total"], 1);
        let (_, searched) = request(
            &state,
            "GET",
            "/v1/admin/users?q=user09",
            &token,
            json!(null),
        )
        .await;
        assert_eq!(searched["total"], 1);
        let (_, wildcard) =
            request(&state, "GET", "/v1/admin/users?q=%25", &token, json!(null)).await;
        assert_eq!(wildcard["total"], 0); // literal query, not SQL pattern
        for query in [
            "limit=0",
            "offset=-1",
            "role=superadmin",
            "status=other",
            "q=x&unexpected=1",
        ] {
            assert_eq!(
                request(
                    &state,
                    "GET",
                    &format!("/v1/admin/users?{query}"),
                    &token,
                    json!(null)
                )
                .await
                .0,
                StatusCode::BAD_REQUEST
            );
        }
        let (code, detail) = request(
            &state,
            "GET",
            "/v1/admin/users/admin%40example.com",
            &token,
            json!(null),
        )
        .await;
        assert_eq!(code, StatusCode::OK);
        assert_eq!(detail["has_password"], true);
        for key in [
            "password_hash",
            "password",
            "access_token",
            "refresh_token",
            "library_changes",
            "content",
        ] {
            assert!(detail.get(key).is_none());
        }
        assert_eq!(
            request(
                &state,
                "GET",
                "/v1/admin/users/missing",
                &token,
                json!(null)
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn disable_revokes_all_login_paths_and_enable_does_not_restore_tokens() {
        let state = state().await;
        let actor = seed(&state, "admin@example.com", "admin").await;
        let user = seed(&state, "user@example.com", "user").await;
        let old = state
            .issue_session("user@example.com".into())
            .await
            .unwrap();
        let pg = state.db.as_ref().unwrap();
        pg.link_oauth("google", "uid", "user@example.com")
            .await
            .unwrap();
        let path = "/v1/admin/users/user%40example.com/actions";
        assert_eq!(
            request(&state, "POST", path, &actor, operation("disable"))
                .await
                .0,
            StatusCode::OK
        );
        assert!(pg.email_for_access(&user).await.unwrap().is_none());
        assert_eq!(
            pg.password_session("user@example.com", "account-password")
                .await
                .err(),
            Some(StatusCode::UNAUTHORIZED)
        );
        assert_eq!(
            pg.refresh_session(&old.refresh_token).await.err(),
            Some(StatusCode::UNAUTHORIZED)
        );
        assert_eq!(
            pg.issue_session("user@example.com").await.err(),
            Some(StatusCode::UNAUTHORIZED)
        );
        let oauth = crate::oauth::OAuthUser {
            provider: "google".into(),
            provider_uid: "uid".into(),
            email: "user@example.com".into(),
        };
        assert_eq!(
            state.oauth_login(&oauth).await.err(),
            Some(StatusCode::UNAUTHORIZED)
        );
        assert_eq!(
            request(&state, "GET", "/v1/me", &old.access_token, json!(null))
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            request(&state, "POST", path, &actor, operation("enable"))
                .await
                .0,
            StatusCode::OK
        );
        assert!(pg
            .email_for_access(&old.access_token)
            .await
            .unwrap()
            .is_none());
        pg.password_session("user@example.com", "account-password")
            .await
            .unwrap();
        assert_eq!(
            request(&state, "POST", path, &actor, operation("revoke_sessions"))
                .await
                .0,
            StatusCode::OK
        );
        let count: i64 = sqlx::query_scalar(&format!(
            "SELECT count(*) FROM {} WHERE email='user@example.com'",
            pg.t("access_tokens")
        ))
        .fetch_one(&pg.pool)
        .await
        .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn user_actions_reauthenticate_and_fail_atomically_without_privilege_escalation() {
        let state = state().await;
        let owner = seed(&state, "owner@example.com", "owner").await;
        let admin = seed(&state, "admin@example.com", "admin").await;
        let user = seed(&state, "user@example.com", "user").await;
        let pg = state.db.as_ref().unwrap();
        let path = "/v1/admin/users/user%40example.com/actions";
        assert_eq!(
            request(&state, "POST", path, &user, operation("disable"))
                .await
                .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            request(
                &state,
                "POST",
                "/v1/admin/users/owner%40example.com/actions",
                &admin,
                operation("disable")
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            request(
                &state,
                "POST",
                "/v1/admin/users/owner%40example.com/actions",
                &owner,
                operation("disable")
            )
            .await
            .0,
            StatusCode::CONFLICT
        );
        assert_eq!(
            request(
                &state,
                "POST",
                path,
                &admin,
                json!({"action":"role","role":"admin","current_password":"account-password"})
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            request(
                &state,
                "POST",
                path,
                &owner,
                json!({"action":"role","role":"root","current_password":"account-password"})
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            request(
                &state,
                "POST",
                path,
                &owner,
                json!({"action":"disable","current_password":"wrong","reason":"测试"})
            )
            .await
            .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        sqlx::query(&format!(
            "ALTER TABLE {} ADD CONSTRAINT reject_user_audit CHECK(action NOT LIKE 'user_%')",
            pg.t("security_audit")
        ))
        .execute(&pg.pool)
        .await
        .unwrap();
        assert_eq!(
            request(&state, "POST", path, &owner, operation("disable"))
                .await
                .0,
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert!(pg.email_for_access(&user).await.unwrap().is_some());
        assert_eq!(
            pg.admin_user_detail("user@example.com").await.unwrap()["disabled"],
            false
        );
        sqlx::query(&format!(
            "ALTER TABLE {} DROP CONSTRAINT reject_user_audit",
            pg.t("security_audit")
        ))
        .execute(&pg.pool)
        .await
        .unwrap();
        assert_eq!(
            request(
                &state,
                "POST",
                path,
                &owner,
                json!({"action":"role","role":"reviewer","current_password":"account-password"})
            )
            .await
            .0,
            StatusCode::OK
        );
        assert!(pg.email_for_access(&user).await.unwrap().is_none());
        let audit: Value = sqlx::query_scalar(&format!(
            "SELECT row_to_json(a) FROM {} a",
            pg.t("security_audit")
        ))
        .fetch_one(&pg.pool)
        .await
        .unwrap();
        assert_eq!(audit["action"], "user_role");
        assert!(!audit.to_string().contains("account-password"));
    }

    #[tokio::test]
    async fn concurrent_owners_cannot_remove_each_other_and_leave_no_owner() {
        let state = state().await;
        let a = seed(&state, "a@example.com", "owner").await;
        let b = seed(&state, "b@example.com", "owner").await;
        let (first, second) = tokio::join!(
            request(
                &state,
                "POST",
                "/v1/admin/users/b%40example.com/actions",
                &a,
                operation("disable")
            ),
            request(
                &state,
                "POST",
                "/v1/admin/users/a%40example.com/actions",
                &b,
                operation("disable")
            )
        );
        assert!(first.0 == StatusCode::OK || second.0 == StatusCode::OK);
        assert!(!(first.0 == StatusCode::OK && second.0 == StatusCode::OK));
        let pg = state.db.as_ref().unwrap();
        let count: i64 = sqlx::query_scalar(&format!(
            "SELECT count(*) FROM {} WHERE role='owner' AND NOT disabled",
            pg.t("accounts")
        ))
        .fetch_one(&pg.pool)
        .await
        .unwrap();
        assert_eq!(count, 1);
    }
}
