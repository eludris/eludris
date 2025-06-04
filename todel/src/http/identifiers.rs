use std::{convert::Infallible, fmt::Display};

use rocket::request::FromParam;

/// A channel identifier that could be either a numerical id or a string slug.
#[autodoc(category = "Spheres")]
#[derive(Clone, Debug)]
pub enum SphereIdentifier {
    ID(u64),
    Slug(String),
}

impl Display for SphereIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SphereIdentifier::ID(id) => write!(f, "{}", id),
            SphereIdentifier::Slug(slug) => f.write_str(slug),
        }
    }
}

impl<'r> FromParam<'r> for SphereIdentifier {
    type Error = Infallible;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Ok(match param.parse() {
            Ok(id) => Self::ID(id),
            Err(_) => Self::Slug(param.to_string()),
        })
    }
}

/// A user identifier that could be either a self reference (@me), a numerical id or a string
/// username.
#[autodoc(category = "Users")]
#[derive(Clone, Debug)]
pub enum UserIdentifier {
    Me,
    ID(u64),
    Username(String),
}

impl Display for UserIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserIdentifier::Me => f.write_str("@me"),
            UserIdentifier::ID(id) => write!(f, "{}", id),
            UserIdentifier::Username(slug) => f.write_str(slug),
        }
    }
}

impl<'r> FromParam<'r> for UserIdentifier {
    type Error = Infallible;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if param == "@me" {
            Ok(Self::Me)
        } else {
            Ok(match param.parse() {
                Ok(id) => Self::ID(id),
                Err(_) => Self::Username(param.to_string()),
            })
        }
    }
}
