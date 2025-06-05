use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;

use super::{Attachment, AttachmentCreate, CustomEmbed, Embed, Reaction, SphereChannel, User};

/// The MessageCreate payload. This is used when you want to create a message using the REST API.
///
/// At least either content, an attachment or an embed have to exist.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "content": "Hello, World!",
///   "reference": 4080402038782
/// }
/// ```
#[autodoc(category = "Messaging", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageCreate {
    /// The message's content. The upper limit is the instance's [`InstanceInfo`] `message_limit`.
    ///
    /// Leading and trailing whitespace will be trimmed off the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default)]
    pub attachments: Vec<AttachmentCreate>,
    #[serde(default)]
    pub embeds: Vec<CustomEmbed>,
    /// The ID of the message referenced by this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_disguise")]
    pub disguise: Option<MessageDisguise>,
}

/// A temporary way to mask the message's author's name and avatar. This is mainly used for
/// bridging and will be removed when webhooks are officially supported.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "name": "Jeff",
///   "avatar": "https://some-u.rl/to/some-image.png"
/// }
/// ```
#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageDisguise {
    /// The name of the message's disguise.
    pub name: Option<String>,
    /// The URL of the message's disguise.
    pub avatar: Option<String>,
}

/// The MessageEdit payload. This is used when you want to edit an existing message using the REST
/// API.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "content": "~~I am smart~~ EDIT: I was wrong."
/// }
/// ```
#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageEdit {
    /// The message's updated content. The upper limit is the instance's [`InstanceInfo`] `message_limit`.
    /// If this is set to be an empty string it will be considered to be null,
    ///
    /// Leading and trailing whitespace will be trimmed off the content.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub content: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<AttachmentCreate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<CustomEmbed>>,
}

/// The Message payload. This is returned when you're provided information about a pre-existing
/// message.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 4080402038782,
///   "author": {
///      "id": 48615849987333,
///      "username": "mlynar",
///      "social_credit": 9999.
///      "badges": 256,
///      "permissions": 8
///   }
///   "content": "Hello, World!"
///   "channel": {
///     "type": "TEXT",
///     "id": 4080402038789,
///     "sphere": 4080402038786,
///     "position": 1,
///     "name": "je-mappelle"
///   }
/// }
/// ```
#[autodoc(category = "Messaging")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The ID of the message.
    pub id: u64,
    /// The message's author.
    pub author: User,
    /// The message's content.
    pub content: Option<String>,
    /// The message referenced by this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<Box<Message>>,
    /// The channel in which the message is sent.
    pub channel: SphereChannel,
    /// The attachments of this message.
    pub attachments: Vec<Attachment>,
    /// The embeds of this message.
    pub embeds: Vec<Embed>,
    /// The reactions of this message.
    pub reactions: Vec<Reaction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_disguise")]
    pub disguise: Option<MessageDisguise>,
}
