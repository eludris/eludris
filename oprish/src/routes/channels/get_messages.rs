use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::Message,
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Get a channel's data using its ID.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/channels/4204171393377
///
/// {
///   "type": "TEXT",
///   "id": 4204171493378,
///   "sphere_id": 4204171493377,
///   "name": "general",
///   "position": 0
/// }
/// ```
#[autodoc("/channels", category = "Messaging")]
#[get("/<channel_id>/messages?<before>&<after>&<limit>")]
pub async fn get_messages(
    channel_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
    before: Option<u64>,
    after: Option<u64>,
    limit: Option<u32>,
) -> RateLimitedRouteResponse<Json<Vec<Message>>> {
    let mut rate_limiter = RateLimiter::new("get_messages", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        Message::get_history(
            channel_id,
            &mut db,
            &mut cache.into_inner(),
            before,
            after,
            limit,
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
