use rocket::{http::Status, response::status::Custom, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, SphereIdentifier, TokenAuth, UserIdentifier, DB},
    models::{ErrorResponse, ServerPayload, Sphere, User},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Leave a sphere using a [`SphereIdentifier`].
///
/// -----
///
/// ### Example
///
/// ```sh
/// ```
#[autodoc("/spheres", category = "Spheres")]
#[delete("/<sphere_identifier>/members/<user_identifier>")]
pub async fn remove_member(
    sphere_identifier: SphereIdentifier,
    user_identifier: UserIdentifier,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Result<Custom<()>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new("join_sphere", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    let mut cache = cache.into_inner();
    let sphere = match sphere_identifier {
        SphereIdentifier::ID(id) => Sphere::get(id, &mut db, &mut cache).await,
        SphereIdentifier::Slug(slug) => Sphere::get_slug(slug, &mut db, &mut cache).await,
    }
    .map_err(|err| rate_limiter.add_headers(err))?;
    let user_id = match user_identifier {
        UserIdentifier::Me => session.0.user_id,
        UserIdentifier::ID(id) => id,
        UserIdentifier::Username(username) => {
            User::get_username(&username, None, &mut db, &mut cache)
                .await
                .map_err(|err| rate_limiter.add_headers(err))?
                .id
        }
    };
    if session.0.user_id != sphere.owner_id && user_id != session.0.user_id {
        error!(rate_limiter, UNAUTHORIZED);
    }
    sphere
        .remove_member(user_id, &mut db)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;
    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::SphereMemberLeave {
                user_id,
                sphere_id: sphere.id,
            })
            .unwrap(),
        )
        .await
        .unwrap();
    rate_limiter.wrap_response(Ok(Custom(Status::NoContent, ())))
}
