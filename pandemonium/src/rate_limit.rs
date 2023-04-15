use std::{
    fmt::Display,
    time::{Duration, SystemTime},
};

use redis::{aio::Connection, AsyncCommands};

/// A simple RateLimiter than can keep track of rate limit data from KeyDB
pub struct RateLimiter {
    cache: Connection,
    key: String,
    reset_after: Duration,
    request_limit: u32,
    request_count: u32,
    last_reset: u64,
}

impl RateLimiter {
    /// Creates a new RateLimiter
    pub fn new<I>(
        cache: Connection,
        identifier: I,
        reset_after: Duration,
        request_limit: u32,
    ) -> RateLimiter
    where
        I: Display,
    {
        RateLimiter {
            cache,
            key: format!("rate_limit:pandemonium:{}", identifier),
            reset_after,
            request_limit,
            request_count: 0,
            last_reset: 0,
        }
    }

    /// Checks if a bucket is rate limited and returns the time until reset if so
    pub async fn process_rate_limit(&mut self) -> Result<(), u64> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;

        if let (Some(last_reset), Some(request_count)) = self
            .cache
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
                self.cache.del::<&str, ()>(&self.key).await.unwrap();
                self.cache
                    .hset_multiple::<&str, &str, u64, ()>(
                        &self.key,
                        &[("last_reset", now), ("request_count", 0)],
                    )
                    .await
                    .unwrap();
                self.last_reset = now;
                self.request_count = 0;
                log::debug!("Reset bucket for {}", self.key);
            }
            if self.request_count >= self.request_limit {
                log::info!("Rate limited bucket {}", self.key);
                Err(self.last_reset + self.reset_after.as_millis() as u64 - now)
            } else {
                self.cache
                    .hincr::<&str, &str, u8, ()>(&self.key, "request_count", 1)
                    .await
                    .unwrap();
                self.request_count += 1;
                Ok(())
            }
        } else {
            log::debug!("New bucket for {}", self.key);
            self.cache
                .hset_multiple::<&str, &str, u64, ()>(
                    &self.key,
                    &[("last_reset", now), ("request_count", 1)],
                )
                .await
                .unwrap();
            Ok(())
        }
    }
}
