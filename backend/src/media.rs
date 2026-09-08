use crate::AppState;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::borrow::Cow;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

#[derive(Clone)]
pub struct MediaConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}

impl MediaConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            endpoint: std::env::var("PROMPTARK_MEDIA_ENDPOINT")
                .or_else(|_| std::env::var("PL_MEDIA_ENDPOINT"))
                .unwrap_or_else(|_| "http://127.0.0.1:9000".into()),
            access_key: std::env::var("PROMPTARK_MEDIA_ACCESS_KEY")
                .or_else(|_| std::env::var("PL_MEDIA_ACCESS_KEY"))
                .unwrap_or_else(|_| "minio".into()),
            secret_key: std::env::var("PROMPTARK_MEDIA_SECRET_KEY")
                .or_else(|_| std::env::var("PL_MEDIA_SECRET_KEY"))
                .unwrap_or_else(|_| "minio123".into()),
            bucket: std::env::var("PROMPTARK_MEDIA_BUCKET")
                .or_else(|_| std::env::var("PL_MEDIA_BUCKET"))
                .unwrap_or_else(|_| "prompt-launcher-media".into()),
        })
    }

    fn bucket(&self) -> Result<(Bucket, Credentials), StatusCode> {
        let endpoint = Url::parse(&self.endpoint).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let bucket = Bucket::new(
            endpoint,
            UrlStyle::Path,
            Cow::Owned(self.bucket.clone()),
            Cow::Owned("us-east-1".into()),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok((bucket, self.credentials()))
    }

    fn credentials(&self) -> Credentials {
        Credentials::new(self.access_key.clone(), self.secret_key.clone())
    }

    pub async fn ping(&self) -> bool {
        let Ok((bucket, creds)) = self.bucket() else {
            return false;
        };
        let action = bucket.head_bucket(Some(&creds));
        let url = action.sign(Duration::from_secs(60));
        let Ok(client) = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(3))
            .build()
        else {
            return false;
        };
        client
            .head(url)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
    }
}

#[derive(Serialize)]
pub struct MediaUpload {
    pub id: String,
    pub url: String,
    pub name: String,
    pub mime: String,
    pub size: usize,
    pub sha256: String,
}

#[derive(Serialize)]
pub struct MediaUrl {
    pub url: String,
}

pub(crate) const MAX_FILE: usize = 5 * 1024 * 1024;
const MAX_ACCOUNT_BYTES: i64 = 100 * 1024 * 1024;

fn storage_client() -> Result<reqwest::Client, StatusCode> {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}

