use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP, TokenAuth, DB},
    models::User,
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Get your own user.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/users/@me
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
#[get("/@me")]
pub async fn get_self(
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<User>> {
    let mut rate_limiter = RateLimiter::new("get_user", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        User::get(
            session.0.user_id,
            Some(session.0.user_id),
            &mut db,
            &mut cache.into_inner(),
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}

/// Get a user by ID.
///
/// This does not require authorization, but authorized users will get a separate rate limit
/// which is usually (hopefully) higher than the guest rate limit.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/users/48615849987333
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
#[get("/<user_id>")]
pub async fn get_user(
    user_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: Option<TokenAuth>,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Json<User>> {
    let mut rate_limiter;
    if let Some(session) = &session {
        rate_limiter = RateLimiter::new("get_user", session.0.user_id, conf);
    } else {
        rate_limiter = RateLimiter::new("guest_get_user", ip, conf);
    }
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        User::get(
            user_id,
            session.map(|s| s.0.user_id),
            &mut db,
            &mut cache.into_inner(),
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}

/// Get a user by their username.
///
/// This does not require authorization, but authorized users will get a separate rate limit
/// which is usually (hopefully) higher than the guest rate limit.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/users/yendri
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
#[get("/<username>", rank = 1)]
pub async fn get_user_with_username(
    username: &str,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: Option<TokenAuth>,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Json<User>> {
    let mut rate_limiter;
    if let Some(session) = &session {
        rate_limiter = RateLimiter::new("get_user", session.0.user_id, conf);
    } else {
        rate_limiter = RateLimiter::new("guest_get_user", ip, conf);
    }
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        User::get_username(
            username,
            session.map(|s| s.0.user_id),
            &mut db,
            &mut cache.into_inner(),
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
