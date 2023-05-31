use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Email {
    pub relay: String,
    pub name: String,
    pub address: String,
    #[serde(default)]
    pub credentials: Option<EmailCredentials>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailCredentials {
    pub username: String,
    pub password: String,
}
