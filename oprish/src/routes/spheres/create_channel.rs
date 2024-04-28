use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    ids::IdGenerator,
    models::{SphereChannel, SphereChannelCreate},
    Conf,
};
use tokio::sync::Mutex;

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Create a new channel within a sphere.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   --json '{"name":"horse","type":"TEXT"}'
///   https://api.eludris.gay/spheres/4204171493377/channels
///
/// {
///   "type": "TEXT",
///   "id": 4204171493378,
///   "sphere_id": 4204171493377,
///   "name": "horse",
///   "position": 0
/// }
/// ```
#[autodoc("/spheres", category = "Spheres")]
#[post("/<sphere_id>/channels", data = "<channel>")]
pub async fn create_channel(
    channel: Json<SphereChannelCreate>,
    sphere_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    id_generator: &State<Mutex<IdGenerator>>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<SphereChannel>> {
    let mut rate_limiter = RateLimiter::new("create_channel", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        SphereChannel::create(
            channel.into_inner(),
            sphere_id,
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
