use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP, TokenAuth, DB},
    models::User,
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

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
        User::get(session.0.user_id, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}

#[get("/<id>")]
pub async fn get_user(
    id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: Option<TokenAuth>,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Json<User>> {
    let mut rate_limiter;
    if let Some(session) = session {
        rate_limiter = RateLimiter::new("get_user", session.0.user_id, conf);
    } else {
        rate_limiter = RateLimiter::new("guest_get_user", ip, conf);
    }
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        User::get(id, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}

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
    if let Some(session) = session {
        rate_limiter = RateLimiter::new("get_user", session.0.user_id, conf);
    } else {
        rate_limiter = RateLimiter::new("guest_get_user", ip, conf);
    }
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(
        User::get_username(username, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
