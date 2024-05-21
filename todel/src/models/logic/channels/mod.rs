mod get;

use sqlx::{pool::PoolConnection, postgres::PgRow, FromRow, Postgres, Row};

use crate::{
    ids::IdGenerator,
    models::{
        Category, ChannelType, ErrorResponse, Sphere, SphereChannel, SphereChannelCreate,
        SphereChannelType, TextChannel, VoiceChannel,
    },
};

impl FromRow<'_, PgRow> for SphereChannel {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        match row.get::<ChannelType, _>("channel_type") {
            ChannelType::Category => Ok(Self::Category(crate::models::Category {
                id: row.get::<i64, _>("id") as u64,
                sphere_id: row.get::<i64, _>("sphere_id") as u64,
                name: row.get("name"),
                position: row.get::<i32, _>("position") as u32,
            })),
            ChannelType::Text => Ok(Self::Text(crate::models::TextChannel {
                id: row.get::<i64, _>("id") as u64,
                sphere_id: row.get::<i64, _>("sphere_id") as u64,
                name: row.get("name"),
                topic: row.get("topic"),
                position: row.get::<i32, _>("position") as u32,
            })),
            ChannelType::Voice => Ok(Self::Voice(crate::models::VoiceChannel {
                id: row.get::<i64, _>("id") as u64,
                sphere_id: row.get::<i64, _>("sphere_id") as u64,
                name: row.get("name"),
                position: row.get::<i32, _>("position") as u32,
            })),
            _ => unreachable!(),
        }
    }
}

impl SphereChannelCreate {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        if self.name.is_empty() || self.name.len() > 32 {
            return Err(error!(
                VALIDATION,
                "name", "The channel's name must be between 1 and 32 characters long"
            ));
        }
        if let Some(topic) = &self.topic {
            if topic.is_empty() || topic.len() > 4096 {
                return Err(error!(
                    VALIDATION,
                    "topic", "The channel's topic must be between 1 and 4096 characters long"
                ));
            }
        }
        Ok(())
    }
}

impl SphereChannel {
    pub async fn create(
        channel: SphereChannelCreate,
        sphere_id: u64,
        id_generator: &mut IdGenerator,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<SphereChannel, ErrorResponse> {
        channel.validate()?;
        Sphere::get_unpopulated(sphere_id, db)
            .await
            .map_err(|err| {
                if let ErrorResponse::NotFound { .. } = err {
                    error!(VALIDATION, "sphere", "Sphere doesn't exist")
                } else {
                    err
                }
            })?;
        let channel_count = sqlx::query!(
            "
SELECT COUNT(id)
FROM channels
WHERE sphere_id = $1
            ",
            sphere_id as i64
        )
        .fetch_one(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch sphere's channel count: {}", err);
            error!(SERVER, "Failed to create channel")
        })?
        .count
        .ok_or_else(|| {
            log::error!("Couldn't fetch sphere's channel count",);
            error!(SERVER, "Failed to create channel")
        })?;
        let channel_id = id_generator.generate();
        sqlx::query(
            "
INSERT INTO channels(id, sphere_id, channel_type, name, topic, position)
VALUES($1, $2, $3, $4, $5, $6)
            ",
        )
        .bind(channel_id as i64)
        .bind(sphere_id as i64)
        .bind(channel.channel_type.get_channel_type())
        .bind(&channel.name)
        .bind(&channel.topic)
        .bind(channel_count)
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't create default sphere channel: {}", err);
            error!(SERVER, "Failed to create sphere")
        })?;
        Ok(match channel.channel_type {
            SphereChannelType::Category => Self::Category(Category {
                id: channel_id,
                sphere_id,
                name: channel.name,
                position: channel_count as u32,
            }),
            SphereChannelType::Text => Self::Text(TextChannel {
                id: channel_id,
                sphere_id,
                name: channel.name,
                topic: channel.topic,
                position: channel_count as u32,
            }),
            SphereChannelType::Voice => Self::Voice(VoiceChannel {
                id: channel_id,
                sphere_id,
                name: channel.name,
                position: channel_count as u32,
            }),
        })
    }

    pub async fn has_member(
        channel_id: u64,
        user_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<bool, ErrorResponse> {
        Ok(sqlx::query!(
            "
SELECT members.id
FROM members
JOIN channels ON channels.id = $2
WHERE members.id = $1
AND members.sphere_id = channels.sphere_id
            ",
            user_id as i64,
            channel_id as i64
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!(
                "Couldn't check if user {} is member of channel {}'s sphere: {}",
                channel_id,
                user_id,
                err
            );
            error!(SERVER, "Failed to check user membership")
        })?
        .is_some())
    }
}
