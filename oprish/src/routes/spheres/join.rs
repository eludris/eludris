use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, SphereIdentifier, TokenAuth, DB},
    models::{ServerPayload, Sphere},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Join a sphere using a [`SphereIdentifier`].
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/spheres/4204171493377/join
///
/// {
///   "id": 4204171493377,
///   "owner_id": 4203748065281,
///   "slug": "horse",
///   "type": "HYBRID",
///   "badges": 0,
///   "channels": [{
///       "type": "TEXT",
///       "id": 4204171493378,
///       "sphere_id": 4204171493377,
///       "name": "general",
///       "position": 0
///     }]
/// }
/// ```
#[autodoc("/spheres", category = "Spheres")]
#[get("/<identifier>/join")]
pub async fn join_sphere(
    identifier: SphereIdentifier,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<Sphere>> {
    let mut rate_limiter = RateLimiter::new("join_sphere", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    let mut cache = cache.into_inner();
    let sphere = match identifier {
        SphereIdentifier::ID(id) => Sphere::get(id, &mut db, &mut cache).await,
        SphereIdentifier::Slug(slug) => Sphere::get_slug(slug, &mut db, &mut cache).await,
    }
    .map_err(|err| rate_limiter.add_headers(err))?;
    let member = sphere
        .add_member(session.0.user_id, &mut db)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;
    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::SphereMemberJoin {
                user: member.user,
                sphere_id: sphere.id,
            })
            .unwrap(),
        )
        .await
        .unwrap();
    rate_limiter.wrap_response(Json(sphere))
}
