pub fn normalize(raw: &str) -> Result<String, String> {
    let url = url::Url::parse(raw.trim()).map_err(|_| "服务地址无效")?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !matches!(url.scheme(), "http" | "https") || (!local && url.scheme() != "https")
        || !url.username().is_empty() || url.password().is_some() || url.path() != "/"
        || url.query().is_some() || url.fragment().is_some()
    {
        return Err("服务地址必须为 HTTPS origin（本机可用 HTTP），不能包含凭据、路径或查询参数".into());
    }
    Ok(url.origin().ascii_serialization())
}

pub fn api_base() -> Result<String, String> {
    normalize(&std::env::var("PROMPTARK_API_BASE").unwrap_or_else(|_| {
        option_env!("PROMPTARK_API_BASE").unwrap_or("http://127.0.0.1:8787").into()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_shared_native_service_origin() {
        assert_eq!(normalize("https://api.example.test/").unwrap(), "https://api.example.test");
        assert_eq!(normalize("http://[::1]:8787/").unwrap(), "http://[::1]:8787");
        for value in ["http://example.com", "https://user:pass@example.com", "file:///tmp/x", "https://api.example/x", "https://api.example?q=x", "https://api.example#x", "invalid"] {
            assert!(normalize(value).is_err(), "{value}");
        }
    }
}
