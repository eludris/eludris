mod account;
mod auth;
mod profile;
mod social;

use crate::models::{Status, User};
use sqlx::{postgres::PgRow, Database, Decode, FromRow, Row};

pub use account::*;

impl<'r, DB: Database> Decode<'r, DB> for Status
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(serde_json::from_str(<&str as Decode<DB>>::decode(value)?)
            .expect("Couldn't deserialize status type"))
    }
}

impl FromRow<'_, PgRow> for User {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.get::<i64, _>("id") as u64,
            username: row.get("username"),
            display_name: row.get("display_name"),
            social_credit: row.get("social_credit"),
            status: Status {
                status_type: row.get("status_type"),
                text: row.get("status"),
            },
            bio: row.get("bio"),
            avatar: row.get::<Option<i64>, _>("avatar").map(|a| a as u64),
            banner: row.get::<Option<i64>, _>("banner").map(|b| b as u64),
            badges: row.get::<i64, _>("badges") as u64,
            permissions: row.get::<i64, _>("permissions") as u64,
            email: Some(row.get("email")),
            verified: Some(row.get("verified")),
        })
    }
}
