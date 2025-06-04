use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP, TokenAuth, DB},
    models::Emoji,
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[autodoc("/emojis", category = "Emojis")]
#[get("/<emoji_id>")]
pub async fn get_emoji(
    emoji_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: Option<TokenAuth>,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Json<Emoji>> {
    let mut rate_limiter;
    if let Some(session) = &session {
        rate_limiter = RateLimiter::new("get_emoji", session.0.user_id, conf);
    } else {
        rate_limiter = RateLimiter::new("guest_get_emoji", ip, conf);
    }
    rate_limiter.process_rate_limit(&mut cache).await?;
    let emoji = Emoji::get(emoji_id, &mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;
    rate_limiter.wrap_response(Json(emoji))
}
