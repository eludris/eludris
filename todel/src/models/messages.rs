use serde::{Deserialize, Serialize};

/// The message payload.
#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The author of message. This field has to be between 2 and 32 characters long.
    ///
    /// The author will be trimmed from leading and trailing whitespace.
    pub author: String,
    /// The content of the message. This field has to be at-least 2 characters long. The upper limit
    /// is the instance's [`InstanceInfo`] `message_limit`.
    ///
    /// You cannot send messages which are just whitespace.
    pub content: String,
}
