use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type")]
pub enum Embed {
    Custom(CustomEmbed),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomEmbed {
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<u8>,
    #[serde(default = "custom_embed_default_border_colour")]
    border_colour: u8,
}

fn custom_embed_default_border_colour() -> u8 {
    0x0
}
