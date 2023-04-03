//! Simple abstraction for a TOML based Derailed configuration file
mod effis_rate_limits;
mod oprish_rate_limits;

use serde::{Deserialize, Serialize};

#[cfg(feature = "logic")]
use anyhow::{bail, Context};
#[cfg(feature = "logic")]
use std::str::FromStr;
#[cfg(feature = "logic")]
use std::{env, fs, path};
#[cfg(feature = "logic")]
use url::Url;

pub use effis_rate_limits::*;
pub use oprish_rate_limits::*;

/// Derailed config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conf {
    pub instance_name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub oprish: OprishConf,
    #[serde(default)]
    pub pandemonium: PandemoniumConf,
    #[serde(default)]
    pub effis: EffisConf,
}

/// Oprish config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OprishConf {
    #[serde(default = "message_limit_default")]
    pub message_limit: usize,
    pub url: String,
    #[serde(default)]
    pub rate_limits: OprishRateLimits,
}

impl Default for OprishConf {
    fn default() -> Self {
        Self {
            url: "https://example.com".to_string(),
            message_limit: message_limit_default(),
            rate_limits: OprishRateLimits::default(),
        }
    }
}

fn message_limit_default() -> usize {
    2048
}

/// Pandemonium config.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Effis config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffisConf {
    #[serde(deserialize_with = "deserialize_file_size")]
    #[serde(default = "file_size_default")]
    pub file_size: u64,
    #[serde(deserialize_with = "deserialize_file_size")]
    #[serde(default = "attachment_file_size_default")]
    pub attachment_file_size: u64,
    pub url: String,
    #[serde(default)]
    pub rate_limits: EffisRateLimits,
}

fn file_size_default() -> u64 {
    20_000_000 // 20MB
}

fn attachment_file_size_default() -> u64 {
    100_000_000 // 100MB
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

/// RateLimit config data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConf {
    pub reset_after: u32,
    pub limit: u32,
}

#[cfg(feature = "logic")]
macro_rules! validate_rate_limit_limits {
    ($rate_limits:expr, $($bucket_name:ident),+) => {
        if $(
            $rate_limits.$bucket_name.limit == 0
            )||+ {
            bail!("RateLimit limit can't be 0");
        }
    };
}

#[cfg(feature = "logic")]
macro_rules! validate_file_sizes {
    ($($size:expr),+) => {
        if $(
            $size == 0
            )||+ {
            bail!("File size can't be 0");
        }
    };
}

#[cfg(feature = "logic")]
impl Conf {
    /// Create a new [`Conf`].
    ///
    /// # Panics
    ///
    /// This function is *intended* to panic if a suitable config is not found.
    ///
    /// That also includes the config file's data failing to deserialise.
    pub fn new<T: AsRef<path::Path>>(path: T) -> anyhow::Result<Self> {
        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file {}", path.as_ref().display()))?;
        let data: Self = toml::from_str(&data).with_context(|| {
            format!("Could not parse {} as valid toml", path.as_ref().display())
        })?;
        data.validate()?;
        Ok(data)
    }

    /// Create a new [`Conf`] by determining it's path based on the "DERAILED_CONF" environment
    /// variable or falling back to "Derailed.toml" if it is not found.
    ///
    /// # Panics
    ///
    /// This function is *intended* to panic if a suitable config is not found.
    ///
    /// That also includes the config file's data failing to deserialise.
    pub fn new_from_env() -> anyhow::Result<Self> {
        Self::new(env::var("DERAILED_CONF").unwrap_or_else(|_| "Derailed.toml".to_string()))
    }

    #[cfg(test)]
    /// Create a new [`Conf`] with default config from the provided instance name.
    fn from_name(instance_name: String) -> anyhow::Result<Self> {
        let conf = Self {
            instance_name,
            description: None,
            oprish: OprishConf::default(),
            pandemonium: PandemoniumConf::default(),
            effis: EffisConf::default(),
        };
        conf.validate()?;
        Ok(conf)
    }

    /// Validates a [`Conf`]
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.instance_name.is_empty() || self.instance_name.len() > 32 {
            bail!("Invalid instance_name length, must be between 1 and 32 characters long");
        }
        if let Some(description) = &self.description {
            if description.is_empty() || description.len() > 2048 {
                bail!("Invalid description length, must be between 1 and 2048 characters long");
            }
        }
        if self.oprish.message_limit < 1024 {
            bail!("Message limit can not be less than 1024 characters");
        }
        validate_rate_limit_limits!(self.oprish.rate_limits, info, message_create, rate_limits);
        validate_rate_limit_limits!(self.pandemonium, rate_limit);
        validate_rate_limit_limits!(self.effis.rate_limits, assets, attachments, fetch_file);

        Url::parse(&self.oprish.url)
            .with_context(|| format!("Invalid oprish url {}", self.oprish.url))?;
        Url::parse(&self.pandemonium.url)
            .with_context(|| format!("Invalid pandemonium url {}", self.pandemonium.url))?;
        Url::parse(&self.effis.url)
            .with_context(|| format!("Invalid effis url {}", self.effis.url))?;

