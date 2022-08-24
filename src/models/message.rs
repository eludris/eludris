//! Message related models.

use rocket::serde::{Deserialize, Serialize};

/// A struct representing a message a client sent via the rest api.
#[derive(Debug, Deserialize, Serialize)]
pub struct ClientMessage {
    author: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRespose {
    content: String,
}

impl MessageRespose {
    pub fn new(message: &ClientMessage) -> MessageRespose {
        MessageRespose {
            content: message.content.to_owned(),
        }
    }
}
