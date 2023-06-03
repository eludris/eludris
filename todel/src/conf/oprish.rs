use serde::{Deserialize, Serialize};

use super::RateLimitConf;

/// Oprish configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OprishConf {
    pub url: String,
    #[serde(default = "message_limit_default")]
    pub message_limit: usize,
    #[serde(default = "bio_limit_default")]
    pub bio_limit: usize,
    #[serde(default)]
    pub rate_limits: OprishRateLimits,
}

impl Default for OprishConf {
    fn default() -> Self {
        Self {
            url: "https://example.com".to_string(),
            message_limit: message_limit_default(),
            bio_limit: bio_limit_default(),
            rate_limits: OprishRateLimits::default(),
        }
    }
}

fn message_limit_default() -> usize {
    2048
}

fn bio_limit_default() -> usize {
    250
}

/// Rate limits that apply to Oprish (The REST API).
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "get_instance_info": {
///     "reset_after": 5,
///     "limit": 2
///   },
///   "create_message": {
///     "reset_after": 5,
///     "limit": 10
///   },
///   "create_user": {
///   },
/// }
/// ```
#[autodoc(category = "Instance")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OprishRateLimits {
    /// Rate limits for the [`get_instance_info`] endpoint.
    #[serde(default = "get_instance_info_default")]
    pub get_instance_info: RateLimitConf,
    /// Rate limits for the [`create_message`] endpoint.
    #[serde(default = "create_message_default")]
    pub create_message: RateLimitConf,
    /// Rate limits for the [`create_user`] endpoint.
    #[serde(default = "create_user_default")]
    pub create_user: RateLimitConf,
    /// Rate limits for the [`verify_user`] endpoint.
    #[serde(default = "verify_user_default")]
    pub verify_user: RateLimitConf,
    /// Rate limits for the [`get_self`], [`get_user`] and [`get_user_from_username`] endpoints.
    #[serde(default = "get_user_default")]
    pub get_user: RateLimitConf,
    /// Rate limits for the [`get_self`], [`get_user`] and [`get_user_from_username`] endpoints for
    /// someone who hasn't made an account.
    #[serde(default = "guest_get_user_default")]
    pub guest_get_user: RateLimitConf,
    /// Rate limits for the [`update_profile`] enpoint.
    #[serde(default = "update_profile_default")]
    pub update_profile: RateLimitConf,
    /// Rate limits for the [`create_session`] endpoint.
    #[serde(default = "create_session_default")]
    pub create_session: RateLimitConf,
}

impl Default for OprishRateLimits {
    fn default() -> Self {
        Self {
            get_instance_info: get_instance_info_default(),
            create_message: create_message_default(),
            create_user: create_message_default(),
            verify_user: verify_user_default(),
            get_user: get_user_default(),
            guest_get_user: guest_get_user_default(),
            update_profile: update_profile_default(),
            create_session: create_session_default(),
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

fn create_user_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 3600,
        limit: 1,
    }
}

fn verify_user_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 600,
        limit: 10,
    }
}

fn get_user_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 10,
    }
}

fn guest_get_user_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 5,
    }
}

fn update_profile_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 3600,
        limit: 5,
    }
}

fn create_session_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 1800,
        limit: 5,
    }
}
