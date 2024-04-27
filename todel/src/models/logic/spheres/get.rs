use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, Sphere, SphereChannel};

impl Sphere {
    async fn populate_channels(
        &mut self,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        let channels: Vec<SphereChannel> = sqlx::query_as(
            "
SELECT *
FROM channels
WHERE sphere = $1
AND is_deleted = FALSE
            ",
        )
        .bind(self.id as i64)
        .fetch_all(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch channels for  {} sphere: {}", self.slug, err);
            error!(SERVER, "Failed to get sphere")
        })?;
        self.channels = channels;
        Ok(())
    }

    pub async fn get(id: u64, db: &mut PoolConnection<Postgres>) -> Result<Self, ErrorResponse> {
        let mut sphere: Self = sqlx::query_as(
            "
SELECT *
FROM spheres
WHERE id = $1
            ",
        )
        .bind(id as i64)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch {} sphere: {}", id, err);
            error!(SERVER, "Failed to get sphere")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
        sphere.populate_channels(db).await?;
        Ok(sphere)
    }

    pub async fn get_slug(
        slug: String,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        let mut sphere: Self = sqlx::query_as(
            "
SELECT *
FROM spheres
WHERE slug = $1
AND is_deleted = FALSE
            ",
        )
        .bind(&slug)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch {} sphere: {}", slug, err);
            error!(SERVER, "Failed to get sphere")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
        sphere.populate_channels(db).await?;
        Ok(sphere)
    }
}
