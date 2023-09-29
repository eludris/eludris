use argon2::Argon2;
use rocket::{http::Status, response::status::Custom, serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{PasswordDeleteCredentials, Session},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Delete a session.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -X DELETE \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/sessions/2342734331909
/// ```
#[autodoc("/sessions", category = "Sessions")]
#[delete("/<session_id>", data = "<delete>")]
pub async fn delete_session(
    session_id: u64,
    delete: Json<PasswordDeleteCredentials>,
    conf: &State<Conf>,
    verifier: &State<Argon2<'static>>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Custom<()>> {
    let mut rate_limiter = RateLimiter::new("delete_session", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(Custom(
        Status::NoContent,
        Session::delete(
            session_id,
            session.0.user_id,
            delete.into_inner(),
            verifier.inner(),
            &mut db,
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
