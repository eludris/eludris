use serde::{Deserialize, Serialize};

/// The user payload.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 48615849987333,
///   "username": "yendri",
///   "display_name": "Nicolas",
///   "social_credit": -69420,
///   "status": "ayúdame por favor",
///   "bio": "NICOLAAAAAAAAAAAAAAAAAAS!!!\n\n\nhttps://cdn.eludris.gay/static/nicolas.mp4",
///   "avatar": 2255112175647,
///   "banner": 2255049523230,
///   "badges": 0,
///   "permissions": 0
/// }
/// ```
#[autodoc(category = "Users")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// The user's ID.
    pub id: u64,
    /// The user's username. This field has to be between 2 and 32 characters long.
    pub username: String,
    /// The user's display name. This field has to be between 2 and 32 characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The user's social credit score.
    pub social_credit: i32,
    /// The user's status. This field cannot be more than 128 characters long.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// The user's bio. The upper limit is the instance's [`InstanceInfo`] `bio_limit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    /// The user's avatar. This field has to be a valid file ID in the "avatar" bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<u64>,
    /// The user's banner. This field has to be a valid file ID in the "banner" bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<u64>,
    /// The user's badges as a bitfield.
    pub badges: u64,
    /// The user's instance-wide permissions as a bitfield.
    pub permissions: u64,
}
