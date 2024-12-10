use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{Category, ErrorResponse};

impl Category {
    pub async fn get_unpopulated(
        id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        let category = sqlx::query_as(
            "
SELECT *
FROM categories
WHERE id = $1
    AND is_deleted = FALSE
            ",
        )
        .bind(id as i64)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch {} category: {}", id, err);
            error!(SERVER, "Failed to get category")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
        Ok(category)
    }
}
