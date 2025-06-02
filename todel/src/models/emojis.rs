use serde::{Deserialize, Serialize};

#[autodoc(category = "Emojis")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Emoji {
    pub id: u64,
    pub file_id: u64,
    pub name: String,
}

#[autodoc(category = "Emojis")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmojiEdit {
    pub name: String,
}

#[autodoc(category = "Emojis")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReactionEmoji {
    Custom(Emoji),
    Unicode,(String),
}

#[autodoc(category = "Emojis")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: ReactionEmoji,
    pub user_ids: Vec<u64>,
}
