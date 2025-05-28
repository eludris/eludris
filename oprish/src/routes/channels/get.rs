use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP, TokenAuth, DB},
    models::SphereChannel,
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
#[autodoc("/channels", category = "Channels")]
#[get("/<channel_id>")]
pub async fn get_channel(
    channel_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: Option<TokenAuth>,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Json<SphereChannel>> {
    let mut rate_limiter;
    if let Some(session) = &session {
        rate_limiter = RateLimiter::new("get_channel", session.0.user_id, conf);
    } else {
        rate_limiter = RateLimiter::new("guest_get_channel", ip, conf);
    }
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        SphereChannel::get(channel_id, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
