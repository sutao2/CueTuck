use crate::oauth::ProviderConfig;
use crate::{require_configuration_admin, AppState};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ring::{
    aead,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, io::Write, path::Path as FilePath, sync::Mutex};

pub const PROVIDERS: [&str; 2] = ["google", "github"];
const DEFAULT_CALLBACK: &str = "http://localhost:8787/v1/session/oauth/callback";
type ApiError = (StatusCode, Json<Value>);
fn invalid(message: &str) -> ApiError {
    (StatusCode::BAD_REQUEST, Json(json!({ "message": message })))
}
fn internal(_: impl std::fmt::Debug) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StoredProvider {
    #[serde(default)]
    revision:i64,
    enabled: bool,
    client_id: String,
    redirect_uri: String,
    encrypted_secret: String,
}

pub struct ConfigStore {
    pub key: [u8; 32],
    memory: Mutex<HashMap<String, StoredProvider>>,
    writes: tokio::sync::Mutex<()>,
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new(random_key())
    }
}
impl ConfigStore {
    pub fn new(key: [u8; 32]) -> Self {
        Self {
            key,
            memory: Mutex::new(HashMap::new()),
            writes: tokio::sync::Mutex::new(()),
        }
    }
}
fn random_key() -> [u8; 32] {
    let mut key = [0; 32];
    SystemRandom::new()
        .fill(&mut key)
        .expect("OS random source unavailable");
    key
}

pub fn load_key(path: &FilePath, existing_config: bool) -> Result<[u8; 32], String> {
    if !path.exists() {
        if existing_config {
            return Err("OAuth 配置密钥文件丢失，请恢复原密钥文件".into());
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            let mut builder = std::fs::DirBuilder::new();
            builder.recursive(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder
                .create(parent)
                .map_err(|_| "无法创建 OAuth 密钥目录")?;
        }
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(path) {
            Ok(mut file) => {
                file.write_all(&random_key())
                    .and_then(|_| file.sync_all())
                    .map_err(|_| "无法保存 OAuth 密钥文件")?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err("无法创建 OAuth 密钥文件".into()),
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if std::fs::metadata(path)
            .map_err(|_| "无法读取 OAuth 密钥文件")?
            .permissions()
            .mode()
            & 0o077
            != 0
        {
            return Err("OAuth 密钥文件权限必须仅限所有者（0600）".into());
        }
    }
    std::fs::read(path)
        .map_err(|_| "无法读取 OAuth 密钥文件")?
        .try_into()
        .map_err(|_| "OAuth 密钥文件必须为 32 字节".into())
}

pub(crate) fn seal(key: &[u8; 32], provider: &str, secret: &str) -> Result<String, StatusCode> {
    if secret.is_empty() {
        return Ok(String::new());
    }
    let key =
        aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_256_GCM, key).map_err(internal)?);
    let mut nonce = [0; 12];
    SystemRandom::new().fill(&mut nonce).map_err(internal)?;
    let mut bytes = secret.as_bytes().to_vec();
    key.seal_in_place_append_tag(
        aead::Nonce::assume_unique_for_key(nonce),
        aead::Aad::from(provider.as_bytes()),
        &mut bytes,
    )
    .map_err(internal)?;
    Ok(URL_SAFE_NO_PAD.encode([nonce.to_vec(), bytes].concat()))
}
pub(crate) fn unseal(key: &[u8; 32], provider: &str, encrypted: &str) -> Result<String, StatusCode> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }
    let mut bytes = URL_SAFE_NO_PAD.decode(encrypted).map_err(internal)?;
    if bytes.len() < 28 {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let (nonce, body) = bytes.split_at_mut(12);
    let key =
        aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_256_GCM, key).map_err(internal)?);
    let text = key
        .open_in_place(
            aead::Nonce::try_assume_unique_for_key(nonce).map_err(internal)?,
            aead::Aad::from(provider.as_bytes()),
            body,
        )
        .map_err(internal)?;
    String::from_utf8(text.to_vec()).map_err(internal)
}

