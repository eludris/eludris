use serde::{Deserialize, Serialize};

/// The message payload.
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
    /// The message's author. This field has to be between 2 and 32 characters long.
    ///
    /// The author will be trimmed from leading and trailing whitespace.
    pub author: String,
    /// The message's content the message. This field has to be at-least 2 characters long. The upper limit
    /// is the instance's [`InstanceInfo`] `message_limit`.
    ///
    /// The content will be trimmed from leading and trailing whitespace.
    pub content: String,
}
