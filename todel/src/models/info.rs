use crate::{
    conf::{EffisRateLimits, OprishRateLimits, RateLimitConf},
    Conf,
};
use serde::{Deserialize, Serialize};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The instance info payload
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

/// The type which represents all of an instance's rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceRateLimits {
    pub oprish: OprishRateLimits,
    pub pandemonium: RateLimitConf,
    pub effis: EffisRateLimits,
}

#[cfg(feature = "logic")]
impl InstanceInfo {
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
