use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;

use super::{Category, Member};

/// The different types a sphere can be.
#[autodoc(category = "Spheres")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[cfg_attr(feature = "logic", derive(sqlx::Type))]
#[cfg_attr(feature = "logic", sqlx(type_name = "sphere_type"))]
#[cfg_attr(feature = "logic", sqlx(rename_all = "UPPERCASE"))]
pub enum SphereType {
    /// Spheres that only support Discord-like chatrooms.
    Chat,
    /// Spheres that only support creating posts in forum style.
    Forum,
    /// Spheres that support both Discord-like chatrooms and forum-like posts.
    Hybrid,
}

impl Display for SphereType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SphereType::Chat => f.write_str("CHAT"),
            SphereType::Forum => f.write_str("FORUM"),
            SphereType::Hybrid => f.write_str("HYBRID"),
        }
    }
}

/// The Sphere payload.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 4080402038786,
///   "owner_id": 4080403808259,
///   "slug": "spehre",
///   "name": "Spehre",
///   "type": "HYBRID",
///   "description": "Truly the sphere of all time",
///   "icon": 4080412852228,
///   "badges": 0,
///   "categories": [
///     {
///       "id":5490083823619,
///       "name":"uncategorised",
///       "position":0,
///       "channels": [
///         {
///           "type":"TEXT",
///           "id":5490083823620,
///           "sphere_id":5490083823619,
///           "name":"general",
///           "position":0,
///           "category_id":5490083823619
///         }
///       ]
///     }
///   ],
///   "members": [
///     {
///       "user": {
///         "id": 5490049220609,
///         "username": "John-Mahjong",
///         "social_credit": 0,
///         "status": {
///           "type": "ONLINE"
///         },
///         "badges": 0,
///         "permissions": 0,
///         "email": "john.mahjong@example.com",
///         "verified": false
///       },
///       "sphere_id": 5490083823619
///     }
///   ]
/// }
/// ```
#[autodoc(category = "Spheres")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sphere {
    /// The spheres's ID.
    pub id: u64,
    /// The ID of the sphere's owner.
    pub owner_id: u64,
    /// The name of the sphere.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The slug of the sphere.
    pub slug: String,
    /// The sphere's type.
    #[serde(rename = "type")]
    pub sphere_type: SphereType,
    /// The sphere's description, can be between 1 and 4096 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The sphere's icon. This field has to be a valid file ID in the "sphere-icons" bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<u64>,
    /// The sphere's banner. This field has to be a valid file ID in the "sphere-banners" bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<u64>,
    /// The sphere's badges as a bitfield.
    pub badges: u64,
    /// The categories that this sphere contains.
    pub categories: Vec<Category>,
    /// The members that are inside this sphere.
    pub members: Vec<Member>,
}

/// The SphereCreate payload.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "slug": "frenche",
///   "type": "HYBRID",
///   "description": "Truly the sphere of all time",
///   "icon": 4080412852228,
/// }
/// ```
#[autodoc(category = "Spheres", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereCreate {
    /// The slug of the sphere. This field has to be between 1 and 32 characters.
    pub slug: String,
    /// The sphere's type.
    #[serde(rename = "type")]
    pub sphere_type: SphereType,
    /// The sphere's display name. This field has to be between 1 and 32 characters.
    pub name: Option<String>,
    /// The sphere's description. This field has to be between 1 and 4096 characters.
    pub description: Option<String>,
    /// The sphere's icon. This field has to be a valid file ID in the "sphere-icons" bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<u64>,
    /// The sphere's banner. This field has to be a valid file ID in the "sphere-banners" bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<u64>,
}

/// The SphereEdit payload.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "name": "Spehre",
///   "type": "HYBRID",
///   "description": "Truly the sphere of all time",
///   "icon": 4080412852228,
/// }
/// ```
#[autodoc(category = "Spheres", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereEdit {
    /// The sphere's display name. This field has to be between 1 and 32 characters.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub name: Option<Option<String>>,
    /// The sphere's type.
    #[serde(rename = "type")]
    pub sphere_type: Option<SphereType>, // Non-hybrid -> hybrid?
    /// The sphere's description, can be less than 4096 characters.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub description: Option<Option<String>>,
    /// The sphere's icon. This field has to be a valid file ID in the "sphere-icons" bucket.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub icon: Option<Option<u64>>,
    /// The sphere's banner. This field has to be a valid file ID in the "sphere-banners" bucket.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    pub banner: Option<Option<u64>>,
}
