use std::{
    fmt::Display,
    time::{Duration, SystemTime},
};

use rocket::{http::Header, response::Responder};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{models::ErrorResponse, Conf};

use crate::Cache;

pub type RateLimitedRouteResponse<T> =
    Result<RateLimitHeaderWrapper<T>, RateLimitHeaderWrapper<ErrorResponse>>;

/// The necessary headers for responses
#[derive(Debug, Responder)]
pub struct RateLimitHeaderWrapper<T> {
    pub inner: T,
    pub rate_limit_reset: Header<'static>,
    pub rate_limit_max: Header<'static>,
    pub rate_limit_byte_max: Header<'static>,
    pub rate_limit_last_reset: Header<'static>,
    pub rate_limit_request_count: Header<'static>,
    pub rate_limit_sent_bytes: Header<'static>,
}

/// A simple struct that can do HTTP rate limiting
#[derive(Debug)]
pub struct RateLimiter {
    key: String,
    reset_after: Duration,
    request_limit: u32,
    file_size_limit: u64,
    request_count: u32,
    last_reset: u64,
    sent_bytes: u64,
}

impl RateLimiter {
    /// Creates a new RateLimiter
    pub fn new<I>(bucket: &str, attachment_bucket: &str, identifier: I, conf: &Conf) -> RateLimiter
    where
        I: Display,
    {
        let (reset_after, request_limit, file_size_limit) = match bucket {
            "assets" => (
                &conf.effis.rate_limits.assets.reset_after,
                &conf.effis.rate_limits.assets.limit,
                conf.effis.rate_limits.assets.file_size_limit,
            ),
            "attachments" => (
                &conf.effis.rate_limits.attachments.reset_after,
                &conf.effis.rate_limits.attachments.limit,
                conf.effis.rate_limits.attachments.file_size_limit,
            ),
            "fetch_file" => (
                &conf.effis.rate_limits.fetch_file.reset_after,
                &conf.effis.rate_limits.fetch_file.limit,
                0,
            ),

            _ => unreachable!(),
        };
        Self {
            key: format!("rate_limit:{}:{}-{}", identifier, bucket, attachment_bucket),
            reset_after: Duration::from_secs(*reset_after as u64),
            request_limit: *request_limit,
            file_size_limit,
            request_count: 0,
            last_reset: 0,
            sent_bytes: 0,
        }
    }

    /// Checks if a bucket is rate limited, if so returns an Error with an ErrorResponse
    pub async fn process_rate_limit(
        &mut self,
        bytes: u64,
        cache: &mut Connection<Cache>,
    ) -> Result<(), RateLimitHeaderWrapper<ErrorResponse>> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;

        if bytes > self.file_size_limit {
            return Err(self
                .wrap_response::<_, ()>(error!(
                    RATE_LIMITED,
                    self.last_reset + self.reset_after.as_millis() as u64 - now
                ))
                .unwrap());
        }

        if let (Some(last_reset), Some(request_count), Some(sent_bytes)) = cache
            .hget::<&str, (&str, &str, &str), (Option<u64>, Option<u32>, Option<u64>)>(
                &self.key,
                ("last_reset", "request_count", "sent_bytes"),
            )
            .await
            .expect("Couldn't query cache")
        {
            self.last_reset = last_reset;
            self.request_count = request_count;
            self.sent_bytes = sent_bytes;
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
                self.sent_bytes = 0;
                log::debug!("Reset bucket for {}", self.key);
            }
            if self.request_count >= self.request_limit {
                log::info!("Rate limited bucket {}", self.key);
                Err(self
                    .wrap_response::<_, ()>(error!(
                        RATE_LIMITED,
                        self.last_reset + self.reset_after.as_millis() as u64 - now
                    ))
                    .unwrap())
            } else if self.sent_bytes + bytes > self.file_size_limit {
                Err(self
                    .wrap_response::<_, ()>(error!(
                        RATE_LIMITED,
                        self.last_reset + self.reset_after.as_millis() as u64 - now
                    ))
                    .unwrap())
            } else {
                cache
                    .hincr::<&str, &str, u8, ()>(&self.key, "request_count", 1)
                    .await
                    .expect("Couldn't query cache");
                self.request_count += 1;
                cache
                    .hincr::<&str, &str, u64, ()>(&self.key, "sent_bytes", bytes)
                    .await
                    .expect("Couldn't query cache");
                self.sent_bytes += bytes;
                Ok(())
            }
        } else {
            log::debug!("New bucket for {}", self.key);
            cache
                .hset_multiple::<&str, &str, u64, ()>(
                    &self.key,
                    &[
                        ("last_reset", now),
                        ("request_count", 1),
                        ("sent_bytes", bytes),
                    ],
                )
                .await
                .expect("Couldn't query cache");
            Ok(())
        }
    }

    pub fn add_headers<T>(&self, data: T) -> RateLimitHeaderWrapper<T> {
        RateLimitHeaderWrapper {
            inner: data,
            rate_limit_reset: Header::new(
                "X-RateLimit-Reset",
                self.reset_after.as_millis().to_string(),
            ),
            rate_limit_max: Header::new("X-RateLimit-Max", self.request_limit.to_string()),
            rate_limit_byte_max: Header::new(
                "X-RateLimit-Byte-Max",
                self.file_size_limit.to_string(),
            ),
            rate_limit_last_reset: Header::new(
                "X-RateLimit-Last-Reset",
                self.last_reset.to_string(),
            ),
            rate_limit_request_count: Header::new(
                "X-RateLimit-Request-Count",
                self.request_count.to_string(),
            ),
            rate_limit_sent_bytes: Header::new(
                "X-RateLimit-Sent-Bytes",
                self.sent_bytes.to_string(),
            ),
        }
    }

    pub fn wrap_response<T, E>(&self, data: T) -> Result<RateLimitHeaderWrapper<T>, E> {
        Ok(self.add_headers(data))
    }
}
