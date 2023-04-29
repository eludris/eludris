use std::{
    fmt::Display,
    time::{Duration, SystemTime},
};

use crate::Cache;
use deadpool_redis::redis::AsyncCommands;
use rocket::http::Header;
use rocket_db_pools::Connection;
use todel::{models::ErrorResponse, Conf};

pub type RateLimitedRouteResponse<T> =
    Result<RateLimitHeaderWrapper<T>, RateLimitHeaderWrapper<ErrorResponse>>;

/// The necessary headers for responses
#[derive(Debug, Responder)]
#[response(content_type = "json")]
pub struct RateLimitHeaderWrapper<T> {
    pub inner: T,
    pub rate_limit_reset: Header<'static>,
    pub rate_limit_max: Header<'static>,
    pub rate_limit_last_reset: Header<'static>,
    pub rate_limit_request_count: Header<'static>,
}

// Can derive debug :chad:
/// A simple RateLimiter than can keep track of rate limit data from KeyDB and add rate limit
/// related headers to a response type
#[derive(Debug)]
pub struct RateLimiter {
    key: String,
    reset_after: Duration,
    request_limit: u32,
    request_count: u32,
    last_reset: u64,
}

impl RateLimiter {
    /// Creates a new RateLimiter
    pub fn new<I>(bucket: &str, identifier: I, conf: &Conf) -> RateLimiter
    where
        I: Display,
    {
        let rate_limit = match bucket {
            "info" => &conf.oprish.rate_limits.info,
            "message_create" => &conf.oprish.rate_limits.message_create,
            "rate_limits" => &conf.oprish.rate_limits.rate_limits,
            _ => unreachable!(),
        };
        RateLimiter {
            key: format!("rate_limit:{}:{}", identifier, bucket),
            reset_after: Duration::from_secs(rate_limit.reset_after as u64),
            request_limit: rate_limit.limit,
            request_count: 0,
            last_reset: 0,
        }
    }

    /// Checks if a bucket is rate limited, if so returns an Error with an ErrorResponse
    pub async fn process_rate_limit(
        &mut self,
        cache: &mut Connection<Cache>,
    ) -> Result<(), RateLimitHeaderWrapper<ErrorResponse>> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;

        if let (Some(last_reset), Some(request_count)) = cache
            .hget::<&str, (&str, &str), (Option<u64>, Option<u32>)>(
                &self.key,
                ("last_reset", "request_count"),
            )
            .await
            .expect("Couldn't query cache")
        {
            self.last_reset = last_reset;
            self.request_count = request_count;
            if now - self.last_reset >= self.reset_after.as_millis() as u64 {
                cache
                    .del::<&str, ()>(&self.key)
                    .await
                    .expect("Couldn't query cache");
                cache
                    .hset_multiple::<&str, &str, u64, ()>(
                        &self.key,
                        &[("last_reset", now), ("request_count", 0)],
                    )
                    .await
                    .expect("Couldn't query cache");
                self.last_reset = now;
                self.request_count = 0;
                log::debug!("Reset bucket for {}", self.key);
            }
            if self.request_count >= self.request_limit {
                log::info!("Rate limited bucket {}", self.key);
                return Err(self
                    .wrap_response::<ErrorResponse, ()>(error!(
                        RATE_LIMITED,
                        self.last_reset + self.reset_after.as_millis() as u64 - now
                    ))
                    .unwrap());
            }
            cache
                .hincr::<&str, &str, u8, ()>(&self.key, "request_count", 1)
                .await
                .expect("Couldn't query cache");
            self.request_count += 1;
            Ok(())
        } else {
            log::debug!("New bucket for {}", self.key);
            cache
                .hset_multiple::<&str, &str, u64, ()>(
                    &self.key,
                    &[("last_reset", now), ("request_count", 1)],
                )
                .await
                .expect("Couldn't query cache");
            Ok(())
        }
    }

    /// Wraps a response in a RateLimitHeaderWrapper which adds headers relevant to rate limiting
    pub fn wrap_response<T, E>(&self, data: T) -> Result<RateLimitHeaderWrapper<T>, E> {
        Ok(RateLimitHeaderWrapper {
            inner: data,
            rate_limit_reset: Header::new(
                "X-RateLimit-Reset",
                self.reset_after.as_millis().to_string(),
            ),
            rate_limit_max: Header::new("X-RateLimit-Max", self.request_limit.to_string()),
            rate_limit_last_reset: Header::new(
                "X-RateLimit-Last-Reset",
                self.last_reset.to_string(),
            ),
            rate_limit_request_count: Header::new(
                "X-RateLimit-Request-Count",
                self.request_count.to_string(),
            ),
        })
    }
}
