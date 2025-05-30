use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP, TokenAuth, UserIdentifier, DB},
    models::{ErrorResponse, User},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Get a user using a [`UserIdentifier`].
///
/// This does not require authorization when not using @me, but authorized users
/// will get a separate rate limit which is usually (hopefully) higher than the
/// guest rate limit.
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
#[get("/<identifier>")]
pub async fn get_user(
    conf: &State<Conf>,
    identifier: UserIdentifier,
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
    let user = match identifier {
        UserIdentifier::Me => match session {
            Some(session) => User::get_unfiltered(session.0.user_id, &mut db).await,
            None => Err(error!(UNAUTHORIZED)),
        },
        UserIdentifier::ID(id) => {
            User::get(
                id,
                session.map(|s| s.0.user_id),
                &mut db,
                &mut cache.into_inner(),
            )
            .await
        }
        UserIdentifier::Username(username) => {
            User::get_username(
                &username,
                session.map(|s| s.0.user_id),
                &mut db,
                &mut cache.into_inner(),
            )
            .await
        }
    }
    .map_err(|err| rate_limiter.add_headers(err))?;
    rate_limiter.wrap_response(Json(user))
}
