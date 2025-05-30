use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{Sphere, User},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[autodoc("/spheres", category = "Spheres")]
#[get("/")]
pub async fn get_spheres(
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<Vec<Sphere>>> {
    let mut rate_limiter = RateLimiter::new("get_spheres", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    let user = User::get_unfiltered(session.0.user_id, &mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;
    rate_limiter.wrap_response(Json(
        user.get_spheres(&mut db, &mut cache.into_inner())
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
