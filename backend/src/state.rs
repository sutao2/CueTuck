use crate::media;
use crate::oauth::{OAuthSettings, OAuthUser};
use crate::password::verify_password;
use crate::postgres::Pg;
use crate::{SessionResponse, SquareItem, AppState};
use axum::http::StatusCode;
use uuid::Uuid;

impl AppState {
    pub async fn from_runtime() -> Result<Self, String> {
        let url = std::env::var("PROMPTARK_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://pl:pl@127.0.0.1:5432/promptark?sslmode=disable".into()
        });
        let pool = sqlx::PgPool::connect(&url)
            .await
            .map_err(|err| format!("postgres: {err}"))?;
        let reset = std::env::var("PROMPTARK_RESET_SCHEMA").ok().as_deref() == Some("1");
        let pg = Pg::new(pool, "public").map_err(|_| "invalid schema".to_string())?;
        pg.apply_schema(reset)
            .await
            .map_err(|err| format!("schema: {err}"))?;
        let redis = {
            let url = std::env::var("PROMPTARK_REDIS_URL")
                .or_else(|_| std::env::var("REDIS_URL"))
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
            match redis::Client::open(url) {
                Ok(client) => client.get_connection_manager().await.ok(),
                Err(_) => None,
            }
        };
        let state = Self {
            db: Some(pg),
            redis,
            media: media::MediaConfig::from_env(),
            oauth: OAuthSettings::default(),
            ..Self::default()
        };
        let email =
            std::env::var("PROMPTARK_DEV_EMAIL").unwrap_or_else(|_| "dev@promptark.local".into());
        let password =
            std::env::var("PROMPTARK_DEV_PASSWORD").unwrap_or_else(|_| "devpass".into());
        let admin_email =
            std::env::var("PROMPTARK_ADMIN_EMAIL").unwrap_or_else(|_| "admin@promptark.local".into());
        let admin_password =
            std::env::var("PROMPTARK_ADMIN_PASSWORD").unwrap_or_else(|_| "adminpass".into());
        if let Some(pg) = &state.db {
            pg.upsert_account(&email, Some(&password), "user")
                .await
                .map_err(|_| "seed user".to_string())?;
            pg.upsert_account(&admin_email, Some(&admin_password), "admin")
                .await
                .map_err(|_| "seed admin".to_string())?;
            if pg.list_items().await.unwrap_or_default().is_empty() {
                state.seed_square_demo();
                let items = state.items.lock().expect("items").clone();
                pg.replace_items(&items)
                    .await
                    .map_err(|_| "seed square".to_string())?;
            }
        }
        Ok(state)
    }

    pub async fn from_pool(pool: sqlx::PgPool, schema: &str) -> Result<Self, StatusCode> {
        let pg = Pg::new(pool, schema)?;
        pg.apply_schema(true)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Self {
            db: Some(pg),
            ..Self::default()
        })
    }

    pub(crate) async fn verify_login(&self, email: &str, password: &str) -> Result<(), StatusCode> {
        if let Some(pg) = &self.db {
            return pg.verify_login(email, password).await;
        }
        let expected = self
            .users
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .get(email)
            .cloned();
        match expected {
            Some(hash) if verify_password(password, &hash) => Ok(()),
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }

    pub(crate) async fn issue_session(&self, email: String) -> Result<SessionResponse, StatusCode> {
        if let Some(pg) = &self.db {
            let (access_token, refresh_token) = pg.issue_session(&email).await?;
            self.cache_tokens(&access_token, &refresh_token, &email)
                .await;
            return Ok(SessionResponse {
                email,
                access_token,
                refresh_token,
            });
        }
        let access_token = format!("acc.{}", Uuid::new_v4());
        let refresh_token = format!("ref.{}", Uuid::new_v4());
        self.access
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .insert(access_token.clone(), email.clone());
        self.refresh
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .insert(refresh_token.clone(), email.clone());
        Ok(SessionResponse {
            email,
            access_token,
            refresh_token,
        })
    }

    async fn cache_tokens(&self, access: &str, refresh: &str, email: &str) {
        if let Some(redis) = &self.redis {
            let mut conn = redis.clone();
            let _: Result<(), _> = redis::cmd("SET")
                .arg(format!("promptark:access:{access}"))
                .arg(email)
                .query_async(&mut conn)
                .await;
            let _: Result<(), _> = redis::cmd("SET")
                .arg(format!("promptark:refresh:{refresh}"))
                .arg(email)
                .query_async(&mut conn)
                .await;
        }
    }

    pub(crate) async fn rotate_refresh(&self, token: &str) -> Result<String, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.rotate_refresh(token).await;
        }
        let email = self
            .refresh
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .remove(token)
            .ok_or(StatusCode::UNAUTHORIZED)?;
        self.access
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .retain(|_, holder| holder != &email);
        Ok(email)
    }

    pub(crate) async fn revoke_access(&self, token: &str) -> Result<bool, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.revoke_access(token).await;
        }
        Ok(self
            .access
            .lock()
            .ok()
            .and_then(|mut map| map.remove(token))
            .is_some())
    }

    pub(crate) async fn email_for_access(&self, token: &str) -> Result<Option<String>, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.email_for_access(token).await;
        }
        Ok(self.access.lock().ok().and_then(|map| map.get(token).cloned()))
    }

    pub(crate) async fn role_of(&self, email: &str) -> Result<String, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.role_of(email).await;
        }
        Ok(self
            .roles
            .lock()
            .ok()
            .and_then(|map| map.get(email).cloned())
            .unwrap_or_else(|| "user".into()))
    }

    pub(crate) async fn square_public(&self) -> Result<bool, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.square_public().await;
        }
        Ok(*self
            .square_public
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
    }

    pub(crate) async fn all_items(&self) -> Result<Vec<SquareItem>, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.list_items().await;
        }
        Ok(self
            .items
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .clone())
    }

    pub(crate) async fn get_item(&self, id: &str) -> Result<Option<SquareItem>, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.get_item(id).await;
        }
        Ok(self
            .items
            .lock()
            .ok()
            .and_then(|rows| rows.iter().find(|item| item.id == id).cloned()))
    }

    pub async fn oauth_login(&self, user: &OAuthUser) -> Result<SessionResponse, StatusCode> {
        let email = if let Some(pg) = &self.db {
            if let Some(existing) = pg.oauth_email(&user.provider, &user.provider_uid).await? {
                existing
            } else {
                pg.link_oauth(&user.provider, &user.provider_uid, &user.email)
                    .await?;
                user.email.clone()
            }
        } else {
            let mut users = self
                .users
                .lock()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            users.entry(user.email.clone()).or_insert_with(String::new);
            self.roles
                .lock()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .entry(user.email.clone())
                .or_insert_with(|| "user".into());
            user.email.clone()
        };
        self.issue_session(email).await
    }

    pub async fn put_oauth_flow(&self, flow_id: &str, payload: String) {
        if let Some(redis) = &self.redis {
            let mut conn = redis.clone();
            let _: Result<(), _> = redis::cmd("SET")
                .arg(format!("promptark:oauth:{flow_id}"))
                .arg(&payload)
                .arg("EX")
                .arg(600)
                .query_async(&mut conn)
                .await;
            return;
        }
        if let Ok(mut map) = self.oauth_flows.lock() {
            map.insert(flow_id.to_string(), payload);
        }
    }

    pub async fn get_oauth_flow(&self, flow_id: &str) -> Option<String> {
        if let Some(redis) = &self.redis {
            let mut conn = redis.clone();
            if let Ok(value) = redis::cmd("GET")
                .arg(format!("promptark:oauth:{flow_id}"))
                .query_async::<String>(&mut conn)
                .await
            {
                return Some(value);
            }
        }
        self.oauth_flows
            .lock()
            .ok()
            .and_then(|map| map.get(flow_id).cloned())
    }

    pub(crate) async fn insert_publication(
        &self,
        publication: &crate::Publication,
    ) -> Result<(), StatusCode> {
        if let Some(pg) = &self.db {
            return pg.insert_publication(publication).await;
        }
        self.publications
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .push(publication.clone());
        Ok(())
    }

    pub(crate) async fn pending_publications(
        &self,
    ) -> Result<Vec<crate::Publication>, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.pending_publications().await;
        }
        Ok(self
            .publications
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .iter()
            .filter(|row| row.status == "pending")
            .cloned()
            .collect())
    }

    pub(crate) async fn publications_for(
        &self,
        email: &str,
    ) -> Result<Vec<crate::Publication>, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.publications_for(email).await;
        }
        Ok(self
            .publications
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .iter()
            .filter(|row| row.author_email.as_deref() == Some(email))
            .cloned()
            .collect())
    }

    pub(crate) async fn get_profile(&self, email: &str) -> Result<crate::me::MeProfile, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.get_profile(email).await;
        }
        let stored = self
            .profiles
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .get(email)
            .cloned();
        Ok(match stored {
            Some((display_name, bio)) => crate::me::MeProfile {
                email: email.into(),
                display_name,
                bio,
            },
            None => crate::me::MeProfile {
                email: email.into(),
                display_name: None,
                bio: None,
            },
        })
    }

    pub(crate) async fn put_profile(
        &self,
        email: &str,
        display_name: Option<String>,
        bio: Option<String>,
    ) -> Result<crate::me::MeProfile, StatusCode> {
        if let Some(pg) = &self.db {
            return pg
                .put_profile(email, display_name.as_deref(), bio.as_deref())
                .await;
        }
        self.profiles
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .insert(email.to_string(), (display_name.clone(), bio.clone()));
        Ok(crate::me::MeProfile {
            email: email.into(),
            display_name,
            bio,
        })
    }

    pub(crate) async fn set_publication_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<crate::Publication, StatusCode> {
        if let Some(pg) = &self.db {
            let publication = pg.set_publication_status(id, status).await?;
            if status == "approved" {
                let title = publication
                    .title
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !title.is_empty() {
                    pg.insert_item(&SquareItem {
                        id: publication.id.clone(),
                        title,
                        kind: "prompt".into(),
                        excerpt: None,
                        model: None,
                        member_count: None,
                        content: publication.content.clone(),
                    })
                    .await?;
                }
            }
            return Ok(publication);
        }
        let publication = {
            let mut rows = self
                .publications
                .lock()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let publication = rows
                .iter_mut()
                .find(|row| row.id == id)
                .ok_or(StatusCode::NOT_FOUND)?;
            publication.status = status.into();
            publication.clone()
        };
        if status == "approved" {
            let title = publication
                .title
                .as_deref()
                .unwrap_or("")
                .trim()
                .to_string();
            if !title.is_empty() {
                self.items
                    .lock()
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                    .push(SquareItem {
                        id: publication.id.clone(),
                        title,
                        kind: "prompt".into(),
                        excerpt: None,
                        model: None,
                        member_count: None,
                        content: publication.content.clone(),
                    });
            }
        }
        Ok(publication)
    }

    pub(crate) async fn favorite_ids(&self, email: &str) -> Result<Vec<String>, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.favorite_ids(email).await;
        }
        Ok(self
            .favorites
            .lock()
            .ok()
            .and_then(|map| map.get(email).cloned())
            .map(|set| set.into_iter().collect())
            .unwrap_or_default())
    }

    pub(crate) async fn put_favorite(&self, email: &str, item_id: &str) -> Result<(), StatusCode> {
        if let Some(pg) = &self.db {
            return pg.put_favorite(email, item_id).await;
        }
        self.favorites
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .entry(email.to_string())
            .or_default()
            .insert(item_id.to_string());
        Ok(())
    }

    pub(crate) async fn delete_favorite(&self, email: &str, item_id: &str) -> Result<(), StatusCode> {
        if let Some(pg) = &self.db {
            return pg.delete_favorite(email, item_id).await;
        }
        if let Ok(mut map) = self.favorites.lock() {
            if let Some(ids) = map.get_mut(email) {
                ids.remove(item_id);
            }
        }
        Ok(())
    }

    pub(crate) async fn set_square_public(&self, value: bool) -> Result<(), StatusCode> {
        if let Some(pg) = &self.db {
            return pg.set_square_public(value).await;
        }
        *self
            .square_public
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? = value;
        Ok(())
    }

    pub(crate) async fn list_users(&self) -> Result<Vec<crate::AdminUser>, StatusCode> {
        if let Some(pg) = &self.db {
            return pg.list_users().await;
        }
        let hashes = self
            .users
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let roles = self
            .roles
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let mut items: Vec<crate::AdminUser> = hashes
            .keys()
            .map(|email| crate::AdminUser {
                email: email.clone(),
                role: roles.get(email).cloned().unwrap_or_else(|| "user".into()),
            })
            .collect();
        items.sort_by(|left, right| left.email.cmp(&right.email));
        Ok(items)
    }

    pub(crate) async fn ping_db(&self) -> bool {
        match &self.db {
            Some(pg) => pg.ping().await,
            None => true,
        }
    }

    pub(crate) async fn ping_redis(&self) -> bool {
        let Some(redis) = &self.redis else {
            return false;
        };
        let mut conn = redis.clone();
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .is_ok()
    }
}
