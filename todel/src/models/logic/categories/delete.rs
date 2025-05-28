use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{Category, ErrorResponse, Sphere};

impl Category {
    pub async fn delete(
        sphere_id: u64,
        category_id: u64,
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

        if sphere_id == category_id {
            return Err(error!(
                VALIDATION,
                "category", "The default category cannot be deleted"
            ));
        }

        let current_position = Category::get_unpopulated(category_id, db)
            .await
            .map_err(|err| {
                if let ErrorResponse::NotFound { .. } = err {
                    error!(VALIDATION, "category", "Category doesn't exist")
                } else {
                    err
                }
            })?
            .position;

        sqlx::query!(
            "
UPDATE categories
SET
    position = CASE
        WHEN (position = $2) THEN -1
        ELSE position - 1
        END,
    is_deleted = (position = $2)
WHERE sphere_id = $1
    AND position >= $2
    AND is_deleted = FALSE
            ",
            sphere_id as i64,
            current_position as i32,
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't update category positions after deletion: {}", err);
            error!(SERVER, "Failed to delete category")
        })?;

        Ok(())
    }
}
