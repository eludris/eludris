use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemInfo {
    pub name: String,
    pub doc: Option<String>,
    pub category: String,
    pub hidden: bool,
    pub package: String,
    pub item: Item,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Item {
    Object(ObjectInfo),
    Enum(EnumInfo),
    Route(RouteInfo),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub doc: Option<String>,
    pub field_type: String,
    pub flattened: bool,
    pub nullable: bool,
    pub ommitable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectInfo {
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
    Struct {
        name: String,
        doc: Option<String>,
        fields: Vec<FieldInfo>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnumInfo {
    // `tag` & `content` are for the serde macro
    pub tag: Option<String>,
    pub untagged: bool,
    pub content: Option<String>,
    pub rename_all: Option<String>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {
    pub method: String,
    pub route: String,
    pub path_params: Vec<ParamInfo>,
    pub query_params: Vec<ParamInfo>,
    pub body_type: Option<String>,
    pub return_type: Option<String>,
    pub guards: Vec<String>,
}
