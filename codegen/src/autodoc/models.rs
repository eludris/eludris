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
    pub r#type: String,
    pub nullable: bool,
    pub omittable: bool,
    pub flattened: bool,
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
    Object {
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
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {
    pub method: String,
    pub route: String,
    pub path_params: Vec<ParamInfo>,
    pub query_params: Vec<ParamInfo>,
    pub body: Option<Body>,
    pub response: Option<Response>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_auth: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body {
    pub r#type: String,
    pub format: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub r#type: String,
    pub format: String,
    pub status_code: u8,
    pub rate_limit: bool,
}
