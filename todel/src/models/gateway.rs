use serde::{Deserialize, Serialize};

use super::{InstanceInfo, Message};
use crate::conf::RateLimitConf;

/// Pandemonium websocket payloads sent by the server to the client.
#[autodoc(category = "Gateway")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "op", content = "d")]
pub enum ServerPayload {
    /// A [`ClientPayload`] `PING` payload response.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "PONG"
    /// }
    /// ```
    Pong,
    /// The event sent when the client gets gateway rate limited.
    ///
    /// The client is supposed to wait `wait` milliseconds before sending any more events,
    /// otherwise they are disconnected.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "RATE_LIMIT",
    ///   "d": {
    ///     "wait": 1010 // 1.01 seconds
    ///   }
    /// }
    /// ```
    RateLimit {
        /// The amount of milliseconds you have to wait before the rate limit ends
        wait: u64,
    },
    /// The payload sent by the server when you initiate a new gateway connection.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "HELLO",
    ///   "d": {
    ///     "heartbeat_interval": 45000,
    ///     "instance_info": {
    ///       "instance_name": "EmreLand",
    ///       "description": "More based than Oliver's instance (trust)",
    ///       "version": "0.3.3",
    ///       "message_limit": 2048,
    ///       "oprish_url": "https://example.com",
    ///       "pandemonium_url": "https://example.com",
    ///       "effis_url": "https://example.com",
    ///       "file_size": 20000000,
    ///       "attachment_file_size": 100000000
    ///     },
    ///     "rate_limit": {
    ///       "reset_after": 10,
    ///       "limit": 5
    ///     }
    ///   }
    /// }
    /// ```
    Hello {
        /// The amount of milliseconds your ping interval is supposed to be.
        heartbeat_interval: u64,
        /// The instance's info.
        ///
        /// This is the same payload you get from the [`get_instance_info`] payload without
        /// ratelimits
        instance_info: Box<InstanceInfo>,
        /// The pandemonium ratelimit info.
        rate_limit: RateLimitConf,
    },
    /// The event sent when the client receives a [`Message`].
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "MESSAGE_CREATE",
    ///   "d": {
    ///     "author": "A Certain Woo",
    ///     "content": "Woo!"
    ///   }
    /// }
    /// ```
    MessageCreate(Message),
}

/// Pandemonium websocket payloads sent by the client to the server.
#[autodoc(category = "Gateway")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "op", content = "d")]
pub enum ClientPayload {
    /// The payload the client is supposed to periodically send the server to not get disconnected.
    ///
    /// The interval where these pings are supposed to be sent can be found in the `HELLO` payload
    /// of the [`ServerPayload`] enum.
    ///
    /// -----
    ///
    /// > **Note**
    /// >
    /// > You are supposed to send your first ping in a connection after `RAND * heartbeat_interval` seconds,
    /// `RAND` being a random floating number between 0 and 1.
    /// >
    /// > This is done to avoid immediately overloading Pandemonium by connecting if it ever has to go down.
    ///
    /// ### Example
    ///
    /// ```json
    /// > {"op":"PING"}
    /// < {"op":"PONG"}
    /// ```
    Ping,
}
