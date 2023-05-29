use std::net::IpAddr;

use serde::{Deserialize, Serialize};

/// The session payload.
///
/// The user should ideally have one session for every client they have on every device.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 2312155037697,
///   "user_id": 2312155693057,
///   "platform": "linux",
///   "client": "pilfer"
/// }
/// ```
#[autodoc(category = "Sessions")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// The session's ID.
    pub id: u64,
    /// The session user's ID.
    pub user_id: u64,
    /// The session's platform (linux, windows, mac, etc.)
    pub platform: String,
    /// The client the session was created by.
    pub client: String,
    /// The session's creation IP address.
    pub ip: IpAddr,
}

/// The SessionCreate payload.
///
/// This is used to authenticate a user and obtain a token to interface with the API.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "indentifier": "yendri",
///   "password": "authentícame por favor",
///   "platform": "linux",
///   "client": "pilfer"
/// }
/// ```
#[autodoc(category = "Sessions")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionCreate {
    /// The session user's identifier. This can be either their email or username.
    pub identifier: String,
    /// The session user's password.
    pub password: String,
    /// The session's platform (linux, windows, mac, etc.)
    pub platform: String,
    /// The client the session was created by.
    pub client: String,
}
