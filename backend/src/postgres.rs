#[cfg(test)]
use crate::password::hash_password;
use crate::password::verify_password;
use crate::{AdminUser, Publication, SquareItem};
use axum::http::StatusCode;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Pg {
    pub pool: PgPool,
    pub schema: String,
}

impl Pg {
    pub fn new(pool: PgPool, schema: impl Into<String>) -> Result<Self, StatusCode> {
        let schema = schema.into();
        if schema.is_empty()
            || !schema
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Ok(Self { pool, schema })
    }

    pub(crate) fn t(&self, table: &str) -> String {
        format!("\"{}\".\"{}\"", self.schema, table)
    }

    pub async fn apply_schema(&self, reset: bool) -> Result<(), sqlx::Error> {
        sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", self.schema))
            .execute(&self.pool)
            .await?;
        if reset {
            for table in [
                "reports",
                "report_events",
                "safety_rules",
                "moderation_policy",
                "ai_configuration",
                "ai_tests",
                "mail_configuration",
                "mail_outbox",
                "mail_attempts",
                "identity_policy",
                "site_configuration",
                "mock_entitlements",
                "mock_orders",
                "mock_code_batches",
                "mock_codes",
                "mock_code_uses",
                "operations_metadata",
                "request_failures",
                "identity_challenges",
                "catalog",
                "catalog_redirects",
                "oauth_verifications",
                "notification_configuration",
                "notification_deliveries",
                "notification_attempts",
                "auth_limits",
                "security_audit",
                "favorites",
                "media_objects",
                "oauth_accounts",
                "access_tokens",
                "refresh_tokens",
                "review_events",
                "publications",
                "square_items",
                "settings",
                "library_changes",
                "redeem_codes",
                "accounts",
            ] {
                sqlx::query(&format!("DROP TABLE IF EXISTS {} CASCADE", self.t(table)))
                    .execute(&self.pool)
                    .await?;
            }
        }
        let statements = [
            format!("CREATE TABLE IF NOT EXISTS {} (bucket_key TEXT PRIMARY KEY, attempts INT NOT NULL, expires_at TIMESTAMPTZ NOT NULL)", self.t("auth_limits")),
            format!("CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, actor_email TEXT NOT NULL, action TEXT NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now())", self.t("security_audit")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS target_email TEXT", self.t("security_audit")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS details JSONB NOT NULL DEFAULT '{{}}'", self.t("security_audit")),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  email TEXT PRIMARY KEY,
                  password_hash TEXT,
                  role TEXT NOT NULL DEFAULT 'user',
                  display_name TEXT,
                  bio TEXT,
                  pro BOOLEAN NOT NULL DEFAULT FALSE
                )",
                self.t("accounts")
            ),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS disabled BOOLEAN NOT NULL DEFAULT FALSE", self.t("accounts")),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  provider TEXT NOT NULL,
                  provider_uid TEXT NOT NULL,
                  email TEXT NOT NULL REFERENCES {}(email) ON DELETE CASCADE,
                  PRIMARY KEY (provider, provider_uid)
                )",
                self.t("oauth_accounts"),
                self.t("accounts")
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  token TEXT PRIMARY KEY,
                  email TEXT NOT NULL REFERENCES {}(email) ON DELETE CASCADE,
                  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
                )",
                self.t("access_tokens"),
                self.t("accounts")
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  token TEXT PRIMARY KEY,
                  email TEXT NOT NULL REFERENCES {}(email) ON DELETE CASCADE,
                  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
                )",
                self.t("refresh_tokens"),
                self.t("accounts")
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  id TEXT PRIMARY KEY,
                  title TEXT NOT NULL,
                  kind TEXT NOT NULL,
                  excerpt TEXT,
                  model TEXT,
                  member_count BIGINT,
                  content TEXT,
                  download_count BIGINT NOT NULL DEFAULT 0,
                  sort_index INT NOT NULL DEFAULT 0
                )",
                self.t("square_items")
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  id TEXT PRIMARY KEY,
                  source_id TEXT NOT NULL,
                  status TEXT NOT NULL,
                  title TEXT,
                  content TEXT,
                  author_email TEXT
                )",
                self.t("publications")
            ),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS category_id TEXT", self.t("square_items")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS reference JSONB", self.t("square_items")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS category_id TEXT", self.t("publications")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS model TEXT", self.t("publications")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'prompt'", self.t("publications")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS members JSONB NOT NULL DEFAULT '[]'", self.t("publications")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ", self.t("publications")),
            format!("ALTER TABLE {} ALTER COLUMN created_at SET DEFAULT now()", self.t("publications")),
            format!("CREATE TABLE IF NOT EXISTS {} (id BIGSERIAL PRIMARY KEY, publication_id TEXT NOT NULL REFERENCES {}(id), actor_email TEXT NOT NULL, status TEXT NOT NULL, reason TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now())", self.t("review_events"), self.t("publications")),
            format!("CREATE INDEX IF NOT EXISTS review_events_publication ON {} (publication_id, id)", self.t("review_events")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS members JSONB NOT NULL DEFAULT '[]'", self.t("square_items")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS visibility TEXT NOT NULL DEFAULT 'online'", self.t("square_items")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS revision BIGINT NOT NULL DEFAULT 0", self.t("square_items")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS recommended BOOLEAN NOT NULL DEFAULT false", self.t("square_items")),
            format!("ALTER TABLE {} ADD COLUMN IF NOT EXISTS listed_at TIMESTAMPTZ", self.t("square_items")),
            format!("ALTER TABLE {} ALTER COLUMN listed_at SET DEFAULT now()", self.t("square_items")),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  email TEXT NOT NULL REFERENCES {}(email) ON DELETE CASCADE,
                  item_id TEXT NOT NULL REFERENCES {}(id) ON DELETE CASCADE,
                  PRIMARY KEY (email, item_id)
                )",
                self.t("favorites"),
                self.t("accounts"),
                self.t("square_items")
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  key TEXT PRIMARY KEY,
                  value TEXT NOT NULL
                )",
                self.t("settings")
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  id TEXT PRIMARY KEY,
                  owner_email TEXT NOT NULL REFERENCES {}(email) ON DELETE CASCADE,
                  object_key TEXT NOT NULL,
                  content_type TEXT,
                  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
                )",
                self.t("media_objects"),
                self.t("accounts")
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  owner_email TEXT NOT NULL REFERENCES {}(email) ON DELETE CASCADE,
                  id TEXT NOT NULL,
                  kind TEXT NOT NULL,
                  payload TEXT NOT NULL,
                  updated_at TEXT NOT NULL,
                  deleted_at TEXT,
                  PRIMARY KEY (owner_email, id)
                )",
                self.t("library_changes"),
                self.t("accounts")
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  code TEXT PRIMARY KEY,
                  used_by TEXT REFERENCES {}(email) ON DELETE SET NULL
                )",
                self.t("redeem_codes"),
                self.t("accounts")
            ),
        ];
        for sql in statements {
            sqlx::query(&sql).execute(&self.pool).await?;
        }
        sqlx::query(&format!(
            "ALTER TABLE {} ADD COLUMN IF NOT EXISTS author_email TEXT",
            self.t("publications")
        ))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!(
            "ALTER TABLE {} ADD COLUMN IF NOT EXISTS display_name TEXT",
            self.t("accounts")
        ))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!(
            "ALTER TABLE {} ADD COLUMN IF NOT EXISTS bio TEXT",
            self.t("accounts")
        ))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!(
            "ALTER TABLE {} ADD COLUMN IF NOT EXISTS pro BOOLEAN NOT NULL DEFAULT FALSE",
            self.t("accounts")
        ))
        .execute(&self.pool)
        .await?;
        sqlx::query(&format!(
            "ALTER TABLE {} ADD COLUMN IF NOT EXISTS download_count BIGINT NOT NULL DEFAULT 0",
            self.t("square_items")
        ))
        .execute(&self.pool)
        .await?;
        self.seed_catalog().await?;
        self.init_risk().await?;
        self.init_moderation().await?;
        self.init_ai().await?;
        self.init_mail().await?;
        self.init_identity().await?;
        self.init_site().await?;
        self.init_mock_billing().await?;
        self.init_operations().await?;
        self.init_catalog_redirects().await?;
        self.init_oauth_verification().await?;
        self.init_notifications().await?;
        Ok(())
    }

    #[cfg(test)]
    pub async fn upsert_account(
        &self,
        email: &str,
        password: Option<&str>,
        role: &str,
    ) -> Result<(), StatusCode> {
        let hash = password.map(hash_password);
        sqlx::query(&format!(
            "INSERT INTO {} (email, password_hash, role) VALUES ($1, $2, $3)
             ON CONFLICT (email) DO UPDATE SET
               password_hash = COALESCE(EXCLUDED.password_hash, {}.password_hash),
               role = EXCLUDED.role",
            self.t("accounts"),
            self.t("accounts")
        ))
        .bind(email)
        .bind(hash)
        .bind(role)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub async fn oauth_config(&self, provider: &str) -> Result<Option<String>, StatusCode> {
        sqlx::query_scalar(&format!(
            "SELECT value FROM {} WHERE key=$1",
            self.t("settings")
        ))
        .bind(format!("oauth_provider_{provider}"))
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    #[cfg(test)]
    pub async fn set_oauth_config(&self, provider: &str, value: &str) -> Result<(), StatusCode> {
        sqlx::query(&format!("INSERT INTO {} (key,value) VALUES ($1,$2) ON CONFLICT (key) DO UPDATE SET value=EXCLUDED.value", self.t("settings")))
            .bind(format!("oauth_provider_{provider}")).bind(value).execute(&self.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub async fn get_profile(&self, email: &str) -> Result<crate::me::MeProfile, StatusCode> {
        let row = sqlx::query(&format!(
            "SELECT email, display_name, bio FROM {} WHERE email = $1",
            self.t("accounts")
        ))
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(match row {
            Some(row) => crate::me::MeProfile {
                email: row.get("email"),
                display_name: row.get("display_name"),
                bio: row.get("bio"),
            },
            None => crate::me::MeProfile {
                email: email.into(),
                display_name: None,
                bio: None,
            },
        })
    }

    pub async fn put_profile(
        &self,
        email: &str,
        display_name: Option<&str>,
        bio: Option<&str>,
    ) -> Result<crate::me::MeProfile, StatusCode> {
        sqlx::query(&format!(
            "INSERT INTO {} (email, role, display_name, bio) VALUES ($1, 'user', $2, $3)
             ON CONFLICT (email) DO UPDATE SET
               display_name = EXCLUDED.display_name,
               bio = EXCLUDED.bio",
            self.t("accounts")
        ))
        .bind(email)
        .bind(display_name)
        .bind(bio)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        self.get_profile(email).await
    }

    fn library_change_from_row(
        row: &sqlx::postgres::PgRow,
    ) -> Result<crate::library::LibraryChange, StatusCode> {
        let payload: String = row.get("payload");
        Ok(crate::library::LibraryChange {
            id: row.get("id"),
            kind: row.get("kind"),
            payload: serde_json::from_str(&payload).unwrap_or(serde_json::Value::Null),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        })
    }

    pub async fn list_library_changes(
        &self,
        email: &str,
        since: &str,
    ) -> Result<Vec<crate::library::LibraryChange>, StatusCode> {
        let rows = sqlx::query(&format!(
            "SELECT id, kind, payload, updated_at, deleted_at FROM {}
             WHERE owner_email = $1
             ORDER BY id",
            self.t("library_changes")
        ))
        .bind(email)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let mut items = rows
            .iter()
            .map(Self::library_change_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        if !since.is_empty() {
            items.retain(|row| {
                crate::library::timestamp_ms(&row.updated_at) > crate::library::timestamp_ms(since)
            });
        }
        Ok(items)
    }

    pub async fn put_library_changes(
        &self,
        email: &str,
        items: &[crate::library::LibraryChange],
    ) -> Result<Vec<crate::library::LibraryChange>, StatusCode> {
        crate::library::validate_changes(items)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        for item in items {
            let payload = serde_json::to_string(&item.payload).unwrap_or_else(|_| "{}".into());
            sqlx::query(&format!(
                "INSERT INTO {0} (owner_email, id, kind, payload, updated_at, deleted_at)
                 VALUES ($1,$2,$3,$4,$5,$6)
                 ON CONFLICT (owner_email, id) DO UPDATE SET
                   kind = EXCLUDED.kind,
                   payload = EXCLUDED.payload,
                   updated_at = EXCLUDED.updated_at,
                   deleted_at = EXCLUDED.deleted_at
                 WHERE (CASE WHEN {0}.updated_at ~ '^[0-9]{{1,16}}$' THEN
                   CASE WHEN {0}.updated_at::numeric >= 1000000000 AND {0}.updated_at::numeric < 100000000000
                     THEN {0}.updated_at::numeric * 1000 ELSE {0}.updated_at::numeric END
                   ELSE 0 END) < EXCLUDED.updated_at::numeric",
                self.t("library_changes")
            ))
            .bind(email)
            .bind(&item.id)
            .bind(&item.kind)
            .bind(payload)
            .bind(crate::library::timestamp_ms(&item.updated_at).to_string())
            .bind(&item.deleted_at)
            .execute(&mut *transaction)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        self.list_library_changes(email, "").await
    }

    pub async fn verify_login(&self, email: &str, password: &str) -> Result<(), StatusCode> {
        let row = sqlx::query(&format!(
            "SELECT password_hash FROM {} WHERE email = $1",
            self.t("accounts")
        ))
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let Some(row) = row else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let hash: Option<String> = row.get("password_hash");
        let Some(hash) = hash else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        if verify_password(password, &hash) {
            Ok(())
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }

    pub async fn revoke_access(&self, token: &str) -> Result<bool, StatusCode> {
        let result = sqlx::query(&format!(
            "DELETE FROM {} WHERE token = $1",
            self.t("access_tokens")
        ))
        .bind(token)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn email_for_access(&self, token: &str) -> Result<Option<String>, StatusCode> {
        let row = sqlx::query(&format!(
            "SELECT a.email FROM {} t JOIN {} a ON a.email=t.email WHERE t.token=$1 AND NOT a.disabled",
            self.t("access_tokens"), self.t("accounts")
        ))
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(row.map(|row| row.get("email")))
    }

    pub async fn role_of(&self, email: &str) -> Result<String, StatusCode> {
        let row = sqlx::query(&format!(
            "SELECT role FROM {} WHERE email = $1",
            self.t("accounts")
        ))
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(row
            .map(|row| row.get("role"))
            .unwrap_or_else(|| "user".into()))
    }

    pub async fn list_users(&self) -> Result<Vec<AdminUser>, StatusCode> {
        let rows = sqlx::query(&format!(
            "SELECT email, role FROM {} ORDER BY email",
            self.t("accounts")
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(rows
            .into_iter()
            .map(|row| AdminUser {
                email: row.get("email"),
                role: row.get("role"),
            })
            .collect())
    }

    fn item_from_row(row: &sqlx::postgres::PgRow) -> SquareItem {
        SquareItem {
            reference: row.get("reference"),
            id: row.get("id"),
            title: row.get("title"),
            kind: row.get("kind"),
            excerpt: row.get("excerpt"),
            model: row.get("model"),
            category_id: row.get("category_id"),
            member_count: row.get("member_count"),
            content: row.get("content"),
            members: row
                .get::<sqlx::types::Json<Vec<crate::PublishedPrompt>>, _>("members")
                .0,
        }
    }

    pub async fn list_items(&self) -> Result<Vec<SquareItem>, StatusCode> {
        self.list_visible_items(false).await
    }

    pub async fn list_visible_items(&self, latest: bool) -> Result<Vec<SquareItem>, StatusCode> {
        let order = if latest {
            "listed_at DESC NULLS LAST, id"
        } else {
            "recommended DESC, sort_index, id"
        };
        let rows = sqlx::query(&format!(
            "SELECT id, title, kind, excerpt, model, member_count, content, category_id, members, reference
             FROM {} WHERE visibility='online' ORDER BY {order}",
            self.t("square_items")
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(rows.iter().map(Self::item_from_row).collect())
    }

    pub async fn get_item(&self, id: &str) -> Result<Option<SquareItem>, StatusCode> {
        let row = sqlx::query(&format!(
            "SELECT id, title, kind, excerpt, model, member_count, content, category_id, members, reference
             FROM {} WHERE id = $1 AND visibility='online'",
            self.t("square_items")
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(row.as_ref().map(Self::item_from_row))
    }

    pub async fn increment_download_count(&self, id: &str) -> Result<(), StatusCode> {
        let result = sqlx::query(&format!(
            "UPDATE {} SET download_count = download_count + 1 WHERE id = $1 AND visibility='online'",
            self.t("square_items")
        ))
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if result.rows_affected() == 0 {
            return Err(StatusCode::NOT_FOUND);
        }
        Ok(())
    }

    pub async fn download_counts(&self) -> Result<HashMap<String, i64>, StatusCode> {
        let rows = sqlx::query(&format!(
            "SELECT id, download_count FROM {}",
            self.t("square_items")
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(rows
            .iter()
            .map(|row| {
                (
                    row.get::<String, _>("id"),
                    row.get::<i64, _>("download_count"),
                )
            })
            .collect())
    }

    #[cfg(test)]
    pub async fn insert_item(&self, item: &SquareItem) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "INSERT INTO {} (id, title, kind, excerpt, model, member_count, content, category_id, members, reference, sort_index)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10, COALESCE((SELECT MAX(sort_index)+1 FROM {0}), 0))",
            self.t("square_items")
        ))
        .bind(&item.id)
        .bind(&item.title)
        .bind(&item.kind)
        .bind(&item.excerpt)
        .bind(&item.model)
        .bind(item.member_count)
        .bind(&item.content)
        .bind(&item.category_id)
        .bind(sqlx::types::Json(&item.members))
        .bind(&item.reference)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub async fn replace_items(&self, items: &[SquareItem]) -> Result<(), StatusCode> {
        sqlx::query(&format!("DELETE FROM {}", self.t("square_items")))
            .execute(&self.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        for (index, item) in items.iter().enumerate() {
            sqlx::query(&format!(
                "INSERT INTO {} (id, title, kind, excerpt, model, member_count, content, sort_index, category_id, members, reference)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
                self.t("square_items")
            ))
            .bind(&item.id)
            .bind(&item.title)
            .bind(&item.kind)
            .bind(&item.excerpt)
            .bind(&item.model)
            .bind(item.member_count)
            .bind(&item.content)
            .bind(index as i32)
            .bind(&item.category_id)
            .bind(sqlx::types::Json(&item.members))
            .bind(&item.reference)
            .execute(&self.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        Ok(())
    }

    pub async fn insert_publication(&self, publication: &Publication) -> Result<(), StatusCode> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        self.insert_publication_in(&mut tx, publication).await?;
        tx.commit()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub(crate) async fn insert_publication_in(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        publication: &Publication,
    ) -> Result<(), StatusCode> {
        self.catalog_lock(tx).await?;
        self.validate_catalog_refs(
            tx,
            publication.category_id.as_deref(),
            publication.model.as_deref(),
        )
        .await?;
        for member in &publication.members {
            self.validate_catalog_refs(tx, member.category_id.as_deref(), member.model.as_deref())
                .await?;
        }
        sqlx::query(&format!(
            "INSERT INTO {} (id, source_id, status, title, content, author_email, category_id, model, kind, members) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
            self.t("publications")
        ))
        .bind(&publication.id)
        .bind(&publication.source_id)
        .bind(&publication.status)
        .bind(&publication.title)
        .bind(&publication.content)
        .bind(&publication.author_email)
        .bind(&publication.category_id)
        .bind(&publication.model)
        .bind(&publication.kind)
        .bind(sqlx::types::Json(&publication.members))
        .execute(&mut **tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    fn publication_from_row(row: &sqlx::postgres::PgRow) -> Publication {
        Publication {
            id: row.get("id"),
            source_id: row.get("source_id"),
            status: row.get("status"),
            title: row.get("title"),
            content: row.get("content"),
            author_email: row.get("author_email"),
            category_id: row.get("category_id"),
            model: row.get("model"),
            kind: row.get("kind"),
            members: row
                .get::<sqlx::types::Json<Vec<crate::PublishedPrompt>>, _>("members")
                .0,
        }
    }

    pub async fn pending_publications(&self) -> Result<Vec<Publication>, StatusCode> {
        let rows = sqlx::query(&format!(
            "SELECT id, source_id, status, title, content, author_email, category_id, model, kind, members FROM {} WHERE status = 'pending'",
            self.t("publications")
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(rows.iter().map(Self::publication_from_row).collect())
    }

    pub async fn publications_for(&self, email: &str) -> Result<Vec<Publication>, StatusCode> {
        let rows = sqlx::query(&format!(
            "SELECT id, source_id, status, title, content, author_email, category_id, model, kind, members FROM {} WHERE author_email = $1 ORDER BY id",
            self.t("publications")
        ))
        .bind(email)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(rows.iter().map(Self::publication_from_row).collect())
    }

    pub async fn set_publication_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<Publication, StatusCode> {
        self.review_publication(id, status, None, None).await
    }

    pub async fn review_publication(
        &self,
        id: &str,
        status: &str,
        reason: Option<&str>,
        actor: Option<(&str, &str)>,
    ) -> Result<Publication, StatusCode> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if let Some((email, token)) = actor {
            let role: Option<String> = sqlx::query_scalar(&format!("SELECT role FROM {} a WHERE email=$1 AND NOT disabled AND EXISTS(SELECT 1 FROM {} WHERE email=a.email AND token=$2) FOR SHARE", self.t("accounts"), self.t("access_tokens")))
                .bind(email).bind(token).fetch_optional(&mut *transaction).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let role = role.ok_or(StatusCode::UNAUTHORIZED)?;
            if !crate::admin_users::staff(&role) {
                return Err(StatusCode::FORBIDDEN);
            }
        }
        self.catalog_lock(&mut transaction).await?;
        let existing = sqlx::query(&format!(
            "SELECT status FROM {} WHERE id = $1 FOR UPDATE",
            self.t("publications")
        ))
        .bind(id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
        let previous: String = existing.get("status");
        if previous != "pending" && previous != status {
            return Err(StatusCode::CONFLICT);
        }
        let row = sqlx::query(&format!(
            "UPDATE {} SET status = $2 WHERE id = $1
             RETURNING id, source_id, status, title, content, author_email, category_id, model, kind, members",
            self.t("publications")
        ))
        .bind(id)
        .bind(status)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
        let publication = Self::publication_from_row(&row);
        if status == "approved" {
            if let Some(mut item) = publication.square_item() {
                self.resolve_catalog_item(&mut transaction, &mut item).await?;
                sqlx::query(&format!(
                    "INSERT INTO {} (id, title, kind, excerpt, model, member_count, content, category_id, members, sort_index)
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9, COALESCE((SELECT MAX(sort_index)+1 FROM {0}), 0)) ON CONFLICT(id) DO NOTHING",
                    self.t("square_items")))
                    .bind(&item.id).bind(&item.title).bind(&item.kind).bind(&item.excerpt).bind(&item.model)
                    .bind(item.member_count).bind(&item.content).bind(&item.category_id).bind(sqlx::types::Json(&item.members))
                    .execute(&mut *transaction).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        }
        if previous == "pending" {
            if let Some((email, _)) = actor {
                sqlx::query(&format!("INSERT INTO {} (publication_id,actor_email,status,reason) VALUES ($1,$2,$3,$4)", self.t("review_events")))
                    .bind(id).bind(email).bind(status).bind(reason).execute(&mut *transaction).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                sqlx::query(&format!("INSERT INTO {} (id,actor_email,action,details) VALUES ($1,$2,'publication_reviewed',$3)", self.t("security_audit")))
                    .bind(uuid::Uuid::new_v4().to_string()).bind(email).bind(serde_json::json!({"publication_id":id,"status":status,"reason":reason}))
                    .execute(&mut *transaction).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        }
        transaction
            .commit()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(publication)
    }

    pub async fn favorite_ids(&self, email: &str) -> Result<Vec<String>, StatusCode> {
        let rows = sqlx::query(&format!(
            "SELECT item_id FROM {} WHERE email = $1",
            self.t("favorites")
        ))
        .bind(email)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(rows.into_iter().map(|row| row.get("item_id")).collect())
    }

    pub async fn put_favorite(&self, email: &str, item_id: &str) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "INSERT INTO {} (email, item_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            self.t("favorites")
        ))
        .bind(email)
        .bind(item_id)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub async fn delete_favorite(&self, email: &str, item_id: &str) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "DELETE FROM {} WHERE email = $1 AND item_id = $2",
            self.t("favorites")
        ))
        .bind(email)
        .bind(item_id)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub async fn square_public(&self) -> Result<bool, StatusCode> {
        let row = sqlx::query(&format!(
            "SELECT value FROM {} WHERE key = 'square_public'",
            self.t("settings")
        ))
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(row
            .map(|row| row.get::<String, _>("value") != "false")
            .unwrap_or(true))
    }

    pub async fn set_square_public(&self, value: bool) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "INSERT INTO {} (key, value) VALUES ('square_public', $1)
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
            self.t("settings")
        ))
        .bind(if value { "true" } else { "false" })
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub async fn oauth_email(
        &self,
        provider: &str,
        provider_uid: &str,
    ) -> Result<Option<String>, StatusCode> {
        let row = sqlx::query(&format!(
            "SELECT email FROM {} WHERE provider = $1 AND provider_uid = $2",
            self.t("oauth_accounts")
        ))
        .bind(provider)
        .bind(provider_uid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(row.map(|row| row.get("email")))
    }

    pub async fn link_oauth(
        &self,
        provider: &str,
        provider_uid: &str,
        email: &str,
    ) -> Result<String, StatusCode> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        self.identity_lock(&mut tx).await?;
        if let Some(existing) = sqlx::query_scalar::<_, String>(&format!(
            "SELECT email FROM {} WHERE provider=$1 AND provider_uid=$2",
            self.t("oauth_accounts")
        ))
        .bind(provider)
        .bind(provider_uid)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            return Ok(existing);
        }
        let accounts: Vec<String> = sqlx::query_scalar(&format!(
            "SELECT email FROM {} WHERE lower(email)=lower($1)",
            self.t("accounts")
        ))
        .bind(email)
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if accounts.len() > 1 {
            return Err(StatusCode::CONFLICT);
        }
        let email = if let Some(existing) = accounts.first() {
            existing.to_owned()
        } else {
            let open: bool = sqlx::query_scalar(&format!(
                "SELECT (data->>'registration_open')::boolean FROM {} WHERE id=1 FOR SHARE",
                self.t("identity_policy")
            ))
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if !open {
                return Err(StatusCode::FORBIDDEN);
            }
            email.trim().to_lowercase()
        };
        let conflict: bool = sqlx::query_scalar(&format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE provider=$1 AND email=$2 AND provider_uid<>$3)",
            self.t("oauth_accounts")
        ))
        .bind(provider)
        .bind(&email)
        .bind(provider_uid)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if conflict {
            return Err(StatusCode::CONFLICT);
        }
        let disabled: Option<bool> = sqlx::query_scalar(&format!(
            "SELECT disabled FROM {} WHERE email=$1 FOR UPDATE",
            self.t("accounts")
        ))
        .bind(&email)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if disabled == Some(true) {
            return Err(StatusCode::FORBIDDEN);
        }
        sqlx::query(&format!(
            "INSERT INTO {} (email, password_hash, role) VALUES ($1, NULL, 'user')
             ON CONFLICT (email) DO NOTHING",
            self.t("accounts")
        ))
        .bind(&email)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        sqlx::query(&format!(
            "INSERT INTO {} (provider, provider_uid, email) VALUES ($1, $2, $3)
             ON CONFLICT (provider, provider_uid) DO NOTHING",
            self.t("oauth_accounts")
        ))
        .bind(provider)
        .bind(provider_uid)
        .bind(&email)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        tx.commit()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(email)
    }

    pub async fn insert_media(
        &self,
        id: &str,
        owner: &str,
        key: &str,
        content_type: Option<&str>,
    ) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "INSERT INTO {} (id, owner_email, object_key, content_type) VALUES ($1,$2,$3,$4)",
            self.t("media_objects")
        ))
        .bind(id)
        .bind(owner)
        .bind(key)
        .bind(content_type)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub async fn media_key(&self, id: &str) -> Result<Option<String>, StatusCode> {
        let row = sqlx::query(&format!(
            "SELECT object_key FROM {} WHERE id = $1",
            self.t("media_objects")
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(row.map(|row| row.get("object_key")))
    }

    pub async fn ping(&self) -> bool {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await.is_ok()
    }

    pub async fn seed_unused_redeem_code(&self, code: &str) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "INSERT INTO {} (code) VALUES ($1) ON CONFLICT (code) DO NOTHING",
            self.t("redeem_codes")
        ))
        .bind(code)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub async fn account_is_pro(&self, email: &str) -> Result<bool, StatusCode> {
        let row = sqlx::query(&format!(
            "SELECT pro FROM {} WHERE email = $1",
            self.t("accounts")
        ))
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(row.map(|row| row.get::<bool, _>("pro")).unwrap_or(false))
    }

    pub async fn redeem_code(&self, email: &str, code: &str) -> Result<(), StatusCode> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let updated = sqlx::query(&format!(
            "UPDATE {} SET used_by = $2 WHERE code = $1 AND used_by IS NULL",
            self.t("redeem_codes")
        ))
        .bind(code)
        .bind(email)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if updated.rows_affected() == 1 {
            sqlx::query(&format!(
                "UPDATE {} SET pro = TRUE WHERE email = $1",
                self.t("accounts")
            ))
            .bind(email)
            .execute(&mut *tx)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            tx.commit()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(());
        }
        let _ = tx.rollback().await;
        let row = sqlx::query(&format!(
            "SELECT used_by FROM {} WHERE code = $1",
            self.t("redeem_codes")
        ))
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match row {
            None => Err(StatusCode::NOT_FOUND),
            Some(_) => Err(StatusCode::CONFLICT),
        }
    }

    pub async fn grant_pro(&self, email: &str) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "UPDATE {} SET pro = TRUE WHERE email = $1",
            self.t("accounts")
        ))
        .bind(email)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }
}
