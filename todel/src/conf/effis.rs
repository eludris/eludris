use serde::{Deserialize, Deserializer, Serialize};
use ubyte::ByteUnit;

use super::RateLimitConf;

/// Effis configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffisConf {
    pub url: String,
    #[serde(deserialize_with = "deserialize_file_size")]
    #[serde(default = "file_size_default")]
    pub file_size: u64,
    #[serde(deserialize_with = "deserialize_file_size")]
    #[serde(default = "attachment_file_size_default")]
    pub attachment_file_size: u64,
    #[serde(default)]
    pub rate_limits: EffisRateLimits,
}

impl Default for EffisConf {
    fn default() -> Self {
        Self {
            file_size: file_size_default(),
            url: "https://example.com".to_string(),
            attachment_file_size: attachment_file_size_default(),
            rate_limits: EffisRateLimits::default(),
        }
    }
}

fn file_size_default() -> u64 {
    20_000_000 // 20MB
}

fn attachment_file_size_default() -> u64 {
    100_000_000 // 100MB
}

/// Rate limits that apply to Effis (The CDN).
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "assets": {
///     "reset_after": 60,
///     "limit": 5,
///     "file_size_limit": 30000000
///   },
///   "attachments": {
///     "reset_after": 180,
///     "limit": 20,
///     "file_size_limit": 500000000
///   },
///   "fetch_file": {
///     "reset_after": 60,
///     "limit": 30
///   }
/// }
/// ```
#[autodoc(category = "Instance")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffisRateLimits {
    /// Rate limits for the asset buckets.
    #[serde(default = "assets_default")]
    pub assets: EffisRateLimitConf,
    /// Rate limits for the attachment bucket.
    #[serde(default = "attachments_default")]
    pub attachments: EffisRateLimitConf,
    /// Rate limits for the file fetching endpoints.
    #[serde(default = "fetch_file_default")]
    pub fetch_file: RateLimitConf,
}

impl Default for EffisRateLimits {
    fn default() -> Self {
        Self {
            assets: assets_default(),
            attachments: attachments_default(),
            fetch_file: fetch_file_default(),
        }
    }
}

/// Represents a single rate limit for Effis.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "reset_after": 60,
///   "limit": 5,
///   "file_size_limit": 30000000
/// }
/// ```
#[autodoc(category = "Instance")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffisRateLimitConf {
    /// The amount of seconds after which the rate limit resets.
    pub reset_after: u32,
    /// The amount of requests that can be made within the `reset_after` interval.
    pub limit: u32,
    /// The maximum amount of bytes that can be sent within the `reset_after` interval.
    #[serde(deserialize_with = "deserialize_file_size")]
    pub file_size_limit: u64,
}

fn assets_default() -> EffisRateLimitConf {
    EffisRateLimitConf {
        reset_after: 60,
        limit: 5,
        file_size_limit: 30_000_000, // 30MB
    }
}

fn attachments_default() -> EffisRateLimitConf {
    EffisRateLimitConf {
        reset_after: 180,
        limit: 20,
        file_size_limit: 500_000_000, // 500MB
    }
}

fn fetch_file_default() -> RateLimitConf {
    RateLimitConf {
        reset_after: 60,
        limit: 30,
    }
}

pub(crate) fn deserialize_file_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(ByteUnit::deserialize(deserializer)?.as_u64())
}
