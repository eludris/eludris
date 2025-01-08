use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;

use super::User;

/// Valid Eludris "channel" types.
///
/// This is only internally used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[cfg_attr(feature = "logic", derive(sqlx::Type))]
#[cfg_attr(feature = "logic", sqlx(type_name = "channel_type"))]
#[cfg_attr(feature = "logic", sqlx(rename_all = "UPPERCASE"))]
pub(crate) enum ChannelType {
    /// A sphere text channel.
    Text,
    /// A sphere voice channel.
    Voice,
    /// A group channel.
    Group,
    /// A direct message channel.
    Direct,
}

/// Valid Eludris sphere "channel" types.
#[autodoc(category = "Channels")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SphereChannelType {
    /// A sphere text channel.
    Text,
    /// A sphere voice channel.
    Voice,
}

#[cfg(feature = "logic")]
impl SphereChannelType {
    pub(crate) fn get_channel_type(&self) -> ChannelType {
        match self {
            Self::Text => ChannelType::Text,
            Self::Voice => ChannelType::Voice,
        }
    }
}

/// The generic definition of the different types an Eludris "channel" can be.
#[autodoc(category = "Channels")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[serde(tag = "type")]
pub enum Channel {
    /// A sphere text channel.
    Text(TextChannel),
    /// A sphere voice channel.
    Voice(VoiceChannel),
    /// A group channel.
    Group(GroupChannel),
    /// A direct message channel.
    Direct(DirectMessageChannel),
}

/// The generic definition of all the different types an Eludris "channel" inside
/// a sphere can be.
#[autodoc(category = "Channels")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[serde(tag = "type")]
pub enum SphereChannel {
    /// A text channel.
    Text(TextChannel),
    /// A voice channel.
    Voice(VoiceChannel),
}

#[cfg(feature = "logic")]
impl SphereChannel {
    pub fn get_id(&self) -> u64 {
        match self {
            SphereChannel::Text(channel) => channel.id,
            SphereChannel::Voice(channel) => channel.id,
        }
    }

    pub fn get_category_id(&self) -> u64 {
        match self {
            SphereChannel::Text(channel) => channel.category_id,
            SphereChannel::Voice(channel) => channel.category_id,
        }
    }

    pub fn get_position(&self) -> u32 {
        match self {
            SphereChannel::Text(channel) => channel.position,
            SphereChannel::Voice(channel) => channel.position,
        }
    }

    pub fn get_name(&self) -> &String {
        match self {
            SphereChannel::Text(channel) => &channel.name,
            SphereChannel::Voice(channel) => &channel.name,
        }
    }

    pub fn get_topic(&self) -> Option<&String> {
        match self {
            SphereChannel::Text(channel) => channel.topic.as_ref(),
            SphereChannel::Voice(..) => None,
        }
    }
}

/// A Discord-like text channel.
///
/// This type of channel can only exist inside spheres.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 4080402038799,
///   "sphere_id": 4080402038786,
///   "name": "downtown-clowntown",
///   "topic": "gacha game channel",
///   "position": 3
/// }
/// ```
#[autodoc(category = "Channels")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextChannel {
    /// The ID of this text channel.
    pub id: u64,
    /// The ID of the sphere that this text channel belongs to.
    pub sphere_id: u64,
    /// The name of this text channel.
    pub name: String,
    /// The topic of this text channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// This text channel's position inside of its sphere.
    pub position: u32,
    /// The ID of the category this channel belongs to.
    pub category_id: u64,
}

/// A Discord-like voice channel.
///
/// This type of channel can only exist inside spheres.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 4080402038800,
///   "sphere_id": 4080402038786,
///   "name": "serious-chats-only",
///   "position": 7
/// }
/// ```
#[autodoc(category = "Channels")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceChannel {
    /// The ID of this voice channel.
    pub id: u64,
    /// The ID of the sphere that this voice channel belongs to.
    pub sphere_id: u64,
    /// The name of this voice channel.
    pub name: String,
    /// This voice channel's position inside of its sphere.
    pub position: u32,
    /// The ID of the category this channel belongs to.
    pub category_id: u64,
}

/// A Discord-like group channel, also known as a group DM.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 4080402038800,
///   "owner": 4080402038776,
///   "name": "abandoned project",
///   "members": [ ... ],
///   "topic": "The amazing group chat for our new banger world-changing project"
/// }
/// ```
#[autodoc(category = "Channels")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupChannel {
    /// The ID of this group channel.
    pub id: u64,
    /// The owner of this group channel.
    pub owner: User,
    /// The name of this group channel.
    pub name: String,
    /// The list of members inside this group channel.
    pub members: Vec<User>,
    /// The file ID of this group channel's icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<u64>,
    /// The topic of this group channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
}

/// A Discord-like private direct message channel.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 4080402038800,
///   "owner": 4080402038776,
///   "recipient": 4080402038777
/// }
/// ```
#[autodoc(category = "Channels")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectMessageChannel {
    /// The ID of this direct message channel.
    pub id: u64,
    /// The owner of this direct message channel.
    pub owner: User,
    /// The recipient of this direct message channel.
    pub recipient: User,
}

/// The SphereChannelCreate payload.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "name": "Canal pour Spehre",
///   "type": "TEXT",
///   "topic": "Boy do I love French"
/// }
/// ```
#[autodoc(category = "Channels", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereChannelCreate {
    /// The name of the new channel.
    pub name: String,
    /// The type of the new channel.
    #[serde(rename = "type")]
    pub channel_type: SphereChannelType,
    /// The topic of the new channel.
    pub topic: Option<String>,
    /// The category of the new channel.
    pub category_id: Option<u64>,
}

/// The SphereChannelEdit payload.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "name": "Channel for Sphere",
///   "topic": "Reject French",
///   "position": 1
/// }
/// ```
#[autodoc(category = "Channels", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereChannelEdit {
    /// The new name of the channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The new topic of the channel.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub topic: Option<Option<String>>,
    /// The new position of the channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
    /// The id of the new category of the channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<u64>,
}
