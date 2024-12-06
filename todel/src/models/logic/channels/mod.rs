mod get;

use sqlx::{pool::PoolConnection, postgres::PgRow, Acquire, FromRow, Postgres, QueryBuilder, Row};

use crate::{
    ids::IdGenerator,
    models::{
        ChannelType, ErrorResponse, Sphere, SphereChannel, SphereChannelCreate, SphereChannelEdit,
        SphereChannelType, TextChannel, VoiceChannel,
    },
};

impl FromRow<'_, PgRow> for SphereChannel {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        match row.get::<ChannelType, _>("channel_type") {
            ChannelType::Text => Ok(Self::Text(crate::models::TextChannel {
                id: row.get::<i64, _>("id") as u64,
                sphere_id: row.get::<i64, _>("sphere_id") as u64,
                name: row.get("name"),
                topic: row.get("topic"),
                position: row.get::<i32, _>("position") as u32,
                category_id: row.get::<Option<i64>, _>("category_id").map(|i| i as u64),
            })),
            ChannelType::Voice => Ok(Self::Voice(crate::models::VoiceChannel {
                id: row.get::<i64, _>("id") as u64,
                sphere_id: row.get::<i64, _>("sphere_id") as u64,
                name: row.get("name"),
                position: row.get::<i32, _>("position") as u32,
                category_id: row.get::<Option<i64>, _>("category_id").map(|i| i as u64),
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

impl SphereChannelEdit {
    pub async fn validate(
        &self,
        sphere_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        // Position is guaranteed to be >0 as it's a u32, so we don't need to validate it here.
        if self.name.is_none()
            && self.topic.is_none()
            && self.position.is_none()
            && self.category_id.is_none()
        {
            return Err(error!(
                VALIDATION,
                "body",
                "You must provide at least one of 'name', 'topic', 'position' or 'category_id'"
            ));
        }
        if let Some(name) = &self.name {
            if name.is_empty() || name.len() > 32 {
                return Err(error!(
                    VALIDATION,
                    "name", "The channel's name must be between 1 and 32 characters long"
                ));
            }
        }
        if let Some(topic) = &self.topic {
            if topic.is_empty() || topic.len() > 4096 {
                return Err(error!(
                    VALIDATION,
                    "topic", "The channel's topic must be between 1 and 4096 characters long"
                ));
            }
        }
        if let Some(category_id) = &self.category_id {
            if self.position.is_none() {
                // Arbitrary, but seems more sane than just assuming a new position.
                return Err(error!(
                    VALIDATION,
                    "body", "Since 'category_id' is provided, 'position' must also be provided."
                ));
            }

            // Verify  that the category actually exists.
            sqlx::query!(
                "
SELECT *
FROM categories
WHERE id = $1 AND sphere_id = $2
                ",
                *category_id as i64,
                sphere_id as i64,
            )
            .fetch_optional(&mut **db)
            .await
            .map_err(|err| {
                log::error!("Couldn't fetch {} category: {}", category_id, err);
                error!(SERVER, "Failed to edit channel")
            })?
            .ok_or_else(|| {
                error!(
                    VALIDATION,
                    "category_id", "Category does not exist in the requested sphere."
                )
            })?;
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
        Sphere::get_unpopulated(sphere_id, db)
            .await
            .map_err(|err| {
                if let ErrorResponse::NotFound { .. } = err {
                    error!(VALIDATION, "sphere", "Sphere doesn't exist")
                } else {
                    err
                }
            })?;
        channel.validate()?;

        let category_id = channel.category_id.unwrap_or(sphere_id);
        let channel_count = sqlx::query!(
            "
SELECT COUNT(id)
FROM channels
WHERE sphere_id = $1 AND category_id = $2
            ",
            sphere_id as i64,
            category_id as i64,
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
INSERT INTO channels(id, sphere_id, channel_type, name, topic, position, category_id)
VALUES($1, $2, $3, $4, $5, $6, $7)
            ",
        )
        .bind(channel_id as i64)
        .bind(sphere_id as i64)
        .bind(channel.channel_type.get_channel_type())
        .bind(&channel.name)
        .bind(&channel.topic)
        .bind(channel_count)
        .bind(category_id as i64)
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't create default sphere channel: {}", err);
            error!(SERVER, "Failed to create sphere")
        })?;
        Ok(match channel.channel_type {
            SphereChannelType::Text => Self::Text(TextChannel {
                id: channel_id,
                sphere_id,
                name: channel.name,
                topic: channel.topic,
                position: channel_count as u32,
                category_id: Some(category_id),
            }),
            SphereChannelType::Voice => Self::Voice(VoiceChannel {
                id: channel_id,
                sphere_id,
                name: channel.name,
                position: channel_count as u32,
                category_id: Some(category_id),
            }),
        })
    }

    pub async fn edit(
        channel: SphereChannelEdit,
        sphere_id: u64,
        channel_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<SphereChannel, ErrorResponse> {
        Sphere::get_unpopulated(sphere_id, db)
            .await
            .map_err(|err| {
                if let ErrorResponse::NotFound { .. } = err {
                    error!(VALIDATION, "sphere", "Sphere doesn't exist")
                } else {
                    err
                }
            })?;
        channel.validate(sphere_id, db).await?;

        let current_channel = SphereChannel::get(channel_id, db).await?;

        let new_name = channel.name;
        let new_topic = channel.topic;
        let mut new_position = channel.position;
        let new_category = channel.category_id;

        let mut transaction = db.begin().await.map_err(|err| {
            log::error!("Couldn't start category edit transaction: {}", err);
            error!(SERVER, "Failed to edit category")
        })?;

        if new_name.is_some() || new_topic.is_some() {
            let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
                "
UPDATE channels
SET
                ",
            );

            if let Some(ref name) = new_name {
                query.push(" name = ").push_bind(name);
            }

            if let Some(ref topic) = new_topic {
                if new_name.is_some() {
                    query.push(", ");
                }
                query.push(" topic = ").push_bind(topic);
            }

            query
                .push(" WHERE id = ")
                .push_bind(channel_id as i64)
                .build()
                .execute(&mut *transaction)
                .await
                .map_err(|err| {
                    log::error!("Couldn't edit channel: {}", err);
                    error!(SERVER, "Failed to edit channel")
                })?;
        }

        if new_category.is_none_or(|cat| cat == current_channel.get_category_id()) {
            if let Some(mut position) = new_position {
                // Move within the same category
                let channel_count = sqlx::query!(
                    "
SELECT COUNT(id)
FROM channels
WHERE category_id = $1
                    ",
                    current_channel.get_category_id() as i64
                )
                .fetch_one(&mut *transaction)
                .await
                .map_err(|err| {
                    log::error!("Couldn't fetch sphere's channel count: {}", err);
                    error!(SERVER, "Failed to edit category")
                })?
                .count
                .ok_or_else(|| {
                    log::error!("Couldn't fetch sphere's channel count",);
                    error!(SERVER, "Failed to edit category")
                })? as u32;

                if position >= channel_count {
                    // If there's 6 channels, the max possible position is 5 (0 through 5 are set).
                    position = channel_count - 1;
                    new_position = Some(position);
                }

                sqlx::query!(
                    "
UPDATE channels
SET position = handle_edit_position($1, $2, position)
WHERE category_id = $3
                    ",
                    current_channel.get_position() as i32,
                    position as i32,
                    sphere_id as i64,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|err| {
                    log::error!("Couldn't update channel position: {}", err);
                    error!(SERVER, "Failed to edit channel")
                })?;
            }
        } else if let Some(category_id) = new_category {
            // Move between different categories
            let position = new_position.unwrap(); // Guaranteed to exist in SphereChannelEdit::validate

            sqlx::query!(
                "
UPDATE channels
SET
    category_id = edit_channel_category($1, $2, position, $3, $4, category_id),
    position    = edit_channel_position($1, $2, position, $3, $4, category_id)
WHERE category_id = $3 OR category_id = $4;
                ",
                current_channel.get_position() as i32,
                position as i32,
                current_channel.get_category_id() as i64,
                category_id as i64,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!("Couldn't update {} category: {}", category_id, err);
                error!(SERVER, "Failed to update category")
            })?;
        }

        transaction.commit().await.map_err(|err| {
            log::error!("Couldn't commit channel edit transaction: {}", err);
            error!(SERVER, "Failed to edit channel")
        })?;

        Ok(match current_channel {
            SphereChannel::Text(channel) => SphereChannel::Text(TextChannel {
                id: channel.id,
                category_id: new_category.or(channel.category_id),
                name: new_name.unwrap_or(channel.name),
                position: new_position.unwrap_or(channel.position),
                sphere_id: channel.sphere_id,
                topic: new_topic.or(channel.topic),
            }),
            SphereChannel::Voice(channel) => SphereChannel::Voice(VoiceChannel {
                id: channel.id,
                category_id: new_category.or(channel.category_id),
                name: new_name.unwrap_or(channel.name),
                position: new_position.unwrap_or(channel.position),
                sphere_id: channel.sphere_id,
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
