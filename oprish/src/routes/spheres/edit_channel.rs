use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{SphereChannel, SphereChannelEdit},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Edit a channel.
///
/// -- STATUS: 201
/// -----
///
/// ### Example
///
/// ```sh
/// curl --request PATCH \
///   -H "Authorization: <token>" \
///   --json '{"name":"Bean","position":42,"category_id":1337}' \
///   https://api.eludris.gay/spheres/1234/channels/5678
/// ```
#[autodoc("/spheres", category = "Spheres")]
#[patch("/<sphere_id>/channels/<channel_id>", data = "<channel>")]
pub async fn edit_channel(
    channel: Json<SphereChannelEdit>,
    sphere_id: u64,
    channel_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<SphereChannel>> {
    let mut rate_limiter = RateLimiter::new("edit_category", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        SphereChannel::edit(channel.into_inner(), sphere_id, channel_id, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
