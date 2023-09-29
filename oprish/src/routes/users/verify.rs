use rocket::{http::Status, response::status::Custom, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::User,
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Verify your email address.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -X POST \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/users/verify?code=123456
/// ```
#[autodoc("/users", category = "Users")]
#[post("/verify?<code>")]
pub async fn verify_user(
    code: u32,
    conf: &State<Conf>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Custom<()>> {
    let mut rate_limiter = RateLimiter::new("verify_user", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(
        User::verify(code, session.0, &mut db, &mut cache.into_inner())
            .await
            .map(|_| Custom(Status::NoContent, ()))
            .map_err(|err| rate_limiter.add_headers(err))?,
    )
}
