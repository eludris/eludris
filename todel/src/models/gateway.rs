use serde::{Deserialize, Serialize};

use super::Message;

/// Pandemonium websocket payloads sent by the server to the client
#[autodoc(category = "Gateway")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "op", content = "d")]
pub enum ServerPayload {
    /// A [`ClientPayload`] `PING` payload response.
    ///
    /// -----
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "op": "PONG"
    /// }
    /// ```
    Pong,
    /// The event sent when the client receives a [`Message`]
    ///
    /// -----
    ///
    /// ## Example
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

/// Pandemonium websocket payloads sent by the client to the server
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
    /// > **Note**
    /// >
    /// > You should send your first ping in a connection after `RAND * heartbeat_interval` seconds, RAND being a random
    /// floating number between 0 and 1
    /// >
    /// > This is done to avoid immediately overloading Pandemonium by connecting if it ever has to go down.
    Ping,
}
