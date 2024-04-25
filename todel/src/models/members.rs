use serde::{Deserialize, Serialize};

use super::User;

/// The Member payload. This represents a User in a sphere.
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
///   "sphere": 4080402038786,
///   "nickname": "Nicky"
/// }
/// ```
#[autodoc(category = "Members")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member {
    /// The underlying User for this member.
    pub user: User,
    /// The sphere to which this member belongs.
    pub sphere: u64,
    /// The nickname of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// The avatar of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_avatar: Option<u64>,
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
///   "server_avatar": 48615849987777
/// }
/// ```
#[autodoc(category = "Members", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberEdit {
    /// The nickname of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// The avatar of this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_avatar: Option<u64>,
}
