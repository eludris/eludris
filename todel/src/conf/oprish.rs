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

macro_rules! oprish_ratelimits {
    ($($bucket:ident => ($bucket_str:literal, $reset_after:literal, $limit:literal)),+$(,)?) => {
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
            $(
                #[doc = "Rate Limits for the [`"]
                #[doc = $bucket_str]
                #[doc = "`] route."]
                #[serde(default = $bucket_str)]
                pub $bucket: RateLimitConf,
            )+
        }

        $(
            pub fn $bucket() -> RateLimitConf {
                RateLimitConf {
                    reset_after: $reset_after,
                    limit: $limit,
                }
            }
        )+

        impl Default for OprishRateLimits {
            fn default() -> Self {
                Self {
                    $(
                        $bucket: $bucket(),
                    )+
                }
            }
        }
    };
}

oprish_ratelimits!(
    get_instance_info => ("get_instance_info", 5, 10),
    create_message => ("create_message", 5, 10),
    create_user => ("create_user", 60, 3),
    verify_user => ("verify_user", 30, 5),
    get_user => ("get_user", 5, 10),
    guest_get_user => ("guest_get_user", 20, 10),
    edit_user => ("edit_user", 60, 5),
    edit_profile => ("edit_profile", 60, 5),
    delete_user => ("delete_user", 30, 1),
    create_password_reset_code => ("create_password_reset_code", 60, 3),
    reset_password => ("reset_password", 60, 3),
    create_session => ("create_session", 60, 3),
    get_sessions => ("get_sessions", 5, 10),
    delete_session => ("delete_session", 5, 10),
    resend_verification => ("resend_verification", 60, 3),
    create_sphere => ("create_sphere", 20, 3),
    get_sphere => ("get_sphere", 5, 10),
    guest_get_sphere => ("guest_get_sphere", 20, 10),
    edit_category => ("edit_category", 10, 5),
    delete_category => ("delete_category", 10, 5),
    create_category => ("create_category", 10, 5),
    create_channel => ("create_channel", 10, 5),
    edit_channel => ("edit_channel", 10, 5),
    delete_channel => ("delete_channel", 10, 5),
    join_sphere => ("join_sphere", 5, 20),
    get_channel => ("get_channel", 5, 20),
    guest_get_channel => ("guest_get_channel", 20, 10),
    get_messages => ("get_messages", 5, 10),
    get_message => ("get_message", 5, 10),
    get_member => ("get_member", 5, 10),
    guest_get_member => ("guest_get_member", 20, 10),
    edit_member => ("edit_member", 5, 10),
    delete_message => ("delete_message", 5, 10),
    edit_message => ("edit_message", 5, 10),
    leave_sphere => ("leave_sphere", 5, 10),
    get_spheres => ("get_spheres", 5, 10),
    create_emoji => ("create_emoji", 5, 10),
    get_emoji => ("get_emoji", 5, 10),
    guest_get_emoji => ("guest_get_emoji", 20, 10),
    edit_emoji => ("edit_emoji", 5, 10),
    delete_emoji => ("edit_emoji", 5, 10),
    add_reaction => ("add_reaction", 5, 10),
    remove_reaction => ("remove_reaction", 5, 10),
    clear_reactions => ("clear_reactions", 5, 10),
);
