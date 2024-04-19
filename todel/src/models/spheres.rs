use serde::{Deserialize, Serialize};

use super::SphereChannel;

/// The different types a sphere can be.
#[autodoc(category = "Spheres")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SphereType {
    /// Spheres that only support Discord-like chatrooms.
    Chat,
    /// Spheres that only support creating posts in forum style.
    Forum,
    /// Spheres that support both Discord-like chatrooms and form-like posts.
    Hybrid,
}

/// The sphere payload.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 4080402038786,
///   "owner_id": 4080403808259,
///   "name": "Spehre",
///   "type": "HYBRID",
///   "description": "Truly the sphere of all time",
///   "icon": 4080412852228,
///   "badges": 0,
///   "channels": [
///     {
///       "type": "TEXT",
///       "id": 4080402038789,
///       "sphere": 4080402038786,
///       "position": 1,
///       "name": "je-mappelle"
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
    pub name: String,
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
    /// The channels that this sphere contains.
    pub channels: Vec<SphereChannel>,
}