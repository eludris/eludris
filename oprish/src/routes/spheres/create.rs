use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    ids::IdGenerator,
    models::{Sphere, SphereCreate},
    Conf,
};
use tokio::sync::Mutex;

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Modify your profile.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   -X PATCH
///   --json '{"display_name":"HappyRu","bio":"I am very happy!"}'
///   https://api.eludris.gay/users/profile
///
/// {
///   "id": 2346806935553
///   "username": "yendri"
///   "display_name": "HappyRu"
///   "social_credit": 0,
///   "bio": "I am very happy!"
///   "badges": 0,
///   "permissions": 0
/// }
/// ```
#[autodoc("/spheres", category = "Spheres")]
#[post("/", data = "<sphere>")]
pub async fn create_sphere(
    sphere: Json<SphereCreate>,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    id_generator: &State<Mutex<IdGenerator>>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<Sphere>> {
    let mut rate_limiter = RateLimiter::new("create_sphere", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        Sphere::create(
            sphere.into_inner(),
            session.0.user_id,
            &mut *id_generator.lock().await,
            &mut db,
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    ))
    // cache
    // .publish::<&str, String, ()>("eludris-events", serde_json::to_string(&payload).unwrap())
    // .await
    // .unwrap();
    // if let ServerPayload::UserUpdate(user) = payload {
    // rate_limiter.wrap_response(Json(user))
    // } else {
    // unreachable!()
    // }
}