impl AppState {
    pub(crate) async fn oauth_revision(&self,name:&str)->Result<i64,StatusCode>{Ok(self.stored_provider(name).await?.map_or(0,|c|c.revision))}
    async fn stored_provider(&self, name: &str) -> Result<Option<StoredProvider>, StatusCode> {
        if let Some(pg) = &self.db {
            return pg
                .oauth_config(name)
                .await?
                .map(|raw| serde_json::from_str(&raw).map_err(internal))
                .transpose();
        }
        Ok(self
            .oauth_config
            .memory
            .lock()
            .map_err(internal)?
            .get(name)
            .cloned())
    }
    pub(crate) async fn runtime_provider(
        &self,
        name: &str,
    ) -> Result<Option<ProviderConfig>, StatusCode> {
        if !PROVIDERS.contains(&name) {
            return Ok(None);
        }
        match self.stored_provider(name).await? {
            Some(stored) if stored.enabled => Ok(Some(ProviderConfig::configured(
                name,
                stored.client_id,
                unseal(&self.oauth_config.key, name, &stored.encrypted_secret)?,
                stored.redirect_uri,
            ))),
            Some(_) => Ok(None),
            None => Ok(self.oauth.providers.get(name).cloned()),
        }
    }
    async fn provider_view(&self, name: &str) -> Result<Value, StatusCode> {
        if let Some(config) = self.stored_provider(name).await? {
            return Ok(
                json!({ "provider": name, "enabled": config.enabled, "client_id": config.client_id,
                "redirect_uri": config.redirect_uri, "secret_configured": !config.encrypted_secret.is_empty(), "source": "database" }),
            );
        }
        let config = self.oauth.providers.get(name);
        Ok(
            json!({ "provider": name, "enabled": config.is_some(), "client_id": config.map(|c| c.client_id.as_str()).unwrap_or(""),
            "redirect_uri": config.map(|c| c.redirect_uri.as_str()).unwrap_or(DEFAULT_CALLBACK),
            "secret_configured": config.is_some_and(|c| !c.client_secret.is_empty()), "source": if config.is_some() { "environment" } else { "none" } }),
        )
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateProvider {
    #[serde(default)]
    revision:Option<i64>,
    #[serde(default)]
    current_password:String,
    enabled: bool,
    client_id: String,
    redirect_uri: String,
    #[serde(default)]
    client_secret: String,
}

fn validate(input: &UpdateProvider, has_secret: bool) -> Result<(), ApiError> {
    if input.client_id.len() > 1024
        || input.client_secret.len() > 4096
        || input.redirect_uri.len() > 2048
    {
        return Err(invalid("配置字段过长"));
    }
    if input.enabled
        && (input.client_id.trim().is_empty()
            || !has_secret
            || input.redirect_uri.trim().is_empty())
    {
        return Err(invalid("启用前请填写 Client ID、Client Secret 和回调地址"));
    }
    if !input.redirect_uri.is_empty() {
        let uri = url::Url::parse(&input.redirect_uri).map_err(|_| invalid("回调地址格式错误"))?;
        let local = uri
            .host_str()
            .is_some_and(|host| matches!(host, "localhost" | "127.0.0.1" | "[::1]"));
        if uri.host_str().is_none()
            || (uri.scheme() != "https" && !(local && uri.scheme() == "http"))
            || !uri.username().is_empty()
            || uri.password().is_some()
            || uri.query().is_some()
            || uri.fragment().is_some()
            || !uri.path().ends_with("/v1/session/oauth/callback")
        {
            return Err(invalid(
                "回调须为 HTTPS（本机可用 HTTP），以 /v1/session/oauth/callback 结尾且不含查询参数",
            ));
        }
    }
    Ok(())
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    require_configuration_admin(&state, &headers).await?;
    let mut items = Vec::new();
    for name in PROVIDERS {
        let mut view=state.provider_view(name).await?;
        view["revision"]=json!(state.oauth_revision(name).await?);
        view["verification"]=state.oauth_verification_view(name).await?;
        items.push(view);
    }
    Ok(Json(json!({ "items": items })))
}

pub async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(mut input): Json<UpdateProvider>,
) -> Result<Json<Value>, ApiError> {
    let map_error = |status| (status, Json(json!({ "message": "无权限或配置服务不可用" })));
    let actor=require_configuration_admin(&state, &headers).await.map_err(map_error)?;
    if !PROVIDERS.contains(&name.as_str()) {
        return Err(invalid("不支持的登录提供商"));
    }
    let _guard = state.oauth_config.writes.lock().await;
    if input.current_password.len()>512{return Err(invalid("当前密码过长"))}
    let mut transaction=if let Some(pg)=&state.db {
        state.limit_account_auth(&actor).await.map_err(map_error)?;
        let mut tx=pg.pool.begin().await.map_err(|_|map_error(StatusCode::INTERNAL_SERVER_ERROR))?;
        crate::oauth_verification::owner_lock(pg,&mut tx,&actor,&crate::bearer_token(&headers).ok_or_else(||map_error(StatusCode::UNAUTHORIZED))?).await.map_err(map_error)?;
        pg.verify_login(&actor,&input.current_password).await.map_err(|_|(StatusCode::FORBIDDEN,Json(json!({"message":"当前密码不正确，请重新验证本人身份"}))))?;
        crate::oauth_verification::config_lock(pg,&mut tx).await.map_err(map_error)?;
        Some(tx)
    }else{None};
    input.client_id = input.client_id.trim().into();
    input.redirect_uri = input.redirect_uri.trim().into();
    input.client_secret = input.client_secret.trim().into();
    let current = state.stored_provider(&name).await.map_err(map_error)?;
    let revision=current.as_ref().map_or(0,|c|c.revision);
    if (state.db.is_some() && input.revision.is_none()) || input.revision.is_some_and(|r|r!=revision){return Err((StatusCode::CONFLICT,Json(json!({"message":"配置版本已变化，请刷新后核对；草稿已保留"}))))}
    let secret = if !input.client_secret.is_empty() {
        input.client_secret.clone()
    } else if let Some(stored) = &current {
        unseal(&state.oauth_config.key, &name, &stored.encrypted_secret).map_err(map_error)?
    } else {
        state
            .oauth
            .providers
            .get(&name)
            .map(|c| c.client_secret.clone())
            .unwrap_or_default()
    };
    validate(&input, !secret.is_empty())?;
    let stored = StoredProvider {
        revision:revision+1,
        enabled: input.enabled,
        client_id: input.client_id,
        redirect_uri: input.redirect_uri,
        encrypted_secret: seal(&state.oauth_config.key, &name, &secret).map_err(map_error)?,
    };
    if let (Some(pg),Some(tx))=(&state.db,transaction.as_mut()) {
        sqlx::query(&format!("INSERT INTO {} (key,value) VALUES ($1,$2) ON CONFLICT(key) DO UPDATE SET value=EXCLUDED.value",pg.t("settings"))).bind(format!("oauth_provider_{name}")).bind(serde_json::to_string(&stored).map_err(|_|map_error(StatusCode::INTERNAL_SERVER_ERROR))?).execute(&mut **tx).await.map_err(|_|map_error(StatusCode::INTERNAL_SERVER_ERROR))?;
        sqlx::query(&format!("UPDATE {} SET status='configuration_changed',message='配置已变化，请重新验证',finished_at=now() WHERE provider=$1",pg.t("oauth_verifications"))).bind(&name).execute(&mut **tx).await.map_err(|_|map_error(StatusCode::INTERNAL_SERVER_ERROR))?;
        crate::admin_risk::audit(pg,tx,&actor,"oauth_configuration_saved",json!({"provider":name,"revision":stored.revision,"enabled":stored.enabled})).await.map_err(map_error)?;
    } else {
        state
            .oauth_config
            .memory
            .lock()
            .map_err(|_| map_error(StatusCode::INTERNAL_SERVER_ERROR))?
            .insert(name.clone(), stored);
    }
    if let Some(tx)=transaction{tx.commit().await.map_err(|_|map_error(StatusCode::INTERNAL_SERVER_ERROR))?;}
    let mut view=state.provider_view(&name).await.map_err(map_error)?;
    view["revision"]=json!(state.oauth_revision(&name).await.map_err(map_error)?);
    view["verification"]=state.oauth_verification_view(&name).await.map_err(map_error)?;
    Ok(Json(view))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    async fn call(
        router: &axum::Router,
        method: &str,
        path: &str,
        token: &str,
        body: Value,
    ) -> (StatusCode, Value) {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), 100_000).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
    }
    fn state() -> AppState {
        let mut state = AppState::default();
        state.oauth.providers.clear();
        state
            .access
            .lock()
            .unwrap()
            .insert("admin-token".into(), "admin@example.com".into());
        state
            .access
            .lock()
            .unwrap()
            .insert("user-token".into(), "user@example.com".into());
        state
            .roles
            .lock()
            .unwrap()
            .insert("admin@example.com".into(), "admin".into());
        state
    }
    fn input() -> Value {
        json!({ "enabled": true, "client_id": "client-test", "client_secret": "secret-for-tests", "redirect_uri": DEFAULT_CALLBACK })
    }

    #[tokio::test]
    async fn configuration_requires_admin_and_valid_fields_without_mutation() {
        let state = state();
        let router = crate::app(state.clone());
        for token in ["", "user-token"] {
            for (method, path) in [
                ("GET", "/v1/admin/oauth"),
                ("PUT", "/v1/admin/oauth/google"),
            ] {
                let (status, _) = call(&router, method, path, token, input()).await;
                assert!(status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN);
            }
        }
        for callback in [
            "http://example.com/v1/session/oauth/callback",
            "https://user:pass@example.com/v1/session/oauth/callback",
            "https://example.com/other",
            "https://example.com/v1/session/oauth/callback?code=x",
            "javascript:alert(1)",
        ] {
            let mut bad = input();
            bad["redirect_uri"] = json!(callback);
            assert_eq!(
                call(&router, "PUT", "/v1/admin/oauth/google", "admin-token", bad)
                    .await
                    .0,
                StatusCode::BAD_REQUEST
            );
        }
        for field in ["client_id", "client_secret", "redirect_uri"] {
            let mut bad = input();
            bad[field] = json!("");
            assert_eq!(
                call(&router, "PUT", "/v1/admin/oauth/google", "admin-token", bad)
                    .await
                    .0,
                StatusCode::BAD_REQUEST
            );
        }
        assert_eq!(
            call(
                &router,
                "PUT",
                "/v1/admin/oauth/unknown",
                "admin-token",
                input()
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
        assert!(state.oauth_config.memory.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn saved_credentials_are_masked_preserved_and_used_immediately() {
        let state = state();
        let router = crate::app(state.clone());
        for provider in PROVIDERS {
            let path = format!("/v1/admin/oauth/{provider}");
            let (status, saved) = call(&router, "PUT", &path, "admin-token", input()).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(saved["secret_configured"], true);
            assert!(saved.get("client_secret").is_none());
            assert!(saved.get("encrypted_secret").is_none());
            let stored = state.stored_provider(provider).await.unwrap().unwrap();
            assert!(!serde_json::to_string(&stored)
                .unwrap()
                .contains("secret-for-tests"));
            let mut next = input();
            next["client_secret"] = json!("");
            next["client_id"] = json!("new-client");
            assert_eq!(
                call(&router, "PUT", &path, "admin-token", next.clone())
                    .await
                    .0,
                StatusCode::OK
            );
            assert_eq!(
                state
                    .runtime_provider(provider)
                    .await
                    .unwrap()
                    .unwrap()
                    .client_secret,
                "secret-for-tests"
            );
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/v1/session/oauth/{provider}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert!(response.status().is_redirection());
            assert!(response.headers()["location"]
                .to_str()
                .unwrap()
                .contains("client_id=new-client"));
            next["enabled"] = json!(false);
            assert_eq!(
                call(&router, "PUT", &path, "admin-token", next).await.0,
                StatusCode::OK
            );
            assert!(state.runtime_provider(provider).await.unwrap().is_none());
            assert_eq!(
                call(
                    &router,
                    "GET",
                    &format!("/v1/session/oauth/{provider}"),
                    "",
                    Value::Null
                )
                .await
                .0,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(
            call(
                &router,
                "GET",
                "/v1/session/oauth/providers",
                "",
                Value::Null
            )
            .await
            .1,
            json!({ "items": [] })
        );
        let list = call(
            &router,
            "GET",
            "/v1/admin/oauth",
            "admin-token",
            Value::Null,
        )
        .await
        .1
        .to_string();
        assert!(!list.contains("secret-for-tests"));
        assert!(!list.contains("encrypted_secret"));
    }

    #[tokio::test]
    async fn explicit_disabled_configuration_overrides_environment() {
        let mut state = state();
        state.oauth.providers.insert(
            "google".into(),
            ProviderConfig::configured(
                "google",
                "env-id".into(),
                "env-secret".into(),
                DEFAULT_CALLBACK.into(),
            ),
        );
        let router = crate::app(state.clone());
        assert_eq!(
            state.provider_view("google").await.unwrap()["source"],
            "environment"
        );
        let mut next = input();
        next["enabled"] = json!(false);
        next["client_secret"] = json!("");
        assert_eq!(
            call(
                &router,
                "PUT",
                "/v1/admin/oauth/google",
                "admin-token",
                next
            )
            .await
            .0,
            StatusCode::OK
        );
        assert!(state.runtime_provider("google").await.unwrap().is_none());
        assert_eq!(
            state.provider_view("google").await.unwrap()["source"],
            "database"
        );
    }

    #[test]
    fn encryption_authenticates_provider_and_key_and_uses_fresh_nonces() {
        let key = random_key();
        let secret = "sensitive-test-secret";
        let encrypted = seal(&key, "google", secret).unwrap();
        assert_eq!(unseal(&key, "google", &encrypted).unwrap(), secret);
        assert_ne!(encrypted, seal(&key, "google", secret).unwrap());
        assert!(unseal(&key, "github", &encrypted).is_err());
        assert!(unseal(&random_key(), "google", &encrypted).is_err());
        assert!(unseal(&key, "google", "bad").is_err());
    }

    #[test]
    fn key_file_survives_restart_and_is_not_recreated_for_existing_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets/oauth.key");
        let key = load_key(&path, false).unwrap();
        assert_eq!(key, load_key(&path, true).unwrap());
        assert!(load_key(&dir.path().join("missing.key"), true).is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
