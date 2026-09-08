use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

pub fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !(a == 0
                || a == 10
                || a == 127
                || a >= 224
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192 && (b == 168 || b == 0 || (b == 88 && c == 99)))
                || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
                || (a == 203 && b == 0 && c == 113))
        }
        IpAddr::V6(ip) => {
            let s = ip.segments();
            (s[0] & 0xe000) == 0x2000
                && !(s[0] == 0x2001 && (s[1] < 0x200 || s[1] == 0xdb8))
                && s[0] != 0x2002
                && !(s[0] == 0x3fff && s[1] < 0x1000)
        }
    }
}
pub fn validate_url(value: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(value).map_err(|_| "接口地址无效")?;
    if value.len() > 2048
        || url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("仅允许无凭据、查询参数和片段的 HTTPS 接口".into());
    }
    match url.host() {
        Some(url::Host::Domain(host))
            if !host.eq_ignore_ascii_case("localhost") && !host.ends_with(".localhost") => {}
        Some(url::Host::Ipv4(ip)) if public_ip(ip.into()) => {}
        Some(url::Host::Ipv6(ip)) if public_ip(ip.into()) => {}
        _ => return Err("不能访问本机、私网或保留地址".into()),
    };
    Ok(url)
}
pub async fn client(value: &str, seconds: u64) -> Result<reqwest::Client, String> {
    let url = validate_url(value)?;
    let host = match url.host().ok_or("缺少接口主机")? {
        url::Host::Domain(host) => host.to_owned(),
        url::Host::Ipv4(ip) => ip.to_string(),
        url::Host::Ipv6(ip) => ip.to_string(),
    };
    let addresses = resolve(&host, url.port_or_known_default().unwrap_or(443)).await?;
    reqwest::Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(seconds.clamp(1, 30)))
        .resolve_to_addrs(&host, &addresses)
        .build()
        .map_err(|_| "无法创建安全连接".into())
}
pub async fn resolve(host: &str, port: u16) -> Result<Vec<SocketAddr>, String> {
    let addresses: Vec<SocketAddr> = tokio::time::timeout(
        Duration::from_secs(3),
        tokio::net::lookup_host((host, port)),
    )
    .await
    .map_err(|_| "DNS 查询超时")?
    .map_err(|_| "DNS 查询失败")?
    .take(17)
    .collect();
    if addresses.is_empty() || addresses.len() > 16 || addresses.iter().any(|a| !public_ip(a.ip()))
    {
        return Err("接口必须解析为公网地址".into());
    }
    Ok(addresses)
}
pub async fn bounded_json(mut response: reqwest::Response) -> Result<serde_json::Value, String> {
    if !response.status().is_success() {
        return Err(format!("提供商返回 HTTP {}", response.status().as_u16()));
    }
    if response.content_length().is_some_and(|n| n > 262_144) {
        return Err("提供商响应过大".into());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "读取提供商响应失败")? {
        if body.len() + chunk.len() > 262_144 {
            return Err("提供商响应过大".into());
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|_| "提供商未返回合法 JSON".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn denies_private_reserved_and_credential_urls() {
        for url in [
            "http://example.com",
            "https://localhost/v1",
            "https://127.0.0.1",
            "https://2130706433",
            "https://10.0.0.1",
            "https://169.254.169.254",
            "https://100.64.0.1",
            "https://[::1]",
            "https://[::ffff:127.0.0.1]",
            "https://[2002:7f00:1::]",
            "https://user:pass@example.com",
            "https://example.com?token=secret",
            "https://example.com/#x",
        ] {
            assert!(validate_url(url).is_err(), "{url}");
        }
        assert!(validate_url("https://api.example.com/v1/chat/completions").is_ok());
    }
    #[tokio::test]
    async fn dns_loopback_is_rejected_before_sending() {
        assert!(client("https://localtest.me/v1", 1).await.is_err());
    }
}
