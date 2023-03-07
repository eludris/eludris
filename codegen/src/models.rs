use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    pub field_type: String,
    pub flattened: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub doc: Option<String>,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum EnumVariant {
    Unit {
        name: String,
        doc: Option<String>,
    },
    Tuple {
        name: String,
        doc: Option<String>,
        field_type: String,
    },
    Struct(StructInfo),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnumInfo {
    pub name: String,
    pub doc: Option<String>,
    // `tag` & `content` are for the serde macro
    pub tag: Option<String>,
    pub untagged: bool,
    pub content: Option<String>,
    pub rename_all: Option<String>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {}
