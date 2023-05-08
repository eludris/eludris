use crate::conf::{EffisRateLimits, OprishRateLimits, RateLimitConf};
use serde::{Deserialize, Serialize};

#[cfg(feature = "logic")]
use crate::Conf;
#[cfg(feature = "logic")]
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Represents information about the connected Eludris instance.
///
/// -----
///
/// ### Example
///
/// ```json
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
/// ```
#[autodoc(category = "Instance")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    /// The name of the instance.
    pub instance_name: String,
    /// The description of the instance.
    ///
    /// This is between 1 and 2048 characters long.
    pub description: Option<String>,
    /// The Eludris version the instance is running.
    pub version: String,
    /// The maximum length of a message's content.
    pub message_limit: usize,
    /// The URL of the instance's Oprish (REST API) endpoint.
    pub oprish_url: String,
    /// The URL of the instance's Pandemonium (WebSocket API) endpoint.
    pub pandemonium_url: String,
    /// The URL of the instance's Effis (CDN) endpoint.
    pub effis_url: String,
    /// The maximum file size (in bytes) of an asset.
    pub file_size: u64,
    /// The maximum file size (in bytes) of an attachment.
    pub attachment_file_size: u64,
    /// The rate limits that apply to the connected Eludris instance.
    ///
    /// This is not present if the `rate_limits` query parameter is not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<InstanceRateLimits>,
}

/// Represents all rate limits that apply to the connected Eludris instance.
#[autodoc(category = "Instance")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceRateLimits {
    /// Rate limits for Oprish (The REST API).
    pub oprish: OprishRateLimits,
    /// Rate limits for Pandemonium (The WebSocket API).
    ///
    /// This is the rate limit for sending messages to the gateway,
    /// and for connecting.
    pub pandemonium: RateLimitConf,
    /// Rate limits for Effis (The CDN).
    pub effis: EffisRateLimits,
}

#[cfg(feature = "logic")]
impl InstanceInfo {
    /// Creates a [`InstanceInfo`] from a [`Conf`]
    pub fn from_conf(conf: &Conf, rate_limits: bool) -> Self {
        InstanceInfo {
            instance_name: conf.instance_name.clone(),
            version: VERSION.to_string(),
            description: conf.description.clone(),
            message_limit: conf.oprish.message_limit,
            oprish_url: conf.oprish.url.clone(),
            pandemonium_url: conf.pandemonium.url.clone(),
            effis_url: conf.effis.url.clone(),
            file_size: conf.effis.file_size,
            attachment_file_size: conf.effis.attachment_file_size,
            rate_limits: rate_limits.then_some(InstanceRateLimits {
                oprish: conf.oprish.rate_limits.clone(),
                pandemonium: conf.pandemonium.rate_limit.clone(),
                effis: conf.effis.rate_limits.clone(),
            }),
        }
    }
}