        validate_file_sizes!(
            self.effis.file_size,
            self.effis.attachment_file_size,
            self.effis.rate_limits.assets.file_size_limit,
            self.effis.rate_limits.attachments.file_size_limit
        );

        Ok(())
    }
}

#[cfg(feature = "logic")]
impl FromStr for Conf {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Self = toml::from_str(s).context("Could not parse provided toml as a Conf")?;
        data.validate()?;
        Ok(data)
    }
}

#[cfg(feature = "logic")]
#[cfg(test)]
mod tests {
    use crate::conf::*;

    #[test]
    fn try_deserialize() {
        // This is yucky since there is leading space but TOML thankfully doesn't mind it
        let conf_str = r#"
            instance_name = "WooChat"
            description = "The poggest place to chat"

            [oprish]
            url = "https://example.com"

            [oprish.rate_limits]
            info = { reset_after = 10, limit = 2}

            [pandemonium]
            url = "wss://foo.bar"
            rate_limit = { reset_after = 20, limit = 10}

            [effis]
            file_size = "100MB"
            url = "https://example.com"

            [effis.rate_limits]
            attachments = { reset_after = 600, limit = 20, file_size_limit = "500MB"}
            "#;

        let conf_str: Conf = toml::from_str(conf_str).unwrap();

        let conf = Conf {
            instance_name: "WooChat".to_string(),
            description: Some("The poggest place to chat".to_string()),
            oprish: OprishConf {
                rate_limits: OprishRateLimits {
                    info: RateLimitConf {
                        reset_after: 10,
                        limit: 2,
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            pandemonium: PandemoniumConf {
                rate_limit: RateLimitConf {
                    reset_after: 20,
                    limit: 10,
                },
                url: "wss://foo.bar".to_string(),
            },
            effis: EffisConf {
                file_size: 100_000_000,
                rate_limits: EffisRateLimits {
                    attachments: EffisRateLimitConf {
                        reset_after: 600,
                        limit: 20,
                        file_size_limit: 500_000_000,
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        };

        assert_eq!(format!("{:?}", conf_str), format!("{:?}", conf));
    }

    #[test]
    fn default_conf() {
        let conf_str = "instance_name = \"TestInstance\"";

        let conf_str: Conf = toml::from_str(conf_str).unwrap();

        let conf = Conf::from_name("TestInstance".to_string()).unwrap();

        assert_eq!(format!("{:?}", conf_str), format!("{:?}", conf));
    }

    macro_rules! test_limit {
        ($conf:expr, $($limit:expr),+) => {
            $(
                $limit.limit = 0;
                assert!($conf.validate().is_err());
                $limit.limit = 1;
                assert!($conf.validate().is_ok());
            )+
        };
    }

    macro_rules! test_urls {
        ($conf:expr, $($service:ident),+) => {
            $(
                $conf.$service.url = "notavalidurl".to_string();
                assert!($conf.validate().is_err());
                $conf.$service.url = "http://avalid.url".to_string();
                assert!($conf.validate().is_ok());
            )+
        };
    }

    macro_rules! test_file_sizes {
        ($conf:expr, $($size:expr),+) => {
            $(
                $size = 0;
                assert!($conf.validate().is_err());
                $size = 1;
                assert!($conf.validate().is_ok());
            )+
        };
    }

    #[test]
    fn validate() {
        let mut conf = Conf::from_name("WooChat".to_string()).unwrap();

        assert!(conf.validate().is_ok());
        conf.instance_name = "".to_string();
        assert!(conf.validate().is_err());
        conf.instance_name = "h".repeat(33);
        assert!(conf.validate().is_err());
        conf.instance_name = "woo".to_string();
        assert!(conf.validate().is_ok());

        conf.description = Some("".to_string());
        assert!(conf.validate().is_err());
        conf.description = Some("h".repeat(2049));
        assert!(conf.validate().is_err());
        conf.description = Some("very cool".to_string());
        assert!(conf.validate().is_ok());

        conf.oprish.message_limit = 2;
        assert!(conf.validate().is_err());
        conf.oprish.message_limit = 1024;
        assert!(conf.validate().is_ok());

        test_limit!(
            conf,
            conf.pandemonium.rate_limit,
            conf.effis.rate_limits.assets,
            conf.effis.rate_limits.attachments,
            conf.effis.rate_limits.fetch_file,
            conf.oprish.rate_limits.info,
            conf.oprish.rate_limits.message_create,
            conf.oprish.rate_limits.rate_limits
        );

        test_urls!(conf, oprish, pandemonium, effis);

        test_file_sizes!(
            conf,
            conf.effis.file_size,
            conf.effis.attachment_file_size,
            conf.effis.rate_limits.assets.file_size_limit,
            conf.effis.rate_limits.attachments.file_size_limit
        );
    }
}
