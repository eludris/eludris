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

/// Create a new sphere.
///
/// A user can only own up to 100 spheres at once, if you, for any reason need
/// to create more, you'll need to delete some of your spheres.
///
/// If you still aren't allowed to create a sphere after doing so, then you will
/// have to wait for the server to run its scheduled cleanup to actually remove
/// the old sphere data from the database. This limitation is imposed to avoid
/// abuse.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   --json '{"slug":"horse","type":"Hybrid"}'
///   https://api.eludris.gay/users/profile
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
///       "sphere": 4204171493377,
///       "name": "general",
///       "position": 0
///     }]
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
