use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP, SphereIdentifier, TokenAuth, DB},
    models::Sphere,
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Get a sphere's data using a [`SphereIdentifier`].
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/spheres/4204171493377
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
#[get("/<identifier>")]
pub async fn get_sphere(
    identifier: SphereIdentifier,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: Option<TokenAuth>,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Json<Sphere>> {
    let mut rate_limiter;
    if let Some(session) = &session {
        rate_limiter = RateLimiter::new("get_sphere", session.0.user_id, conf);
    } else {
        rate_limiter = RateLimiter::new("guest_get_sphere", ip, conf);
    }
    rate_limiter.process_rate_limit(&mut cache).await?;
    let sphere = match identifier {
        SphereIdentifier::ID(id) => Sphere::get(id, &mut db, &mut cache.into_inner()).await,
        SphereIdentifier::Slug(slug) => {
            Sphere::get_slug(slug.to_string(), &mut db, &mut cache.into_inner()).await
        }
    }
    .map_err(|err| rate_limiter.add_headers(err))?;
    rate_limiter.wrap_response(Json(sphere))
}
