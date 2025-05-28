use rocket::{http::Status, response::status::Custom, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{ErrorResponse, Message, ServerPayload, SphereChannel},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[autodoc("/channels", category = "Messaging")]
#[delete("/<channel_id>/messages/<message_id>")]
pub async fn delete_message(
    channel_id: u64,
    message_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Result<Custom<()>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new("delete_message", session.0.user_id, conf);
    if !SphereChannel::has_member(channel_id, session.0.user_id, &mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?
    {
        error!(rate_limiter, UNAUTHORIZED);
    }
    rate_limiter.process_rate_limit(&mut cache).await?;
    let mut cache = cache.into_inner();
    let message = Message::get(message_id, &mut db, &mut cache)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::MessageDelete {
                channel_id,
                message_id,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    rate_limiter.wrap_response(
        message
            .delete(&mut db)
            .await
            .map(|_| Custom(Status::NoContent, ())),
    )
}
