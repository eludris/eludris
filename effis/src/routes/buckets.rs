use rocket::{form::Form, serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::ClientIP,
    ids::IDGenerator,
    models::{ErrorResponse, FetchResponse, File, FileData, FileUpload},
    Conf,
};
use tokio::sync::Mutex;

use crate::{
    rate_limit::{RateLimitedRouteResponse, RateLimiter},
    Cache, BUCKETS, DB,
};

/// Upload a file to Effis under a specific bucket.
/// At the moment, only the attachments bucket is supported.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -F file=@trolley.mp4 \
///   -F spoiler=true \
///   https://cdn.eludris.gay/attachments/
///
/// {
///   "id": 2198189244420,
///   "name": "trolley.mp4",
///   "bucket": "attachments",
///   "spoiler": true,
///   "metadata": {
///     "type": "video",
///     "width": 576,
///     "height": 682
///   }
/// }
/// ```
#[autodoc(category = "Files")]
#[post("/<bucket>", data = "<upload>")]
pub async fn upload_file<'a>(
    bucket: &'a str,
    upload: Form<FileUpload<'a>>,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    conf: &State<Conf>,
    gen: &State<Mutex<IDGenerator>>,
) -> RateLimitedRouteResponse<Json<FileData>> {
    let mut rate_limiter = RateLimiter::new("attachments", bucket, ip, conf.inner());
    rate_limiter
        .process_rate_limit(upload.file.len(), &mut cache)
        .await?;
    check_bucket(bucket).map_err(|e| rate_limiter.add_headers(e))?;
    let upload = upload.into_inner();
    let file = File::create(
        upload.file,
        bucket.to_string(),
        gen.inner(),
        &mut db,
        upload.spoiler,
    )
    .await
    .map_err(|e| rate_limiter.add_headers(e))?;
    rate_limiter.wrap_response(Json(file))
}

#[get("/<bucket>/<id>")]
pub async fn get<'a>(
    bucket: &'a str,
    id: u128,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<FetchResponse<'a>> {
    let mut rate_limiter = RateLimiter::new("fetch_file", bucket, ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;
    check_bucket(bucket).map_err(|e| rate_limiter.add_headers(e))?;
    let file = File::fetch_file(id, bucket, &mut db)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;
    rate_limiter.wrap_response(file)
}

#[get("/<bucket>/<id>/download")]
pub async fn download<'a>(
    bucket: &'a str,
    id: u128,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<FetchResponse<'a>> {
    let mut rate_limiter = RateLimiter::new("fetch_file", bucket, ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;
    check_bucket(bucket).map_err(|e| rate_limiter.add_headers(e))?;
    let file = File::fetch_file_download(id, bucket, &mut db)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;
    rate_limiter.wrap_response(file)
}

#[get("/<bucket>/<id>/data")]
pub async fn get_data<'a>(
    bucket: &'a str,
    id: u128,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Json<FileData>> {
    let mut rate_limiter = RateLimiter::new("fetch_file", bucket, ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;
    check_bucket(bucket).map_err(|e| rate_limiter.add_headers(e))?;
    let file = File::fetch_file_data(id, bucket, &mut db)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;
    rate_limiter.wrap_response(Json(file))
}

fn check_bucket(bucket: &str) -> Result<(), ErrorResponse> {
    if !BUCKETS.contains(&bucket) {
        return Err(error!(VALIDATION, "bucket", "Unknown bucket"));
    }
    Ok(())
}
