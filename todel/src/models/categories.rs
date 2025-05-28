use serde::{Deserialize, Serialize};

use super::SphereChannel;

/// A category that contains and orders channels.
///
/// A category can only exist inside a sphere.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "id": 4080402038798,
///   "sphere_id": 4080402038786,
///   "name": "channels (shocker)",
///   "position": 5
/// }
/// ```
#[autodoc(category = "Categories")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Category {
    /// The ID of this category.
    pub id: u64,
    /// The name of this category.
    pub name: String,
    /// This category's position inside of its sphere.
    pub position: u32,
    /// The channels that belong to this category.
    pub channels: Vec<SphereChannel>,
}

/// The CategoryCreate payload.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "name": "CategoryChannels begone"
/// }
/// ```
#[autodoc(category = "Categories", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryCreate {
    /// The name of this category.
    pub name: String,
}

/// The CategoryEdit payload.
///
/// -----
///
/// ### Example
///
/// ```json
/// {
///   "name": "CategoryChannels begone",
///   "position": 42,
/// }
/// ```
#[autodoc(category = "Categories", hidden = true)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryEdit {
    /// The name of this category.
    pub name: Option<String>,
    pub position: Option<u32>,
}
