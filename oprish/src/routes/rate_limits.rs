use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{http::ClientIP, models::InstanceRateLimits, Conf};

use crate::{
    rate_limit::{RateLimitedRouteResponse, RateLimiter},
    Cache,
};

#[get("/rate_limits")]
pub async fn rate_limits(
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Json<InstanceRateLimits>> {
    let conf = conf.inner();
    let mut rate_limiter = RateLimiter::new("rate_limits", address, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(InstanceRateLimits {
        oprish: conf.oprish.rate_limits.clone(),
        pandemonium: conf.pandemonium.rate_limit.clone(),
        effis: conf.effis.rate_limits.clone(),
    }))
}
