use argon2::Argon2;
use rand::rngs::StdRng;
use rocket::{http::Status, response::status::Custom, serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, DB},
    models::{CreatePasswordResetCode, Emailer, ResetPassword, User},
    Conf,
};
use tokio::sync::Mutex;

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[post("/reset-password", data = "<create_code>")]
pub async fn create_password_reset_code(
    create_code: Json<CreatePasswordResetCode>,
    conf: &State<Conf>,
    rng: &State<Mutex<StdRng>>,
    emailer: &State<Emailer>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
) -> RateLimitedRouteResponse<Custom<()>> {
    let mut rate_limiter = RateLimiter::new("create_password_reset_code", &create_code.email, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(Custom(
        Status::NoContent,
        User::create_password_reset_code(
            create_code.into_inner(),
            &mut *rng.lock().await,
            conf,
            emailer.inner(),
            &mut db,
            &mut cache.into_inner(),
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}

#[patch("/reset-password", data = "<reset>")]
pub async fn reset_password(
    reset: Json<ResetPassword>,
    conf: &State<Conf>,
    rng: &State<Mutex<StdRng>>,
    hasher: &State<Argon2<'static>>,
    mailer: &State<Emailer>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
) -> RateLimitedRouteResponse<Custom<()>> {
    let mut rate_limiter = RateLimiter::new("reset_password", &reset.email, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(Custom(
        Status::NoContent,
        User::reset_password(
            reset.into_inner(),
            hasher.inner(),
            &mut *rng.lock().await,
            mailer,
            conf,
            &mut db,
            &mut cache.into_inner(),
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
