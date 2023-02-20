pub mod messages;

use rocket::{serde::json::Json, Route, State};
use rocket_db_pools::Connection;
use todel::{
    http::ClientIP,
    models::{InstanceInfo, InstanceRateLimits},
    Conf,
};

use crate::{
    rate_limit::{RateLimitedRouteResponse, RateLimiter},
    Cache, VERSION,
}; // poggers

#[get("/?<rate_limits>")]
pub async fn index(
    rate_limits: bool,
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Json<InstanceInfo>> {
    let mut rate_limiter = RateLimiter::new("info", address, conf.inner());
    rate_limiter.process_rate_limit(&mut cache).await?;
    rate_limiter.wrap_response(Json(InstanceInfo {
        instance_name: conf.instance_name.clone(),
        version: VERSION,
        description: conf.description.clone(),
        message_limit: conf.oprish.message_limit,
        oprish_url: &conf.oprish.url,
        pandemonium_url: &conf.pandemonium.url,
        effis_url: &conf.effis.url,
        file_size: conf.effis.file_size,
        attachment_file_size: conf.effis.attachment_file_size,
        rate_limits: rate_limits.then_some(InstanceRateLimits {
            oprish: conf.oprish.rate_limits.clone(),
            pandemonium: conf.pandemonium.rate_limit.clone(),
            effis: conf.effis.rate_limits.clone(),
        }),
    }))
}

pub fn get_routes() -> Vec<Route> {
    routes![index]
}
