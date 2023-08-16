use argon2::Argon2;
use rocket::{http::Status, response::status::Custom, serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP, DB},
    ids::IdGenerator,
    models::{Secret, Session, SessionCreate, SessionCreated},
    Conf,
};
use tokio::sync::Mutex;

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
///   "client": "pilfer"
/// }' \
///   https://api.eludris.gay/sessions
///
/// {
///   "token": "<token>",
///   "session": {
///     "id": 2472278163458,
///     "user_id": 48615849987333,
///     "platform": "linux",
///     "client": "pilfer",
///     "ip": "fc00:e10d:7150:b1gb:00b5:f00d:babe:1337"
///   }
/// }
/// ```
#[autodoc("/sessions", category = "Sessions")]
#[post("/", data = "<session>")]
pub async fn create_session(
    session: Json<SessionCreate>,
    verifier: &State<Argon2<'static>>,
    id_generator: &State<Mutex<IdGenerator>>,
    secret: &State<Secret>,
    conf: &State<Conf>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Custom<Json<SessionCreated>>> {
    let mut rate_limiter = RateLimiter::new("create_session", &ip, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(Custom(
        Status::Created,
        Json(
            Session::create(
                session.into_inner(),
                *ip,
                secret,
                verifier.inner(),
                &mut *id_generator.lock().await,
                &mut db,
            )
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
        ),
    ))
}
