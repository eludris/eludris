use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{Emoji, EmojiEdit, ErrorResponse, ServerPayload, Sphere},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[autodoc("/emojis", category = "Emojis")]
#[patch("/<emoji_id>", data = "<edit>")]
pub async fn edit_emoji(
    edit: Json<EmojiEdit>,
    emoji_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Result<Json<Emoji>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new("edit_emoji", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    let edit = edit.into_inner();
    let mut emoji = Emoji::get(emoji_id, &mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;

    let sphere = Sphere::get_unpopulated(emoji.sphere_id, &mut db)
        .await
        .map_err(|err| {
            rate_limiter.add_headers(if let ErrorResponse::NotFound { .. } = err {
                error!(VALIDATION, "sphere", "Sphere doesn't exist")
            } else {
                err
            })
        })?;

    if sphere.owner_id != session.0.user_id {
        return Err(rate_limiter.add_headers(error!(FORBIDDEN)));
    }

    emoji
        .edit(&edit, &mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::EmojiUpdate {
                sphere_id: sphere.id,
                emoji_id: emoji.id,
                data: edit,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    rate_limiter.wrap_response(Ok(Json(emoji)))
}
