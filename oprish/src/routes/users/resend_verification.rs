use rocket::{http::Status, response::status::Custom, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{Emailer, User},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

///Resend the verification email.
///
/// -- STATUS: 204
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -X POST \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/users/resend-verification
/// ```
#[autodoc("/users", category = "Users")]
#[post("/resend-verification")]
pub async fn resend_verification(
    conf: &State<Conf>,
    emailer: &State<Emailer>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Custom<()>> {
    let mut rate_limiter = RateLimiter::new("resend_verification", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(
        User::resend_verification(session.0, emailer, &mut db, &mut cache.into_inner(), conf)
            .await
            .map(|_| Custom(Status::NoContent, ()))
            .map_err(|err| rate_limiter.add_headers(err))?,
    )
}
