use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{ErrorResponse, Message, ReactionEmojiReference, ServerPayload, SphereChannel},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[autodoc("/channels", category = "Emojis")]
#[delete("/<channel_id>/messages/<message_id>/emojis", data = "<emoji>")]
pub async fn remove_reaction(
    emoji: Json<ReactionEmojiReference>,
    channel_id: u64,
    message_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Result<Json<Message>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new(
        "remove_reaction",
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
    let emoji = message
        .remove_reaction(emoji.into_inner(), session.0.user_id, &mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::MessageReactionDelete {
                channel_id,
                message_id,
                user_id: session.0.user_id,
                emoji,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    rate_limiter.wrap_response(Ok(Json(message)))
}
