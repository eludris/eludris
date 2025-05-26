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
///   ...
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
    /// Rate limits for the [`edit_user`] enpoint.
    #[serde(default = "edit_user_default")]
    pub edit_user: RateLimitConf,
    /// Rate limits for the [`edit_profile`] enpoint.
    #[serde(default = "edit_profile_default")]
    pub edit_profile: RateLimitConf,
    /// Rate limits for the [`delete_user`] enpoint.
    #[serde(default = "delete_user_default")]
    pub delete_user: RateLimitConf,
    /// Rate limits for the [`create_password_reset_code`] enpoint.
    #[serde(default = "create_password_reset_code_default")]
    pub create_password_reset_code: RateLimitConf,
    /// Rate limits for the [`reset_password`] enpoint.
    #[serde(default = "reset_password_default")]
    pub reset_password: RateLimitConf,
    /// Rate limits for the [`create_session`] endpoint.
    #[serde(default = "create_session_default")]
    pub create_session: RateLimitConf,
    /// Rate limits for the [`get_sessions`] endpoint.
    #[serde(default = "get_sessions_default")]
    pub get_sessions: RateLimitConf,
    /// Rate limits for the [`delete_session`] endpoint.
    #[serde(default = "delete_session_default")]
    pub delete_session: RateLimitConf,
    /// Rate limits for the [`resend_verification`] endpoint.
    #[serde(default = "resend_verification_default")]
    pub resend_verification: RateLimitConf,
    /// Rate limits for the [`create_sphere`] endpoint.
    #[serde(default = "create_sphere_default")]
    pub create_sphere: RateLimitConf,
    /// Rate limits for the [`get_sphere`] and [`get_sphere_from_slug`] endpoints.
    #[serde(default = "get_sphere_default")]
    pub get_sphere: RateLimitConf,
    /// Rate limits for the [`get_sphere`] and [`get_sphere_from_slug`] endpoints for
    /// someone who hasn't made an account.
    #[serde(default = "guest_get_sphere_default")]
    pub guest_get_sphere: RateLimitConf,
    /// Rate limits for the [`create_category`] endpoint.
    #[serde(default = "create_category_default")]
    pub create_category: RateLimitConf,
    /// Rate limits for the [`edit_category`] endpoint.
    #[serde(default = "edit_category_default")]
    pub edit_category: RateLimitConf,
    /// Rate limits for the [`delete_category`] endpoint.    
    #[serde(default = "delete_category_default")]
    pub delete_category: RateLimitConf,
    /// Rate limits for the [`create_channel`] endpoint.    
    #[serde(default = "create_channel_default")]
    pub create_channel: RateLimitConf,
    /// Rate limits for the [`edit_channel`] endpoint.
    #[serde(default = "edit_channel_default")]
    pub edit_channel: RateLimitConf,
    /// Rate limits for the [`delete_channel`] endpoint.
    #[serde(default = "delete_channel_default")]
    pub delete_channel: RateLimitConf,
    /// Rate limits for the [`join_sphere`] and [`join_sphere_from_slug`] endpoints.
    #[serde(default = "join_sphere_default")]
    pub join_sphere: RateLimitConf,
    /// Rate limits for the [`get_channel`] endpoint.
    #[serde(default = "get_channel_default")]
    pub get_channel: RateLimitConf,
    /// Rate limits for the [`get_channel`] endpoint for someone who hasn't made
    /// an account.
    #[serde(default = "guest_get_channel_default")]
    pub guest_get_channel: RateLimitConf,
    /// Rate limits for the [`get_messages`] endpoint.
    #[serde(default = "get_messages_default")]
    pub get_messages: RateLimitConf,
    /// Rate limits for the [`get_member`] endpoint.
    #[serde(default = "get_member_default")]
    pub get_member: RateLimitConf,
    /// Rate limits for the [`get_member`] endpoint for someone who hasn't made
    /// an account.
    #[serde(default = "guest_get_member_default")]
    pub guest_get_member: RateLimitConf,
    /// Rate limits for the [`edit_member`] endpoint.
    #[serde(default = "edit_member_default")]
    pub edit_member: RateLimitConf,
}

impl Default for OprishRateLimits {
    fn default() -> Self {
        Self {
            get_instance_info: get_instance_info_default(),
            create_message: create_message_default(),
            create_user: create_user_default(),
            verify_user: verify_user_default(),
            get_user: get_user_default(),
            guest_get_user: guest_get_user_default(),
            edit_user: edit_user_default(),
            edit_profile: edit_profile_default(),
            delete_user: delete_user_default(),
            create_password_reset_code: create_password_reset_code_default(),
            reset_password: reset_password_default(),
            create_session: create_session_default(),
            get_sessions: get_sessions_default(),
            delete_session: delete_session_default(),
            resend_verification: resend_verification_default(),
            create_sphere: create_sphere_default(),
            get_sphere: get_sphere_default(),
            guest_get_sphere: guest_get_sphere_default(),
            create_category: create_category_default(),
            edit_category: edit_category_default(),
            delete_category: delete_category_default(),
            create_channel: create_channel_default(),
            edit_channel: edit_channel_default(),
            delete_channel: delete_channel_default(),
            join_sphere: join_sphere_default(),
            get_channel: get_channel_default(),
            guest_get_channel: guest_get_channel_default(),
            get_messages: get_messages_default(),
            get_member: get_member_default(),
            guest_get_member: guest_get_member_default(),
            edit_member: edit_member_default(),
        }
    }
}

fn get_instance_info_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 10,
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
        reset_after: 180,
        limit: 1,
    }
}

fn verify_user_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 180,
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

fn edit_user_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 180,
        limit: 5,
    }
}

fn edit_profile_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 180,
        limit: 5,
    }
}

fn delete_user_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 300,
        limit: 1,
    }
}

fn create_password_reset_code_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 180,
        limit: 2,
    }
}

fn reset_password_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 60,
        limit: 5,
    }
}

fn create_session_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 600,
        limit: 5,
    }
}

fn get_sessions_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 180,
        limit: 5,
    }
}

fn delete_session_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 60,
        limit: 10,
    }
}

fn resend_verification_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 60,
        limit: 5,
    }
}

fn create_sphere_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 60,
        limit: 5,
    }
}

fn get_sphere_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 20,
    }
}

fn guest_get_sphere_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 10,
    }
}

fn edit_category_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 10,
        limit: 5,
    }
}

fn delete_category_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 10,
        limit: 5,
    }
}

fn create_category_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 10,
        limit: 5,
    }
}

fn create_channel_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 10,
        limit: 5,
    }
}

fn edit_channel_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 10,
        limit: 5,
    }
}

fn delete_channel_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 10,
        limit: 5,
    }
}

fn join_sphere_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 20,
    }
}

fn get_channel_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 20,
    }
}

fn guest_get_channel_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 10,
    }
}

fn get_messages_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 10,
    }
}

fn get_member_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 10,
    }
}
fn guest_get_member_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 10,
    }
}
fn edit_member_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 5,
        limit: 10,
    }
}
