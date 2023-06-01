use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Email {
    pub relay: String,
    pub name: String,
    pub address: String,
    #[serde(default)]
    pub credentials: Option<EmailCredentials>,
    #[serde(default)]
    pub subjects: EmailSubjects,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailSubjects {
    #[serde(default = "subject_verify")]
    pub verify: String,
}

impl Default for EmailSubjects {
    fn default() -> Self {
        Self {
            verify: subject_verify(),
        }
    }
}

pub fn subject_verify() -> String {
    "Verify your Eludris account".to_string()
}
