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
///   "author": "Not a weeb",
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
}

/// The MessageCreate payload. This is used when you want to create a message using the REST API.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "author": "Not a weeb",
///   "content": "Hello, World!"
/// }
/// ```
#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The message's author.
    pub author: User,
    /// There message's content.
    pub content: String,
}
