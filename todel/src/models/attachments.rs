use serde::{Deserialize, Serialize};

use super::FileData;

/// The AttachmentCreate payload. This is used when you want to attach a file to a message.
///
/// Only file_id is required.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "file_id": "2195354353667",
///   "description": "A cat smiling like a jolly fella"
/// }
/// ``
#[autodoc(category = "Messaging", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachmentCreate {
    /// The ID of the attached file.
    pub file_id: u64,
    /// Short description of the attached file, usually attributed to pictures. Max 256 characters.
    pub description: Option<String>,
    /// Whether the attachment is marked as a spoiler.
    #[serde(default = "spoiler_default")]
    #[serde(skip_serializing_if = "is_false")]
    pub spoiler: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}

fn spoiler_default() -> bool {
    false
}

/// The Attachment payload. This is returned alongside `Message` when you're provided information about a
/// pre-existing message.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "file": {
///       "id": 2195354353667,
///       "name": "das_ding.png",
///       "bucket": "attachments",
///       "metadata": {
///         "type": "IMAGE",
///         "width": 1600,
///         "height": 1600
///       }
///    },
///   "description": "Image of a beautiful creature.",
///   "spoiler": "false",
/// }
/// ```
#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    pub file: FileData,
    pub description: Option<String>,
    pub spoiler: bool,
}
