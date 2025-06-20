use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{ErrorResponse, Message, ServerPayload, SphereChannel},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[autodoc("/channels", category = "Emojis")]
#[delete("/<channel_id>/messages/<message_id>/reactions/clear")]
pub async fn clear_reactions(
    channel_id: u64,
    message_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Result<Json<Message>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new(
        "clear_reactions",
        format!("{}:{}", channel_id, session.0.user_id),
        conf,
    );
    rate_limiter.process_rate_limit(&mut cache).await?;

    if !SphereChannel::has_member(channel_id, session.0.user_id, &mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?
    {
        error!(rate_limiter, UNAUTHORIZED);
    }

    let mut cache = cache.into_inner();
    let mut message = Message::get(message_id, &mut db, &mut cache)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;
    message
        .clear_reactions(&mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::MessageReactionClear {
                channel_id,
                message_id,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    rate_limiter.wrap_response(Ok(Json(message)))
}
