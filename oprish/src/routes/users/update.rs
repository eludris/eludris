use argon2::Argon2;
use rand::rngs::StdRng;
use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{ServerPayload, UpdateUser, User},
    Conf,
};
use tokio::sync::Mutex;

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Modify your user account.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   -X PATCH
///   --json '{"email":"nicolas.maduro@presidencia.gob.ve","username":"nicolas"}'
///   https://api.eludris.gay/users
///
/// {
///   "id": 2346806935553
///   "username": "nicolas"
///   "display_name": "HappyRu"
///   "social_credit": 0,
///   "bio": "I am very happy!"
///   "badges": 0,
///   "permissions": 0
/// }
/// ```
#[autodoc("/users", category = "Users")]
#[patch("/", data = "<user>")]
pub async fn update_user(
    user: Json<UpdateUser>,
    hasher: &State<Argon2<'static>>,
    rng: &State<Mutex<StdRng>>,
    conf: &State<Conf>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<User>> {
    let mut rate_limiter = RateLimiter::new("update_user", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    let payload = ServerPayload::UserUpdate(
        User::update(
            session.0.user_id,
            user.into_inner(),
            hasher.inner(),
            &mut *rng.lock().await,
            &mut db,
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    );
    cache
        .publish::<&str, String, ()>("eludris-events", serde_json::to_string(&payload).unwrap())
        .await
        .unwrap();
    if let ServerPayload::UserUpdate(user) = payload {
        rate_limiter.wrap_response(Json(user))
    } else {
        unreachable!()
    }
}
