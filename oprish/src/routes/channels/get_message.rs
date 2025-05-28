use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{ErrorResponse, Message, SphereChannel},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[autodoc("/channels", category = "Messaging")]
#[get("/<channel_id>/messages/<message_id>")]
pub async fn get_message(
    channel_id: u64,
    message_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Result<Json<Message>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new("get_message", session.0.user_id, conf);
    if !SphereChannel::has_member(channel_id, session.0.user_id, &mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?
    {
        error!(rate_limiter, UNAUTHORIZED);
    }
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Ok(Json(
        Message::get(message_id, &mut db, &mut cache.into_inner())
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    )))
}
