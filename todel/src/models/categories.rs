use serde::{Deserialize, Serialize};


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
#[autodoc(category = "Channels")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Category {
    /// The ID of this category.
    pub id: u64,
    /// The ID of the sphere that this category belongs to.
    pub sphere_id: u64,
    /// The name of this category.
    pub name: String,
    /// This category's position inside of its sphere.
    pub position: u32,
}
