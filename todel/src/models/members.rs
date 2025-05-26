use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;

use super::User;

/// The Member payload. This represents a [`User`] in a sphere.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "user": {
///     "id": 48615849987333,
///     "username": "yendri",
///     "display_name": "Nicolas",
///     ...
///   },
///   "sphere_id": 4080402038786,
///   "nickname": "Nicky"
/// }
/// ```
#[autodoc(category = "Members")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member {
    /// The underlying User for this member.
    pub user: User,
    /// The sphere to which this member belongs.
    pub sphere_id: u64,
    /// The sphere-specific nickname of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// The sphere-specific avatar of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sphere_avatar: Option<u64>,
    /// The sphere-specific banner of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sphere_banner: Option<u64>,
    /// The sphere-specific bio of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sphere_bio: Option<String>,
    /// The sphere-specific status of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sphere_status: Option<String>,
}

/// The MemberEdit payload.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "nickname": "Nick",
///   "sphere_avatar": 48615849987777
/// }
/// ```
#[autodoc(category = "Members", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberEdit {
    /// The sphere-specific nickname of this member. This field has to be between 1 and 32
    /// characters.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub nickname: Option<Option<String>>,
    /// The sphere-specific avatar of this member. Has to be a valid file in the `member-avatars`
    /// bucket.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub sphere_avatar: Option<Option<u64>>,
    /// The sphere-specific banner of this member. Has to be a valid file in the `member-banners`
    /// bucket.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub sphere_banner: Option<Option<u64>>,
    /// The sphere-specific bio of this member. This field has to be between 1 and 4096 characters.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub sphere_bio: Option<Option<String>>,
    /// The sphere-specific status of this member. This field has to be between 1 and 128
    /// characters.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub sphere_status: Option<Option<String>>,
}
