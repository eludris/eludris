use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{ServerPayload, Sphere},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Join a sphere using its ID.
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
#[get("/<id>/join")]
pub async fn join_sphere(
    id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<Sphere>> {
    let mut rate_limiter = RateLimiter::new("join_sphere", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    let member = Sphere::join(id, session.0.user_id, &mut db)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;
    let mut cache = cache.into_inner();
    let sphere = Sphere::get(id, &mut db, &mut cache)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;
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

/// Join a sphere using its slug.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/spheres/horse/join
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
#[get("/<slug>/join", rank = 1)]
pub async fn join_sphere_from_slug(
    slug: &str,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<Sphere>> {
    let mut rate_limiter = RateLimiter::new("join_sphere", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    let member = Sphere::join_slug(slug.to_string(), session.0.user_id, &mut db)
        .await
        .map_err(|e| rate_limiter.add_headers(e))?;
    let mut cache = cache.into_inner();
    let sphere = Sphere::get_slug(slug.to_string(), &mut db, &mut cache)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;
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
