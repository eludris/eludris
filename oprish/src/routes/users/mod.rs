mod get;
mod profile;
mod update;
mod verify;

use argon2::Argon2;
use rand::rngs::StdRng;
use rocket::{http::Status, response::status::Custom, serde::json::Json, Route, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP, DB},
    ids::IdGenerator,
    models::{Emailer, User, UserCreate},
    Conf,
};
use tokio::sync::Mutex;

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Create a new user.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   --json '{
///   "username": "yendri",
///   "email": "yendri@llamoyendri.io",
///   "password": "authentícame por favor"
/// }' \
///   https://api.eludris.gay/users
///
/// {
///   "id": 48615849987333,
///   "username": "yendri",
///   "social_credit": 0,
///   "badges": 0,
///   "permissions": 0
/// }
/// ```
#[autodoc("/users", category = "Users")]
#[post("/", data = "<user>")]
pub async fn create_user(
    user: Json<UserCreate>,
    hasher: &State<Argon2<'static>>,
    rng: &State<Mutex<StdRng>>,
    id_generator: &State<Mutex<IdGenerator>>,
    conf: &State<Conf>,
    mailer: &State<Emailer>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Custom<Json<User>>> {
    let mut rate_limiter = RateLimiter::new("create_user", ip, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(Custom(
        Status::Created,
        Json(
            User::create(
                user.into_inner(),
                hasher.inner(),
                &mut *rng.lock().await,
                &mut *id_generator.lock().await,
                conf,
                mailer,
                &mut db,
                &mut cache.into_inner(),
            )
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
        ),
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        create_user,
        verify::verify_user,
        get::get_self,
        get::get_user,
        get::get_user_with_username,
        update::update_user,
        profile::update_profile,
    ]
}
