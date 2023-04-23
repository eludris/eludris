pub mod messages;

use rocket::{serde::json::Json, Route, State};
use rocket_db_pools::Connection;
use todel::{http::ClientIP, models::InstanceInfo, Conf};

use crate::{
    rate_limit::{RateLimitedRouteResponse, RateLimiter},
    Cache,
}; // poggers

#[autodoc(category = "Instance")]
#[get("/?<rate_limits>")]
pub async fn get_instance_info(
    rate_limits: bool,
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Json<InstanceInfo>> {
    let mut rate_limiter = RateLimiter::new("info", address, conf.inner());
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(Json(InstanceInfo::from_conf(conf.inner(), rate_limits)))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_instance_info]
}
