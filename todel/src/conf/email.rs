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
    #[serde(default = "subject_delete")]
    pub delete: String,
    #[serde(default = "subject_password_reset")]
    pub password_reset: String,
    #[serde(default = "subject_user_updated")]
    pub user_updated: String,
}

impl Default for EmailSubjects {
    fn default() -> Self {
        Self {
            verify: subject_verify(),
            delete: subject_verify(),
            password_reset: subject_password_reset(),
            user_updated: subject_user_updated(),
        }
    }
}

pub fn subject_verify() -> String {
    "Verify your Eludris account".to_string()
}

pub fn subject_delete() -> String {
    "Your Eludris account has been successfully deleted".to_string()
}

pub fn subject_password_reset() -> String {
    "Your Eludris password has been reset".to_string()
}

pub fn subject_user_updated() -> String {
    "Your Eludris account has been updated".to_string()
}
