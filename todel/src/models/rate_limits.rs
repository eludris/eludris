use serde::{Deserialize, Serialize};

use crate::conf::{EffisRateLimits, OprishRateLimits, RateLimitConf};

/// The type which represents all of an instance's rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceRateLimits {
    pub oprish: OprishRateLimits,
    pub pandemonium: RateLimitConf,
    pub effis: EffisRateLimits,
}
