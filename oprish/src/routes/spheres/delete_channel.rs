use rocket::State;
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{ErrorResponse, ServerPayload, Sphere, SphereChannel},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Delete a channel.
///
/// -- STATUS: 200
/// -----
///
/// ### Example
///
/// ```sh
/// curl --request DELETE \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/spheres/1234/channels/5678
/// ```
#[autodoc("/spheres", category = "Spheres")]
#[delete("/<sphere_id>/channels/<channel_id>")]
pub async fn delete_channel(
    sphere_id: u64,
    channel_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<()> {
    let mut rate_limiter = RateLimiter::new("delete_category", session.0.user_id, conf);
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

    let response = rate_limiter.wrap_response(
        SphereChannel::delete(sphere_id, channel_id, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    );

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::SphereChannelDelete {
                channel_id,
                sphere_id,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    response
}
