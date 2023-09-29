use serde::{Deserialize, Serialize};

use super::User;

/// The MessageCreate payload. This is used when you want to create a message using the REST API.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "content": "Hello, World!"
/// }
/// ```
#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageCreate {
    /// The message's content. This field has to be at-least 2 characters long. The upper limit
    /// is the instance's [`InstanceInfo`] `message_limit`.
    ///
    /// The content will be trimmed from leading and trailing whitespace.
    pub content: String,
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
///   "author": {
///      "id": 48615849987333,
///      "username": "mlynar",
///      "social_credit": 9999.
///      "badges": 256,
///      "permissions": 8
///   }
///   "content": "Hello, World!"
/// }
/// ```
#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The message's author.
    pub author: User,
    /// There message's data.
    #[serde(flatten)]
    pub message: MessageCreate,
}
