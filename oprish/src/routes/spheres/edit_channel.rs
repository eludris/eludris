use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{ErrorResponse, ServerPayload, Sphere, SphereChannel, SphereChannelEdit},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Edit a channel.
///
/// Name must be less than or equal to 32 characters long if provided.
///
/// Topic must be less than or equal to 4096 characters long if provided.
///
/// Position must be greater than or equal to 0.
/// It is automatically upper-bounded to the number of channels in the category.
///
/// If category_id is provided, position *must* also be provided.
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
    let mut rate_limiter = RateLimiter::new("edit_channel", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    let sphere = Sphere::get_unpopulated(sphere_id, &mut db)
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

    let (channel, channel_edit) =
        SphereChannel::edit(channel.into_inner(), sphere_id, channel_id, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?;

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::SphereChannelUpdate {
                data: channel_edit,
                channel_id,
                sphere_id,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    rate_limiter.wrap_response(Json(channel))
}
