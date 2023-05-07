use crate::conf::{EffisRateLimits, OprishRateLimits, RateLimitConf};
use serde::{Deserialize, Serialize};

#[cfg(feature = "logic")]
use crate::Conf;
#[cfg(feature = "logic")]
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Represents information about the connected Eludris instance.
#[autodoc(category = "Instance")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub instance_name: String,
    pub description: Option<String>,
    pub version: String,
    pub message_limit: usize,
    pub oprish_url: String,
    pub pandemonium_url: String,
    pub effis_url: String,
    pub file_size: u64,
    pub attachment_file_size: u64,
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
