use serde::{Deserialize, Serialize};

/// The data Effis provides for files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub id: u64,
    pub name: String,
    pub bucket: String,
    #[serde(default = "spoiler_default")]
    #[serde(skip_serializing_if = "is_false")]
    pub spoiler: bool,
    pub metadata: FileMetadata,
}

fn is_false(value: &bool) -> bool {
    !value
}

fn spoiler_default() -> bool {
    false
}

/// The enum representing all the possible Effis supported file metadatas
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum FileMetadata {
    Text,
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<usize>,
    },
    Video {
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<usize>,
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
