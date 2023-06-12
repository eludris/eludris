use serde::{Deserialize, Serialize};

use super::{InstanceInfo, Message, Status, User};
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
    /// The payload sent when the client gets gateway rate limited.
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
    /// The payload sent when the client has successfully authenticated. This contains the data the
    /// user needs on startup.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "AUTHENTICATED",
    ///   "user": {
    ///     "id": 48615849987334,
    ///     "username": "barbaz",
    ///     "social_credit": 3,
    ///     "badges": 0,
    ///     "permissions": 0
    ///   },
    ///   "users": [
    ///     {
    ///       "id": 48615849987333,
    ///       "username": "foobar",
    ///       "social_credit": 42,
    ///       "badges": 0,
    ///       "permissions": 0
    ///     }
    ///   ],
    /// }
    /// ```
    Authenticated {
        user: User,
        /// The currently online users who are relavent to the connector.
        users: Vec<User>,
    },
    /// The payload received when a user updates themselves. This includes both user updates from
    /// the [`update_user`] endpoint and profile updates from the [`update_profile`] endpoint.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "id": 48615849987333,
    ///   "username": "foobar",
    ///   "social_credit": 42,
    ///   "badges": 0,
    ///   "permissions": 0
    /// }
    /// ```
    UserUpdate(User),
    /// The payload sent when a user's presence is updated.
    ///
    /// This is mainly used for when a user goes offline or online.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "user_id": 48615849987333,
    ///   "status": {
    ///     "type": "IDLE",
    ///     "text": "BURY THE LIGHT DEEP WITHIN"
    ///   }
    /// }
    /// ```
    PresenceUpdate { user_id: u64, status: Status },
    /// The payload sent when the client receives a [`Message`].
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
    /// {
    ///   "op": "PING"
    /// }
    /// ```
    Ping,
    /// The first payload the client is supposed to send. The data of this payload is expected to
    /// be a session token obtained from the [`create_session`] route.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "AUTHENTICATE",
    ///   "d": "eyJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoyMzQxMDY1MjYxMDU3LCJzZXNzaW9uX2lkIjoyMzQxMDgyNDMxNDg5fQ.j-nMmVTLXplaC4opGdZH32DUSWt1yD9Tm9hgB9M6oi4" // You're not supposed to use this example token (eckd)
    /// }
    /// ```
    Authenticate(String),
}
