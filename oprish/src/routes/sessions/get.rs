use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::Session,
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Create a new session.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   --json '{
///   "identifier": "yendri",
///   "password": "authentícame por favor",
///   "platform": "linux",
///   "client":"pilfer"
/// }' \
///   https://api.eludris.gay/sessions
///
/// {
///   "token": "<token>",
///   "session": {
///     "indentifier": "yendri",
///     "password": "authentícame por favor",
///     "platform": "linux",
///     "client": "pilfer"
///   }
/// }
/// ```
#[autodoc("/sessions", category = "Sessions")]
#[get("/")]
pub async fn get_sessions(
    conf: &State<Conf>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<Vec<Session>>> {
    let mut rate_limiter = RateLimiter::new("get_sessions", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(Json(
        Session::get_sessions(session.0.user_id, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
