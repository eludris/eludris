use std::{io::ErrorKind, path::Path};

use crate::{
    rate_limit::{RateLimitedRouteResponse, RateLimiter},
    Cache,
};
use rocket::{
    http::{ContentType, Header},
    State,
};
use rocket_db_pools::Connection;
use todel::{
    http::ClientIP,
    models::{ErrorResponse, FetchResponse},
    Conf,
};
use tokio::fs::File;

/// Simple struct meant to represent static disk files
struct StaticFile<'a> {
    file: File,
    path: &'a Path,
    content_type: Option<ContentType>,
}

/// Get a static file by its name.
/// Static files are added by the instance owner and cannot be externally modified.
///
/// The `Content-Deposition` header is set to `inline`.
/// Use the [`download_static_file`] endpoint to get `Content-Deposition` set to `attachment`.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl https://cdn.eludris.gay/static/pengin.mp4
///
/// <raw file data>
/// ```
#[autodoc(category = "Files")]
#[get("/<name>", rank = 1)]
pub async fn get_static_file<'a>(
    name: &'a str,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<FetchResponse<'a>> {
    let mut rate_limiter = RateLimiter::new("fetch_file", "static", ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;

    let StaticFile {
        file,
        path,
        content_type,
    } = get_file(name)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;

    rate_limiter.wrap_response(FetchResponse {
        file,
        disposition: Header::new(
            "Content-Disposition",
            format!(
                "inline; filename=\"{}\"",
                path.file_name().unwrap().to_str().unwrap()
            ),
        ),
        content_type: content_type.unwrap_or(ContentType::Any),
    })
}

#[get("/<name>/download", rank = 1)]
pub async fn download_static_file<'a>(
    name: &'a str,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Result<FetchResponse<'a>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new("fetch_file", "static", ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;

    let StaticFile {
        file,
        path,
        content_type,
    } = get_file(name)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;

    rate_limiter.wrap_response(Ok(FetchResponse {
        file,
        disposition: Header::new(
            "Content-Disposition",
            format!(
                "attachment; filename=\"{}\"",
                path.file_name().unwrap().to_str().unwrap()
            ),
        ),
        content_type: content_type.unwrap_or(ContentType::Any),
    }))
}

async fn get_file(name: &str) -> Result<StaticFile, ErrorResponse> {
    let path = Path::new(name)
        .file_name()
        .map(Path::new)
        .ok_or_else(|| error!(VALIDATION, "name", "Invalid file name"))?;

    let extension = path.extension();
    let content_type = match extension {
        Some(extension) => ContentType::from_extension(
            extension
                .to_str()
                .ok_or_else(|| error!(VALIDATION, "name", "Invalid file extension"))?,
        ),
        None => None,
    };

    let file = File::open(Path::new("./files/static").join(path))
        .await
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                error!(NOT_FOUND)
            } else {
                error!(SERVER, "Failed to get static file from storage")
            }
        })?;

    log::debug!("Fetched static file {}", name);

    Ok(StaticFile {
        file,
        path,
        content_type,
    })
}
