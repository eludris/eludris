use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{UpdateUserProfile, User},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Modify your profile.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   -X PATCH
///   --json '{"display_name":"HappyRu","bio":"I am very happy!"}'
///   https://api.eludris.gay/users/profile
///
/// {
///   "id": 2346806935553
///   "username": "yendri"
///   "display_name": "HappyRu"
///   "social_credit": 0,
///   "bio": "I am very happy!"
///   "badges": 0,
///   "permissions": 0
/// }
/// ```
#[autodoc("/users", category = "Users")]
#[patch("/profile", data = "<profile>")]
pub async fn update_profile(
    profile: Json<UpdateUserProfile>,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<User>> {
    let mut rate_limiter = RateLimiter::new("update_profile", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        User::update_profile(session.0.user_id, profile.into_inner(), conf, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
