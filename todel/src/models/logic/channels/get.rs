use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, SphereChannel};

impl SphereChannel {
    pub async fn get(id: u64, db: &mut PoolConnection<Postgres>) -> Result<Self, ErrorResponse> {
        sqlx::query_as(
            "
SELECT *
FROM channels
WHERE id = $1
    AND is_deleted = FALSE
            ",
        )
        .bind(id as i64)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch channel data {}: {}", id, err);
            error!(SERVER, "Failed to fetch channel data")
        })?
        .ok_or_else(|| error!(NOT_FOUND))
    }
}
