use crate::AppState;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};
use serde::Serialize;
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
        reqwest::Client::new().head(url).send().await.is_ok()
    }
}

#[derive(Serialize)]
pub struct MediaUpload {
    pub id: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct MediaUrl {
    pub url: String,
}

pub async fn upload(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<MediaUpload>, StatusCode> {
    let email = crate::require_user(&state, &headers).await?;
    let media = state.media.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut bytes = None;
    let mut content_type = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if field.name() == Some("file") {
            content_type = field.content_type().map(str::to_string);
            bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?
                    .to_vec(),
            );
        }
    }
    let bytes = bytes.ok_or(StatusCode::BAD_REQUEST)?;
    let id = format!("media.{}", Uuid::new_v4());
    let key = format!("promptark/{id}");
    let (bucket, creds) = media.bucket()?;
    let put = bucket.put_object(Some(&creds), &key);
    let url = put.sign(Duration::from_secs(60));
    let response = reqwest::Client::new()
        .put(url)
        .body(bytes)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    if let Some(pg) = &state.db {
        pg.insert_media(&id, &email, &key, content_type.as_deref())
            .await?;
    }
    let get = bucket.get_object(Some(&creds), &key);
    let signed = get.sign(Duration::from_secs(600));
    Ok(Json(MediaUpload {
        id,
        url: signed.to_string(),
    }))
}

pub async fn signed_url(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MediaUrl>, StatusCode> {
    let media = state.media.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let key = if let Some(pg) = &state.db {
        pg.media_key(&id).await?.ok_or(StatusCode::NOT_FOUND)?
    } else {
        return Err(StatusCode::NOT_FOUND);
    };
    let (bucket, creds) = media.bucket()?;
    let get = bucket.get_object(Some(&creds), &key);
    Ok(Json(MediaUrl {
        url: get.sign(Duration::from_secs(600)).to_string(),
    }))
}
