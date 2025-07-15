use serde::{Deserialize, Serialize};

use super::SpherePermissions;

#[autodoc(category = "Roles")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereRole {
    pub id: u64,
    pub sphere_id: u64,
    pub position: u32,
    pub name: String,
    pub colour: u32,
    pub allowed_permissions: SpherePermissions,
    pub denied_permissions: SpherePermissions,
}

#[autodoc(category = "Roles")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereRoleCreate {
    pub name: String,
    pub colour: Option<u32>,
    pub allowed_permissions: Option<SpherePermissions>,
    pub denied_permissions: Option<SpherePermissions>,
}

#[autodoc(category = "Roles")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereRoleEdit {
    pub position: Option<u32>,
    pub name: Option<String>,
    pub colour: Option<u32>,
    pub allowed_permissions: Option<SpherePermissions>,
    pub denied_permissions: Option<SpherePermissions>,
}
