use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, Sphere, SphereChannel};

impl SphereChannel {
    pub async fn delete(
        sphere_id: u64,
        channel_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        Sphere::get_unpopulated(sphere_id, db)
            .await
            .map_err(|err| {
                if let ErrorResponse::NotFound { .. } = err {
                    error!(VALIDATION, "sphere", "Sphere doesn't exist")
                } else {
                    err
                }
            })?;

        let current_channel = SphereChannel::get(channel_id, db).await?;

        sqlx::query!(
            "
UPDATE channels
SET
    position = CASE
        WHEN (position = $2) THEN -1
        ELSE position - 1
        END,
    is_deleted = (position = $2)
WHERE category_id = $1
    AND position >= $2
    AND is_deleted = FALSE
            ",
            current_channel.get_category_id() as i64,
            current_channel.get_position() as i32,
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't delete channel: {}", err);
            error!(SERVER, "Failed to delete channel")
        })?;

        Ok(())
    }
}
