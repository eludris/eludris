use std::fmt;

use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;

/// The type of a user's status.
#[autodoc(category = "Users")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[cfg_attr(feature = "logic", derive(sqlx::Type))]
#[cfg_attr(feature = "logic", sqlx(type_name = "status"))]
#[cfg_attr(feature = "logic", sqlx(rename_all = "UPPERCASE"))]
pub enum StatusType {
    Online,
    Offline,
    Idle,
    Busy,
}

/// A user's status.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "type": "BUSY",
///   "text": "ayúdame por favor",
/// }
/// ```
#[autodoc(category = "Users")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    #[serde(rename = "type")]
    pub status_type: StatusType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

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
///   "status": {
///     "type": "BUSY",
///     "text": "ayúdame por favor",
///    },
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
    /// The user's status.
    pub status: Status,
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
    /// The user's email. This is only shown when the user queries their own data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The user's verification status. This is only shown when the user queries their own data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
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
/// ### Example
///
/// ```json
/// {
///   "username": "yendri",d
///   "email": "yendri@llamoyendri.io",
///   "password": "authentícame por favor" // don't actually use this as a password
/// }
/// ```
#[autodoc(category = "Users", hidden = true)]
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

/// The UpdateUser payload. Any field set to `null`, `undefined` or is missing will be disregarded
/// and won't affect the user.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "password": "authentícame por favor",
///   "username": "yendli",
///   "email": "yendli2@yemail.yom"
/// }
/// ```
#[autodoc(category = "Users", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateUser {
    /// The user's current password for validation.
    pub password: String,
    /// The user's new username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// The user's new email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The user's new password.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_password: Option<String>,
}

/// The UpdateUserProfile payload. This payload is used to update a user's profile. The abscence of a
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
#[autodoc(category = "Users", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateUserProfile {
    /// The user's new display name. This field has to be between 2 and 32 characters long.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub display_name: Option<Option<String>>,
    /// The user's new status. This field cannot be more than 150 characters long.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub status: Option<Option<String>>,
    /// The user's new status type. This must be one of `ONLINE`, `OFFLINE`, `IDLE` and `BUSY`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_type: Option<StatusType>,
    /// The user's new bio. The upper limit is the instance's [`InstanceInfo`] `bio_limit`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub bio: Option<Option<String>>,
    /// The user's new avatar. This field has to be a valid file ID in the "avatars" bucket.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub avatar: Option<Option<u64>>,
    /// The user's new banner. This field has to be a valid file ID in the "banners" bucket.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub banner: Option<Option<u64>>,
}

/// The CreatePasswordResetCode payload. This is used when a user wants to generate a code
/// to reset their password, most commonly because they forgot their old one.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "email": "someemail@ma.il"
/// }
/// ```
#[autodoc(category = "Users", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePasswordResetCode {
    /// The user's email.
    pub email: String,
}

/// The ResetPassword payload. This is used when the user wants to reset their password using a
/// password reset code.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "code": 234567,
///   "email": "someemail@ma.il",
///   "password": "wow such security"
/// }
/// ```
#[autodoc(category = "Users")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResetPassword {
    /// The password reset code the user got emailed.
    pub code: u32,
    /// The user's email.
    pub email: String,
    /// The user's new password.
    pub password: String,
}

/// The DeleteCredentials payload. This is used in multiple places in the API to provide extra
/// credentials for deleting important user-related stuff.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "password": "wowsuchpassword"
/// }
/// ```
#[autodoc(category = "Users")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PasswordDeleteCredentials {
    pub password: String,
}
