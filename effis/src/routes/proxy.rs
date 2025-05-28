use reqwest::header::USER_AGENT;
use reqwest::Client;
use rocket::http::ContentType;
use rocket::State;
use rocket_db_pools::Connection;
use todel::http::ClientIP;
use todel::models::ErrorResponse;
use todel::Conf;

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};
use crate::Cache;

#[derive(Debug, Responder)]
pub struct ProxyResponse {
    pub file: Vec<u8>,
    pub content_type: ContentType,
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
#[autodoc(category = "Proxy")]
#[get("/proxy?<url>")]
pub async fn proxy<'a>(
    url: String,
    ip: ClientIP,
    mut cache: Connection<Cache>,
    http: &State<Client>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Result<ProxyResponse, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new("proxy_file", "attachments", ip, conf.inner());
    rate_limiter.process_rate_limit(0, &mut cache).await?;
    let resp = http
        .get(url)
        .header(
            USER_AGENT,
            "Mozilla/5.0 (compatible; eludris/0.4.0-alpha1;)",
        )
        .send()
        .await
        .map_err(|_| {
            rate_limiter.add_headers(error!(
                VALIDATION,
                "url", "Couldn't fetch data from the provided URL"
            ))
        })?;
    if resp.content_length() > Some(conf.effis.proxy_file_size) {
        error!(
            rate_limiter,
            VALIDATION, "data", "Proxied data exceeds file limit"
        );
    }
    let content_type = resp
        .headers()
        .get("CONTENT-TYPE")
        .ok_or_else(|| {
            rate_limiter.add_headers(error!(
                VALIDATION,
                "data", "Proxied data doesn't provide a file type"
            ))
        })?
        .to_str()
        .map_err(|_| {
            rate_limiter.add_headers(error!(
                VALIDATION,
                "data", "Proxied data provides an invalid file type header"
            ))
        })?
        .to_string();

    match content_type.as_str() {
        "image/gif" | "image/jpeg" | "image/png" | "image/webp" | "video/mp4" | "video/webm"
        | "video/quicktime" => {}
        _ => {
            error!(
                rate_limiter,
                VALIDATION, "data", "Proxied data uses a non-allowed content type"
            );
        }
    }

    rate_limiter.wrap_response(Ok(ProxyResponse {
        file: resp
            .bytes()
            .await
            .map_err(|_| {
                rate_limiter.add_headers(error!(
                    VALIDATION,
                    "url", "Couldn't fetch data from the provided URL"
                ))
            })?
            .to_vec(),
        content_type: ContentType::parse_flexible(&content_type).unwrap(),
    }))
}
