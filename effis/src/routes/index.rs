use rocket::{form::Form, serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::ClientIP,
    ids::IDGenerator,
    models::{FetchResponse, File, FileData, FileUpload},
    Conf,
};
use tokio::sync::Mutex;

use crate::{
    rate_limit::{RateLimitedRouteResponse, RateLimiter},
    Cache, DB,
};

/// Upload an attachment to Effis under a specific bucket.
/// This is a shortcut to [`upload_file`] with the attachments bucket.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -F file=@thang-big.png \
///   -F spoiler=false \
///   https://cdn.eludris.gay/
///
/// {
///   "id": 2199681302540,
///   "name": "thang-big.png",
///   "bucket": "attachments",
///   "metadata": {
///     "type": "image",
///     "width": 702,
///     "height": 702
///   }
/// }
/// ```
#[autodoc(category = "Files")]
#[post("/", data = "<upload>")]
pub async fn upload_attachment<'a>(
    upload: Form<FileUpload<'a>>,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    conf: &State<Conf>,
    gen: &State<Mutex<IDGenerator>>,
) -> RateLimitedRouteResponse<Json<FileData>> {
    let mut rate_limiter = RateLimiter::new("attachments", "attachments", ip, conf.inner());
    rate_limiter
        .process_rate_limit(upload.file.len(), &mut cache)
        .await?;
    let upload = upload.into_inner();
    let file = File::create(
        upload.file,
        "attachments".to_string(),
        gen.inner(),
        &mut db,
        upload.spoiler,
    )
    .await
    .map_err(|e| rate_limiter.add_headers(e))?;
    rate_limiter.wrap_response(Json(file))
}

/// Get an attachment by ID.
/// This is a shortcut to [`get_file`] with the attachments bucket.
///
/// The `Content-Deposition` header is set to `inline`.
/// Use the [`download_attachment`] endpoint to get `Content-Deposition` set to `attachment`.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl https://cdn.eludris.gay/2199681302540
///
/// <raw file data>
/// ```
#[autodoc(category = "Files")]
#[get("/<id>")]
pub async fn get_attachment<'a>(
    id: u64,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<FetchResponse<'a>> {
    let mut rate_limiter = RateLimiter::new("fetch_file", "attachments", ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;
    let file = File::fetch_file(id, "attachments", &mut db)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;
    rate_limiter.wrap_response(file)
}

/// Get an attachment by ID.
/// This is a shortcut to [`download_file`] with the attachments bucket.
///
/// The `Content-Deposition` header is set to `attachment`.
/// Use the [`get_attachment`] endpoint to get `Content-Deposition` set to `inline`.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl https://cdn.eludris.gay/attachments/2199681302540/download
///
/// <raw file data>
/// ```
#[autodoc(category = "Files")]
#[get("/<id>/download")]
pub async fn download_attachment<'a>(
    id: u64,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<FetchResponse<'a>> {
    let mut rate_limiter = RateLimiter::new("fetch_file", "attachments", ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;
    let file = File::fetch_file_download(id, "attachments", &mut db)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;
    rate_limiter.wrap_response(file)
}

/// Get a file's metadata by ID from a specific bucket.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   https://cdn.eludris.gay/2198189244420/data
///
/// {
///   "id": 2199681302540,
///   "name": "thang-big.png",
///   "bucket": "attachments",
///   "metadata": {
///     "type": "image",
///     "width": 702,
///     "height": 702
///   }
/// }
/// ```
#[autodoc(category = "Files")]
#[get("/<id>/data")]
pub async fn get_attachment_data<'a>(
    id: u64,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Json<FileData>> {
    let mut rate_limiter = RateLimiter::new("fetch_file", "attachments", ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;
    let file = File::fetch_file_data(id, "attachments", &mut db)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;
    rate_limiter.wrap_response(Json(file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rocket;
    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
    };
    use todel::models::{FileData, FileMetadata};
    use tokio::fs;

    async fn test_upload_file(client: &Client, file_name: &str, spoiler: bool) -> FileData {
        let file_data = fs::read(format!("../tests/{}", file_name)).await.unwrap();

        let body: Vec<u8> = [
            "--BOUNDARY\r\n".bytes().collect(),
            format!(
                r#"Content-Disposition: form-data; name="file"; filename="{}""#,
                file_name
            )
            .bytes()
            .collect(),
            "\r\nContent-Type: text/plain\r\n\r\n".bytes().collect(),
            file_data.to_vec(),
            "\r\n--BOUNDARY\r\n".bytes().collect(),
            r#"Content-Disposition: form-data; name="spoiler""#
                .bytes()
                .collect(),
            "\r\n\r\n".bytes().collect(),
            spoiler.to_string().bytes().collect(),
            "\r\n--BOUNDARY--\r\n\r\n".bytes().collect(),
        ]
        .into_iter()
        .flatten()
        .collect();

        let response = client
            .post(uri!(upload_attachment))
            .header(ContentType::parse_flexible("multipart/form-data; boundary=BOUNDARY").unwrap())
            .body(body)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let data: FileData = response.into_json().await.unwrap();

        assert_eq!(data.name, file_name);
        assert_eq!(data.spoiler, spoiler);
        assert_eq!(data.bucket, "attachments");

        let response = client
            .get(uri!(get_attachment_data(data.id)))
            .dispatch()
            .await;
        assert_eq!(response.into_json::<FileData>().await.unwrap(), data);

        let response = client.get(uri!(get_attachment(data.id))).dispatch().await;
        assert_eq!(response.into_bytes().await.unwrap(), file_data);

        let response = client
            .get(uri!(download_attachment(data.id)))
            .dispatch()
            .await;
        assert_eq!(response.into_bytes().await.unwrap(), file_data);

        data
    }

    // #[rocket::async_test]
    async fn test_index() {
        let client = Client::untracked(rocket().unwrap()).await.unwrap();

        let data = test_upload_file(&client, "test-text.txt", false).await;

        assert_eq!(data.metadata, FileMetadata::Text);

        let data = test_upload_file(&client, "test-text.txt", true).await;

        assert_eq!(data.metadata, FileMetadata::Text);

        let data = test_upload_file(&client, "test-image.png", true).await;

        assert_eq!(
            data.metadata,
            FileMetadata::Image {
                width: Some(280),
                height: Some(280)
            }
        );

        let data = test_upload_file(&client, "test-video.mp4", false).await;

        assert_eq!(
            data.metadata,
            FileMetadata::Video {
                width: Some(8),
                height: Some(8)
            }
        );

        let data = test_upload_file(&client, "test-other", false).await;

        assert_eq!(data.metadata, FileMetadata::Other);
    }
}
