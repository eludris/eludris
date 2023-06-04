use argon2::Argon2;
use rand::rngs::StdRng;
use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{UpdateUser, User},
    Conf,
};
use tokio::sync::Mutex;

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

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
    rate_limiter.wrap_response(Json(
        User::update(
            session.0.user_id,
            user.into_inner(),
            hasher.inner(),
            &mut *rng.lock().await,
            &mut db,
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
