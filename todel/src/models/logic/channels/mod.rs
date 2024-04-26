use sqlx::{postgres::PgRow, FromRow, Row};

use crate::models::{ChannelType, SphereChannel};

impl FromRow<'_, PgRow> for SphereChannel {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        match row.get::<ChannelType, _>("channel_type") {
            ChannelType::Category => Ok(Self::Category(crate::models::Category {
                id: row.get::<i64, _>("id") as u64,
                sphere: row.get::<i64, _>("sphere") as u64,
                name: row.get("name"),
                position: row.get::<i32, _>("position") as u32,
            })),
            ChannelType::Text => Ok(Self::Text(crate::models::TextChannel {
                id: row.get::<i64, _>("id") as u64,
                sphere: row.get::<i64, _>("sphere") as u64,
                name: row.get("name"),
                topic: row.get("topic"),
                position: row.get::<i32, _>("position") as u32,
            })),
            ChannelType::Voice => Ok(Self::Voice(crate::models::VoiceChannel {
                id: row.get::<i64, _>("id") as u64,
                sphere: row.get::<i64, _>("sphere") as u64,
                name: row.get("name"),
                position: row.get::<i32, _>("position") as u32,
            })),

            _ => unreachable!(),
        }
    }
}
