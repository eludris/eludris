use serde::{Deserialize, Serialize};

use super::RateLimitConf;

/// Pandemonium configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PandemoniumConf {
    pub url: String,
    #[serde(default = "pandemonium_rate_limit_default")]
    pub rate_limit: RateLimitConf,
}

impl Default for PandemoniumConf {
    fn default() -> Self {
        Self {
            url: "https://example.com".to_string(),
            rate_limit: pandemonium_rate_limit_default(),
        }
    }
}

fn pandemonium_rate_limit_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 10,
        limit: 5,
    }
}
