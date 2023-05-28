use serde::{Deserialize, Serialize};

/// Represents a file stored on Effis.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 2195354353667,
///   "name": "das_ding.png",
///   "bucket": "attachments",
///   "metadata": {
///     "type": "image",
///     "width": 1600,
///     "height": 1600
///   }
/// }
/// ```
#[autodoc(category = "Files")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileData {
    /// The file's ID.
    pub id: u64,
    /// The file's name.
    pub name: String,
    /// The bucket the file is stored in.
    pub bucket: String,
    /// If the file is marked as a spoiler.
    #[serde(default = "spoiler_default")]
    #[serde(skip_serializing_if = "is_false")]
    pub spoiler: bool,
    /// The [`FileMetadata`] of the file.
    pub metadata: FileMetadata,
}

fn is_false(value: &bool) -> bool {
    !value
}

fn spoiler_default() -> bool {
    false
}

/// The enum representing all the possible Effis supported file metadatas.
///
/// -----
///
/// ### Examples
///
/// ```json
/// {
///   "type": "text"
/// }
/// {
///   "type": "image",
///   "width": 5120,
///   "height": 1440
/// }
/// {
///   "type": "video",
///   "width": 1920,
///   "height": 1080
/// }
/// {
///   "type": "other"
/// }
/// ```
#[autodoc(category = "Files")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum FileMetadata {
    Text,
    Image {
        /// The image's width in pixels.
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<usize>,
        /// The image's height in pixels.
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<usize>,
    },
    Video {
        /// The video's width in pixels.
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<usize>,
        /// The video's height in pixels.
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<usize>,
    },
    Other,
}

#[cfg(feature = "logic")]
pub struct File {
    pub id: u64,
    pub file_id: u64,
    pub name: String,
    pub content_type: String,
    pub hash: String,
    pub bucket: String,
    pub spoiler: bool,
    pub width: Option<usize>,
    pub height: Option<usize>,
}
