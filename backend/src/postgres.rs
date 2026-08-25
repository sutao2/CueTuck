use crate::password::{hash_password, verify_password};
use crate::{AdminUser, Publication, SquareItem};
use axum::http::StatusCode;
use sqlx::{PgPool, Row};
use uuid::Uuid;

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

    fn t(&self, table: &str) -> String {
        format!("\"{}\".\"{}\"", self.schema, table)
    }

    pub async fn apply_schema(&self, reset: bool) -> Result<(), sqlx::Error> {
        sqlx::query(&format!(
            "CREATE SCHEMA IF NOT EXISTS \"{}\"",
            self.schema
        ))
        .execute(&self.pool)
        .await?;
        if reset {
            for table in [
                "favorites",
                "media_objects",
                "oauth_accounts",
                "access_tokens",
                "refresh_tokens",
                "publications",
                "square_items",
                "settings",
                "accounts",
            ] {
                sqlx::query(&format!("DROP TABLE IF EXISTS {} CASCADE", self.t(table)))
                    .execute(&self.pool)
                    .await?;
            }
        }
        let statements = [
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                  email TEXT PRIMARY KEY,
                  password_hash TEXT,
                  role TEXT NOT NULL DEFAULT 'user'
                )",
                self.t("accounts")
            ),
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
        Ok(())
    }

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

    pub async fn issue_session(&self, email: &str) -> Result<(String, String), StatusCode> {
        let access = format!("acc.{}", Uuid::new_v4());
        let refresh = format!("ref.{}", Uuid::new_v4());
        sqlx::query(&format!(
            "INSERT INTO {} (token, email) VALUES ($1, $2)",
            self.t("access_tokens")
        ))
        .bind(&access)
        .bind(email)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        sqlx::query(&format!(
            "INSERT INTO {} (token, email) VALUES ($1, $2)",
            self.t("refresh_tokens")
        ))
        .bind(&refresh)
        .bind(email)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok((access, refresh))
    }

    pub async fn rotate_refresh(&self, token: &str) -> Result<String, StatusCode> {
        let row = sqlx::query(&format!(
            "DELETE FROM {} WHERE token = $1 RETURNING email",
            self.t("refresh_tokens")
        ))
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let email: String = row.ok_or(StatusCode::UNAUTHORIZED)?.get("email");
        sqlx::query(&format!(
            "DELETE FROM {} WHERE email = $1",
            self.t("access_tokens")
        ))
        .bind(&email)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(email)
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
            "SELECT email FROM {} WHERE token = $1",
            self.t("access_tokens")
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
            id: row.get("id"),
            title: row.get("title"),
            kind: row.get("kind"),
            excerpt: row.get("excerpt"),
            model: row.get("model"),
            member_count: row.get("member_count"),
            content: row.get("content"),
        }
    }

    pub async fn list_items(&self) -> Result<Vec<SquareItem>, StatusCode> {
        let rows = sqlx::query(&format!(
            "SELECT id, title, kind, excerpt, model, member_count, content
             FROM {} ORDER BY sort_index, id",
            self.t("square_items")
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(rows.iter().map(Self::item_from_row).collect())
    }

    pub async fn get_item(&self, id: &str) -> Result<Option<SquareItem>, StatusCode> {
        let row = sqlx::query(&format!(
            "SELECT id, title, kind, excerpt, model, member_count, content
             FROM {} WHERE id = $1",
            self.t("square_items")
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(row.as_ref().map(Self::item_from_row))
    }

    pub async fn insert_item(&self, item: &SquareItem) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "INSERT INTO {} (id, title, kind, excerpt, model, member_count, content, sort_index)
             VALUES ($1,$2,$3,$4,$5,$6,$7, COALESCE((SELECT MAX(sort_index)+1 FROM {0}), 0))",
            self.t("square_items")
        ))
        .bind(&item.id)
        .bind(&item.title)
        .bind(&item.kind)
        .bind(&item.excerpt)
        .bind(&item.model)
        .bind(item.member_count)
        .bind(&item.content)
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
                "INSERT INTO {} (id, title, kind, excerpt, model, member_count, content, sort_index)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
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
            .execute(&self.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        Ok(())
    }

    pub async fn insert_publication(&self, publication: &Publication) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "INSERT INTO {} (id, source_id, status, title, content, author_email) VALUES ($1,$2,$3,$4,$5,$6)",
            self.t("publications")
        ))
        .bind(&publication.id)
        .bind(&publication.source_id)
        .bind(&publication.status)
        .bind(&publication.title)
        .bind(&publication.content)
        .bind(&publication.author_email)
        .execute(&self.pool)
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
        }
    }

    pub async fn pending_publications(&self) -> Result<Vec<Publication>, StatusCode> {
        let rows = sqlx::query(&format!(
            "SELECT id, source_id, status, title, content, author_email FROM {} WHERE status = 'pending'",
            self.t("publications")
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(rows.iter().map(Self::publication_from_row).collect())
    }

    pub async fn publications_for(&self, email: &str) -> Result<Vec<Publication>, StatusCode> {
        let rows = sqlx::query(&format!(
            "SELECT id, source_id, status, title, content, author_email FROM {} WHERE author_email = $1 ORDER BY id",
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
        let row = sqlx::query(&format!(
            "UPDATE {} SET status = $2 WHERE id = $1
             RETURNING id, source_id, status, title, content, author_email",
            self.t("publications")
        ))
        .bind(id)
        .bind(status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
        Ok(Self::publication_from_row(&row))
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
    ) -> Result<(), StatusCode> {
        sqlx::query(&format!(
            "INSERT INTO {} (email, password_hash, role) VALUES ($1, NULL, 'user')
             ON CONFLICT (email) DO NOTHING",
            self.t("accounts")
        ))
        .bind(email)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        sqlx::query(&format!(
            "INSERT INTO {} (provider, provider_uid, email) VALUES ($1, $2, $3)
             ON CONFLICT (provider, provider_uid) DO NOTHING",
            self.t("oauth_accounts")
        ))
        .bind(provider)
        .bind(provider_uid)
        .bind(email)
        .execute(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
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
}