pub(crate) fn validate_file(name: &str, mime: &str, bytes: &[u8]) -> Result<(), StatusCode> {
    if name.trim().is_empty()
        || name.chars().count() > 255
        || name
            .chars()
            .any(|c| c.is_control() || c == '/' || c == '\\')
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if bytes.len() > MAX_FILE {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let extension = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    let valid = match (extension.as_str(), mime) {
        ("png", "image/png") => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        ("jpg" | "jpeg", "image/jpeg") => bytes.starts_with(&[255, 216, 255]),
        ("gif", "image/gif") => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        ("webp", "image/webp") => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"),
        ("pdf", "application/pdf") => bytes.starts_with(b"%PDF-"),
        ("txt" | "md" | "csv" | "json", "text/plain") => {
            std::str::from_utf8(bytes).is_ok() && !bytes.contains(&0)
        }
        ("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document")
        | ("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        | ("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation") => {
            bytes.starts_with(b"PK\x03\x04")
        }
        _ => false,
    };
    if !valid {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    Ok(())
}

pub(crate) async fn reserve(
    pg: &crate::postgres::Pg,
    owner: &str,
    key: &str,
    file: &MediaUpload,
) -> Result<(), StatusCode> {
    let mut tx = pg
        .pool
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    // Lock the owner, not existing objects: two first uploads must also serialize.
    sqlx::query(&format!(
        "SELECT email FROM {} WHERE email=$1 FOR UPDATE",
        pg.t("accounts")
    ))
    .bind(owner)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let usage = sqlx::query(&format!("SELECT COUNT(*) AS count, COALESCE(SUM(size),0)::BIGINT AS bytes FROM {} WHERE owner_email=$1", pg.t("media_objects")))
        .bind(owner).fetch_one(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if usage.get::<i64, _>("count") >= 256
        || usage.get::<i64, _>("bytes") + file.size as i64 > MAX_ACCOUNT_BYTES
    {
        return Err(StatusCode::CONFLICT);
    }
    sqlx::query(&format!("INSERT INTO {} (id,owner_email,object_key,content_type,file_name,size,sha256,ready) VALUES ($1,$2,$3,$4,$5,$6,$7,FALSE)", pg.t("media_objects")))
        .bind(&file.id).bind(owner).bind(key).bind(&file.mime).bind(&file.name).bind(file.size as i64).bind(&file.sha256)
        .execute(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}

pub async fn upload(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<MediaUpload>, StatusCode> {
    let email = crate::require_user(&state, &headers).await?;
    let media = state
        .media
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut file = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| error.status())?
    {
        if field.name() != Some("file") || file.is_some() {
            return Err(StatusCode::BAD_REQUEST);
        }
        let name = field
            .file_name()
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_string();
        let mime = field
            .content_type()
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_string();
        let bytes = field.bytes().await.map_err(|error| error.status())?;
        validate_file(&name, &mime, &bytes)?;
        file = Some((name, mime, bytes));
    }
    let (name, mime, bytes) = file.ok_or(StatusCode::BAD_REQUEST)?;
    let id = format!("media.{}", Uuid::new_v4());
    let key = format!("promptark/{id}");
    let result = MediaUpload {
        url: format!("/v1/media/{id}/content"),
        id,
        name,
        mime,
        size: bytes.len(),
        sha256: format!("{:x}", Sha256::digest(&bytes)),
    };
    let (bucket, creds) = media.bucket()?;
    let client = storage_client()?;
    reserve(pg, &email, &key, &result).await?;
    let put = bucket.put_object(Some(&creds), &key);
    let url = put.sign(Duration::from_secs(60));
    let transfer = async {
        let response = client
            .put(url)
            .body(bytes)
            .send()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;
        if !response.status().is_success() {
            return Err(StatusCode::BAD_GATEWAY);
        }
        crate::require_user(&state, &headers).await?;
        let changed = sqlx::query(&format!(
            "UPDATE {} SET ready=TRUE WHERE id=$1 AND owner_email=$2",
            pg.t("media_objects")
        ))
        .bind(&result.id)
        .bind(&email)
        .execute(&pg.pool)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        if changed.rows_affected() != 1 {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
        Ok(())
    }
    .await;
    if let Err(error) = transfer {
        // Only this attempt's generated key is eligible for cleanup, never a user path.
        let delete = bucket.delete_object(Some(&creds), &key);
        let cleaned = client
            .delete(delete.sign(Duration::from_secs(60)))
            .send()
            .await
            .is_ok_and(|response| {
                response.status().is_success() || response.status() == StatusCode::NOT_FOUND
            });
        if cleaned {
            let _ = sqlx::query(&format!(
                "DELETE FROM {} WHERE id=$1 AND owner_email=$2",
                pg.t("media_objects")
            ))
            .bind(&result.id)
            .bind(&email)
            .execute(&pg.pool)
            .await;
        }
        return Err(error);
    }
    Ok(Json(result))
}

pub async fn private_url(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<MediaUrl>, StatusCode> {
    let email = crate::require_user(&state, &headers).await?;
    owned(&state, &email, &id).await?;
    Ok(Json(MediaUrl {
        url: format!("/v1/media/{id}/content"),
    }))
}

async fn owned(
    state: &AppState,
    email: &str,
    id: &str,
) -> Result<sqlx::postgres::PgRow, StatusCode> {
    let pg = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let row = sqlx::query(&format!("SELECT object_key,file_name,content_type,size,sha256,ready FROM {} WHERE id=$1 AND owner_email=$2", pg.t("media_objects")))
        .bind(id).bind(email).fetch_optional(&pg.pool).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?.ok_or(StatusCode::NOT_FOUND)?;
    if row.get::<Option<String>, _>("sha256").is_none()
        || row.get::<Option<i64>, _>("size").is_none()
        || row.get::<Option<String>, _>("file_name").is_none()
    {
        return Err(StatusCode::CONFLICT);
    }
    if !row.get::<bool, _>("ready") {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(row)
}

pub async fn download(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Response, StatusCode> {
    let email = crate::require_user(&state, &headers).await?;
    let row = owned(&state, &email, &id).await?;
    let size: i64 = row.get("size");
    if !(0..=MAX_FILE as i64).contains(&size) {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let media = state
        .media
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let key: String = row.get("object_key");
    let (bucket, creds) = media.bucket()?;
    let get = bucket.get_object(Some(&creds), &key);
    let mut response = storage_client()?
        .get(get.sign(Duration::from_secs(60)))
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() || response.content_length().is_some_and(|n| n > size as u64)
    {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let mut bytes = Vec::with_capacity(size as usize);
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?
    {
        if bytes.len() + chunk.len() > size as usize {
            return Err(StatusCode::BAD_GATEWAY);
        }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.len() != size as usize
        || format!("{:x}", Sha256::digest(&bytes)) != row.get::<String, _>("sha256")
    {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let name: String = row.get("file_name");
    let mime = row
        .get::<Option<String>, _>("content_type")
        .ok_or(StatusCode::BAD_GATEWAY)?;
    validate_file(&name, &mime, &bytes).map_err(|_| StatusCode::BAD_GATEWAY)?;
    crate::require_user(&state, &headers).await?;
    Ok((
        [
            ("content-type", "application/octet-stream".to_string()),
            (
                "content-disposition",
                format!(
                    "attachment; filename*=UTF-8''{}",
                    urlencoding::encode(&name)
                ),
            ),
            ("x-content-type-options", "nosniff".into()),
            ("cache-control", "no-store".into()),
        ],
        bytes,
    )
        .into_response())
}
