use rocket::{form::Form, serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::ClientIP,
    ids::IdGenerator,
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
    gen: &State<Mutex<IdGenerator>>,
) -> RateLimitedRouteResponse<Json<FileData>> {
    let mut rate_limiter = RateLimiter::new("attachments", "attachments", ip, conf.inner());
    rate_limiter
        .process_rate_limit(upload.file.len(), &mut cache)
        .await?;
    let upload = upload.into_inner();
    let file = File::create(
        upload.file,
        "attachments".to_string(),
        &mut *gen.inner().lock().await,
        &mut db,
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
#[get("/<id>?<size>")]
pub async fn get_attachment<'a>(
    id: u64,
    ip: ClientIP,
    size: Option<u32>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<FetchResponse<'a>> {
    let mut rate_limiter = RateLimiter::new("fetch_file", "attachments", ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;
    let file = File::fetch_file(id, "attachments", size, &mut db)
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
#[get("/<id>/download?<size>")]
pub async fn download_attachment<'a>(
    id: u64,
    ip: ClientIP,
    size: Option<u32>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<FetchResponse<'a>> {
    let mut rate_limiter = RateLimiter::new("fetch_file", "attachments", ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;
    let file = File::fetch_file_download(id, "attachments", size, &mut db)
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
