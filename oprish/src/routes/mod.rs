pub mod messages;

use rocket::{serde::json::Json, Route, State};
use rocket_db_pools::Connection;
use todel::{http::ClientIP, models::InstanceInfo, Conf};

use crate::{
    rate_limit::{RateLimitedRouteResponse, RateLimiter},
    Cache,
}; // poggers

/// Get information about the instance you're sending this request to.
///
/// Most of this data comes from the instance's configuration.
///
/// -----
///
/// # Example
///
/// ```sh
/// curl https://api.eludris.gay/
///
/// {
///   "instance_name": "eludris",
///   "description": "The *almost* official Eludris instance - ooliver.\nThis is **not** a testing instance as it is bridged to Eludis. Use your own local instance for testing.",
///   "version": "0.3.2",
///   "message_limit": 2000,
///   "oprish_url": "https://api.eludris.gay",
///   "pandemonium_url": "wss://ws.eludris.gay/",
///   "effis_url": "https://cdn.eludris.gay",
///   "file_size": 20000000,
///   "attachment_file_size": 25000000
/// }
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
