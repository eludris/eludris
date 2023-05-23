use serde::{Deserialize, Serialize};

use super::RateLimitConf;

/// Rate limits that apply to Oprish (The REST API).
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "info": {
///     "reset_after": 5,
///     "limit": 2
///   },
///   "message_create": {
///     "reset_after": 5,
///     "limit": 10
///   }
/// }
/// ```
#[autodoc(category = "Instance")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OprishRateLimits {
    /// Rate limits for the [`get_instance_info`] endpoint.
    #[serde(default = "get_instance_info_default")]
    pub get_instance_info: RateLimitConf,
    /// Rate limits for the [`create_message`] endpoint.
    #[serde(default = "create_message_default")]
    pub create_message: RateLimitConf,
}

impl Default for OprishRateLimits {
    fn default() -> Self {
        Self {
            get_instance_info: get_instance_info_default(),
            create_message: create_message_default(),
        }
    }
}

fn get_instance_info_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 2,
    }
}

fn create_message_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 10,
    }
}
