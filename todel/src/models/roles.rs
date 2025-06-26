use serde::{Deserialize, Serialize};

use super::SpherePermissions;

#[autodoc(category = "Roles")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type")]
pub struct SphereRole {
    pub id: u64,
    pub sphere_id: u64,
    pub position: u32,
    pub name: String,
    pub allowed_permissions: SpherePermissions,
    pub denied_permissions: SpherePermissions,
}
