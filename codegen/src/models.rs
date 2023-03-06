use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ItemInfo {
    Struct(StructInfo),
    Enum(EnumInfo),
    Route(RouteInfo),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub doc: Option<String>,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub doc: Option<String>,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnumInfo {}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {}
