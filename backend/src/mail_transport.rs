use lettre::{
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        client::{AsyncSmtpConnection, TlsParameters},
        extension::ClientId,
    },
    Message,
};
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_header_injection_and_non_public_servers() {
        assert!(address("user@example.com"));
        assert!(!address("user@example.com\r\nBcc:someone@example.com"));
        assert!(server("smtp.example.com", 465));
        for (host, port) in [
            ("127.0.0.1", 465),
            ("localhost", 587),
            ("169.254.169.254", 465),
            ("smtp.example.com", 25),
            ("user@host.example", 465),
            ("host.example/path", 587),
        ] {
            assert!(!server(host, port));
        }
    }
    #[tokio::test]
    async fn smtp_starttls_cannot_downgrade_or_send_credentials_to_plain_server() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = BufReader::new(socket);
            socket
                .get_mut()
                .write_all(b"220 test ESMTP\r\n")
                .await
                .unwrap();
            let mut line = String::new();
            socket.read_line(&mut line).await.unwrap();
            assert!(line.starts_with("EHLO"));
            socket
                .get_mut()
                .write_all(b"250-test\r\n250 AUTH PLAIN LOGIN\r\n")
                .await
                .unwrap();
            line.clear();
            let _ = tokio::time::timeout(Duration::from_secs(2), socket.read_line(&mut line)).await;
            assert!(!line.starts_with("AUTH"));
            assert!(!line.starts_with("MAIL"));
        });
        let mut conn = AsyncSmtpConnection::connect_tokio1(
            addr,
            Some(Duration::from_secs(1)),
            &ClientId::Domain("promptark.local".into()),
            None,
            None,
        )
        .await
        .unwrap();
        assert!(secure_starttls(&mut conn, "smtp.example.com")
            .await
            .is_err());
        drop(conn);
        server.await.unwrap();
    }
}
pub fn address(value: &str) -> bool {
    value.len() <= 254
        && !value.contains(['\r', '\n', ' '])
        && value.parse::<lettre::Address>().is_ok()
}
pub fn server(host: &str, port: u16) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
        && [465, 587].contains(&port)
        && crate::outbound::validate_url(&format!("https://{host}")).is_ok()
}
async fn secure_starttls(conn: &mut AsyncSmtpConnection, host: &str) -> Result<(), String> {
    if !conn.can_starttls() {
        return Err("SMTP 不支持必需的 STARTTLS，未发送凭据".into());
    }
    conn.starttls(
        TlsParameters::new(host.to_owned()).map_err(|_| "TLS 配置失败")?,
        &ClientId::Domain("promptark.local".into()),
    )
    .await
    .map_err(|_| "SMTP TLS 握手或证书验证失败".into())
}
pub async fn send(
    host: &str,
    port: u16,
    tls: &str,
    username: &str,
    password: &str,
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
    id: &str,
) -> Result<(), String> {
    if !server(host, port)
        || !address(from)
        || !address(to)
        || !["implicit", "starttls"].contains(&tls)
    {
        return Err("邮件服务器或地址配置无效".into());
    }
    let work = async {
        let targets = crate::outbound::resolve(host, port).await?;
        let tls_parameters = if tls == "implicit" {
            Some(TlsParameters::new(host.to_owned()).map_err(|_| "TLS 配置失败")?)
        } else {
            None
        };
        let mut conn = AsyncSmtpConnection::connect_tokio1(
            targets.as_slice(),
            Some(Duration::from_secs(5)),
            &ClientId::Domain("promptark.local".into()),
            tls_parameters,
            None,
        )
        .await
        .map_err(|_| "SMTP 连接或 TLS 证书验证失败")?;
        if tls == "starttls" {
            secure_starttls(&mut conn, host).await?
        }
        if !conn.is_encrypted() {
            return Err("SMTP 连接未加密，未发送凭据".into());
        }
        conn.auth(
            &[Mechanism::Plain, Mechanism::Login],
            &Credentials::new(username.into(), password.into()),
        )
        .await
        .map_err(|_| "SMTP 身份认证失败")?;
        let message = Message::builder()
            .from(from.parse().map_err(|_| "发件地址无效")?)
            .to(to.parse().map_err(|_| "收件地址无效")?)
            .subject(subject)
            .message_id(Some(format!("{id}@promptark.local")))
            .body(body.to_owned())
            .map_err(|_| "邮件格式无效")?;
        conn.send(message.envelope(), &message.formatted())
            .await
            .map_err(|_| "SMTP 拒绝邮件，或未收到接受确认")?;
        // Delivery has been accepted. A slow QUIT must not turn acceptance into a retry.
        Ok(())
    };
    tokio::time::timeout(Duration::from_secs(20), work)
        .await
        .map_err(|_| "SMTP 发送超时；服务器是否已接受未知".to_owned())?
}
