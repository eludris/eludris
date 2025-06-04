use serde::{Deserialize, Serialize};

#[autodoc(category = "Emojis")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct Emoji {
    pub id: u64,
    pub file_id: u64,
    pub name: String,
    pub uploader_id: u64,
    pub sphere_id: u64,
}

#[autodoc(category = "Emojis")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmojiCreate {
    pub file_id: u64,
    pub name: String,
}

#[autodoc(category = "Emojis")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmojiEdit {
    pub name: String,
}

#[autodoc(category = "Emojis")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type", content = "emoji")]
pub enum ReactionEmoji {
    Custom(Emoji),
    Unicode(String),
}

impl ReactionEmoji {
    pub fn get_ref(&self) -> ReactionEmojiReference {
        match self {
            ReactionEmoji::Custom(emoji) => ReactionEmojiReference::Custom(emoji.id),
            ReactionEmoji::Unicode(emoji) => ReactionEmojiReference::Unicode(emoji.clone()),
        }
    }
}

#[autodoc(category = "Emojis")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type", content = "emoji")]
pub enum ReactionEmojiReference {
    Custom(u64),
    Unicode(String),
}

#[autodoc(category = "Emojis")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: ReactionEmoji,
    pub user_ids: Vec<u64>,
}
