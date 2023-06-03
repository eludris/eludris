use std::fmt;

use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;

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

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.display_name.as_ref().unwrap_or(&self.username)
        )
    }
}

/// The UserCreate payload.
///
/// This is used when a user is initially first created. For authentication payloads check
/// [`SessionCreate`].
///
/// -----
///
/// ### Examples
///
/// ```json
/// {
///   "username": "yendri",
///   "email": "yendri@llamoyendri.io",
///   "password": "autentícame por favor" // don't actually use this as a password
/// }
/// ```
#[autodoc(category = "Users")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserCreate {
    /// The user's name.
    ///
    /// This is different to their `display_name` as it denotes how they're more formally
    /// referenced by the API.
    pub username: String,
    /// The user's email.
    pub email: String,
    /// The user's password.
    pub password: String,
}

/// The SetUserProfile payload. This payload is used to update a user's profile. The abscence of a
/// field or it being `undefined` means that it won't have an effect. Explicitly setting a field as
/// `null` will clear it.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "display_name": "HappyRu",
///   "bio": "I am very happy!"
/// }
/// ```
#[autodoc(category = "Users")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateUserProfile {
    /// The user's new display name. This field has to be between 2 and 32 characters long.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub display_name: Option<Option<String>>,
    /// The user's new status. This field cannot be more than 128 characters long.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub status: Option<Option<String>>,
    /// The user's new bio. The upper limit is the instance's [`InstanceInfo`] `bio_limit`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub bio: Option<Option<String>>,
    /// The user's new avatar. This field has to be a valid file ID in the "avatar" bucket.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub avatar: Option<Option<u64>>,
    /// The user's new banner. This field has to be a valid file ID in the "banner" bucket.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub banner: Option<Option<u64>>,
}
