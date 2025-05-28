pub mod channels;
pub mod sessions;
pub mod spheres;
pub mod users;

use rocket::{serde::json::Json, Route, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP},
    models::InstanceInfo,
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter}; // poggers

/// Get information about the instance you're sending this request to.
///
/// Most of this data comes from the instance's configuration.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl https://api.eludris.gay/?rate_limits
///
/// {
///   "instance_name": "eludris",
///   "description": "The *almost* official Eludris instance - ooliver.",
///   "version": "0.3.2",
///   "message_limit": 2000,
///   "oprish_url": "https://api.eludris.gay",
///   "pandemonium_url": "wss://ws.eludris.gay/",
///   "effis_url": "https://cdn.eludris.gay",
///   "file_size": 20000000,
///   "attachment_file_size": 25000000,
///   "rate_limits": {
///     "oprish": {
///       "info": {
///         "reset_after": 5,
///         "limit": 2
///       },
///       "message_create": {
///         "reset_after": 5,
///         "limit": 10
///       }
///     },
///     "pandemonium": {
///       "reset_after": 10,
///       "limit": 5
///     },
///     "effis": {
///       "assets": {
///         "reset_after": 60,
///         "limit": 5,
///         "file_size_limit": 30000000
///       },
///       "attachments": {
///         "reset_after": 180,
///         "limit": 20,
///         "file_size_limit": 500000000
///       },
///       "fetch_file": {
///         "reset_after": 60,
///         "limit": 30
///       }
///     }
///   }
/// }
/// ```
#[autodoc(category = "Instance")]
#[get("/?<rate_limits>")]
pub async fn get_instance_info(
    rate_limits: bool,
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Json<InstanceInfo>> {
    let mut rate_limiter = RateLimiter::new("get_instance_info", address, conf.inner());
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(Json(InstanceInfo::from_conf(conf.inner(), rate_limits)))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_instance_info]
}
