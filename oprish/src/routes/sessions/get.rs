use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::Session,
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Get all sessions.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/sessions
///
/// [
///   {
///     "id": 2472278163458,
///     "user_id": 48615849987333,
///     "platform": "linux",
///     "client": "pilfer",
///     "ip": "fc00:e10d:7150:b1gb:00b5:f00d:babe:1337"
///   },
///   {
///     "id": 2472278163867,
///     "user_id": 48615849987333,
///     "platform": "python",
///     "client": "velum",
///     "ip": "127.0.0.1"
///   }
/// ]
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
