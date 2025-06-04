use serde::{Deserialize, Serialize};

#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type")]
pub enum Embed {
    Custom(CustomEmbed),
    Image {
        url: String,
        width: u32,
        height: u32,
    },
    Video {
        url: String,
        width: u32,
        height: u32,
    },
    Website {
        url: String,
        name: Option<String>,
        title: Option<String>,
        description: Option<String>,
        colour: Option<String>,
        image: Option<String>,
        image_width: Option<u32>,
        image_height: Option<u32>,
    },
    YouTubeVideo {
        url: String,
        title: String,
        video_id: String,
        description: Option<String>,
        channel: String,
        channel_url: String,
        timestamp: Option<u32>,
    },
    Spotify {
        url: String,
        title: String,
        iframe: String,
    },
}

#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomEmbed {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u8>,
    #[serde(default = "custom_embed_default_border_colour")]
    #[serde(skip_serializing_if = "is_zero")]
    pub border_colour: u8,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

fn custom_embed_default_border_colour() -> u8 {
    0x0
}
