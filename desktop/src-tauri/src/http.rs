use crate::local_database::get_setting_in_dir;
use reqwest::{Client, ClientBuilder, Proxy};
use std::path::Path;
use std::sync::Mutex;

static RUNTIME_PROXY: Mutex<String> = Mutex::new(String::new());

pub fn proxy_from_setting(raw: &str) -> Result<Option<Proxy>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let parsed = url::Url::parse(trimmed).map_err(|_| "代理地址无效".to_string())?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("代理只支持 http 或 https".into()),
    }
    if parsed.host_str().is_none() {
        return Err("代理地址无效".into());
    }
    Proxy::all(trimmed).map(Some).map_err(|error| error.to_string())
}

pub fn set_runtime_proxy(raw: &str) -> Result<(), String> {
    let _ = proxy_from_setting(raw)?;
    *RUNTIME_PROXY.lock().map_err(|error| error.to_string())? = raw.trim().to_string();
    Ok(())
}

pub fn load_from_dir(dir: &Path) {
    let raw = get_setting_in_dir(dir, "http_proxy").unwrap_or_default();
    if set_runtime_proxy(&raw).is_err() {
        let _ = set_runtime_proxy("");
    }
}

fn current_proxy() -> String {
    RUNTIME_PROXY
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

pub fn apply_proxy(builder: ClientBuilder, raw: &str) -> Result<ClientBuilder, String> {
    Ok(match proxy_from_setting(raw)? {
        Some(proxy) => builder.proxy(proxy),
        None => builder,
    })
}

pub fn apply_runtime_proxy(builder: ClientBuilder) -> Result<ClientBuilder, String> {
    apply_proxy(builder, &current_proxy())
}

pub fn client_builder() -> Result<ClientBuilder, String> {
    apply_runtime_proxy(Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30)))
}

pub fn client() -> Result<Client, String> {
    client_builder()?
        .build()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_setting_follows_system() {
        assert!(proxy_from_setting("").unwrap().is_none());
        assert!(proxy_from_setting("   ").unwrap().is_none());
    }

    #[test]
    fn http_and_https_urls_are_accepted() {
        assert!(proxy_from_setting("http://127.0.0.1:7890").unwrap().is_some());
        assert!(proxy_from_setting("https://proxy.example:8443")
            .unwrap()
            .is_some());
    }

    #[test]
    fn other_schemes_and_garbage_are_rejected() {
        assert!(proxy_from_setting("socks5://127.0.0.1:1080").is_err());
        assert!(proxy_from_setting("ftp://127.0.0.1:21").is_err());
        assert!(proxy_from_setting("not-a-url").is_err());
    }

    #[test]
    fn runtime_proxy_rejects_invalid_without_breaking_client() {
        set_runtime_proxy("").unwrap();
        assert!(set_runtime_proxy("nope").is_err());
        assert!(client().is_ok());
    }
}
