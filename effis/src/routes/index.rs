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
