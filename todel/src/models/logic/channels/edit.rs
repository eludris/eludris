use sqlx::{pool::PoolConnection, Acquire, Postgres, QueryBuilder};

use crate::models::{
    ErrorResponse, Sphere, SphereChannel, SphereChannelEdit, TextChannel, VoiceChannel,
};

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

            // TODO: Move DB check.
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
    pub async fn edit(
        mut channel: SphereChannelEdit,
        sphere_id: u64,
        channel_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(SphereChannel, SphereChannelEdit), ErrorResponse> {
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

        let mut transaction = db.begin().await.map_err(|err| {
            log::error!("Couldn't start category edit transaction: {}", err);
            error!(SERVER, "Failed to edit category")
        })?;

        if let Some(ref name) = channel.name {
            if name != current_channel.get_name() {
                sqlx::query!(
                    "
    UPDATE channels
    SET name = $1
    WHERE id = $2
                    ",
                    name,
                    channel_id as i64,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|err| {
                    log::error!("Couldn't edit channel: {}", err);
                    error!(SERVER, "Failed to edit channel")
                })?;
            }
        }

        if let Some(ref topic) = channel.topic {
            if Some(topic) != current_channel.get_topic() {
                sqlx::query!(
                    "
    UPDATE channels
    SET topic = $1
    WHERE id = $2
                    ",
                    topic,
                    channel_id as i64,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|err| {
                    log::error!("Couldn't edit channel: {}", err);
                    error!(SERVER, "Failed to edit channel")
                })?;
            }
        }

        if let Some(mut position) = channel.position {
            let destination_category = channel
                .category_id
                .unwrap_or_else(|| current_channel.get_category_id());

            if destination_category != current_channel.get_category_id()
                || position != current_channel.get_position()
            {
                // At least one of category and position changed, execute edit.
                let channel_count = sqlx::query!(
                    "
SELECT COUNT(id)
FROM channels
WHERE category_id = $1
                    ",
                    destination_category as i64
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

                if destination_category != current_channel.get_category_id() {
                    // Move between different categories
                    let category_id = channel.category_id.unwrap(); // Guaranteed to exist by above check
                    if position >= channel_count {
                        // Since we're actually moving between categories, our new position can be the
                        // current max position + 1 (i.e. equal to channel count.)
                        position = channel_count;
                        channel.position = Some(position);
                    }

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
                } else {
                    if position >= channel_count {
                        // If there's 6 channels, the max possible position is 5 (0 through 5 are set).
                        position = channel_count - 1;
                        channel.position = Some(position);
                    }

                    // Move within the same category
                    sqlx::query!(
                        "
UPDATE channels
SET position = edit_position($1, $2, position)
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
            }
        }

        transaction.commit().await.map_err(|err| {
            log::error!("Couldn't commit channel edit transaction: {}", err);
            error!(SERVER, "Failed to edit channel")
        })?;

        let result = {
            let channel = channel.clone();
            match current_channel {
                SphereChannel::Text(current_channel) => Self::Text(TextChannel {
                    id: channel_id,
                    sphere_id,
                    name: channel.name.unwrap_or(current_channel.name),
                    topic: channel.topic.or(current_channel.topic),
                    position: channel.position.unwrap_or(current_channel.position),
                    category_id: channel.category_id.unwrap_or(current_channel.category_id),
                }),
                SphereChannel::Voice(current_channel) => Self::Voice(VoiceChannel {
                    id: channel_id,
                    sphere_id,
                    name: channel.name.clone().unwrap_or(current_channel.name),
                    position: channel.position.unwrap_or(current_channel.position),
                    category_id: channel.category_id.unwrap_or(current_channel.category_id),
                }),
            }
        };

        Ok((result, channel))
    }
}
