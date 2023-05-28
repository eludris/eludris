use lazy_static::lazy_static;
use regex::Regex;

use crate::models::{ErrorResponse, UserCreate};

impl UserCreate {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        lazy_static! {
            static ref USERNAME_REGEX: Regex =
                Regex::new(r"^[a-z0-9_-]+$").expect("Could not compile username regex");
            // https://stackoverflow.com/a/201378
            static ref EMAIL_REGEX: Regex = Regex::new(r#"^(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])$"#).expect("Could not compile email regex");
        };

        if !USERNAME_REGEX.is_match(&self.username) {
            Err(error!(
                VALIDATION,
                "username",
                "The user's username must only consist of lowercase letters, numbers, underscores and dashes"
            ))
        } else if self.username.len() < 2 || self.username.len() > 32 {
            Err(error!(
                VALIDATION,
                "username", "The user's username must be between 2 and 32 characters in length"
            ))
        } else if !EMAIL_REGEX.is_match(&self.email) {
            Err(error!(
                VALIDATION,
                "email", "The user's email must be valid"
            ))
        } else if self.password.len() < 8 {
            Err(error!(
                VALIDATION,
                "password", "The user's password must be be at least 8 characters long"
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::UserCreate;

    macro_rules! test_user_create_error {
        (username: $username:expr) => {
            let user = UserCreate {
                username: $username.to_string(),
                email: "yendri@llamoyendri.io".to_string(),
                password: "autentícame por favor".to_string(),
            };
            assert!(user.validate().is_err());
        };
        (email: $email:expr) => {
            let user = UserCreate {
                username: "yendri".to_string(),
                email: $email.to_string(),
                password: "autentícame por favor".to_string(),
            };
            assert!(user.validate().is_err());
        };
        (password: $password:expr) => {
            let user = UserCreate {
                username: "yendri".to_string(),
                email: "yendri@llamoyendri.io".to_string(),
                password: $password.to_string(),
            };
            assert!(user.validate().is_err());
        };
    }

    #[test]
    fn validate_user_create() {
        let user = UserCreate {
            username: "yendri".to_string(),
            email: "yendri@llamoyendri.io".to_string(),
            password: "autentícame por favor".to_string(),
        };

        assert!(user.validate().is_ok());

        test_user_create_error!(username: "y"); // one character
        test_user_create_error!(username: "yendri_jesus_sanchez_gonzalez1988"); // too long
        test_user_create_error!(username: "yendri sanchez"); // spaces
        test_user_create_error!(username: "sánchez"); // unicode
        test_user_create_error!(username: "Yendri"); // capital letters

        test_user_create_error!(email: "no"); // invalid email

        test_user_create_error!(password: "1234"); // too short
    }
}
