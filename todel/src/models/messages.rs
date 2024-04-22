use serde::{Deserialize, Serialize};

use super::User;
use super::Channel;

/// The MessageCreate payload. This is used when you want to create a message using the REST API.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "content": "Hello, World!",
///   "reference": 4080402038782
/// }
/// ```
#[autodoc(category = "Messaging", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageCreate {
    /// The message's content. This field has to be at-least 2 characters long. The upper limit
    /// is the instance's [`InstanceInfo`] `message_limit`.
    ///
    /// The content will be trimmed from leading and trailing whitespace.
    pub content: String,
    /// The ID of the message referenced by this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_disguise")]
    pub disguise: Option<MessageDisguise>,
}

/// A temporary way to mask the message's author's name and avatar. This is mainly used for
/// bridging and will be removed when webhooks are officially supported.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "name": "Jeff",
///   "avatar": "https://some-u.rl/to/some-image.png"
/// }
/// ```
#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageDisguise {
    /// The name of the message's disguise.
    pub name: Option<String>,
    /// The URL of the message's disguise.
    pub avatar: Option<String>,
}

/// The Message payload. This is returned when you're provided information about a pre-existing
/// message.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 4080402038782,
///   "author": {
///      "id": 48615849987333,
///      "username": "mlynar",
///      "social_credit": 9999.
///      "badges": 256,
///      "permissions": 8
///   }
///   "content": "Hello, World!"
///   "channel": {
///     "type": "TEXT",
///     "id": 4080402038789,
///     "sphere": 4080402038786,
///     "position": 1,
///     "name": "je-mappelle"
///   }
/// }
/// ```
#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The ID of the message.
    pub id: u64,
    /// The message's author.
    pub author: User,
    /// The message's content. This field has to be at-least 2 characters long. The upper limit
    /// is the instance's [`InstanceInfo`] `message_limit`.
    ///
    /// The content will be trimmed from leading and trailing whitespace.
    pub content: String,
    /// The message referenced by this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<Box<Message>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_disguise")]
    pub disguise: Option<MessageDisguise>,
    /// The channel in which the message is sent.
    pub channel: Channel,
}
