use sqlx::{pool::PoolConnection, Acquire, Postgres};

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

        let mut transaction = db.begin().await.map_err(|err| {
            log::error!("Couldn't start category delete transaction: {}", err);
            error!(SERVER, "Failed to delete category")
        })?;

        sqlx::query!(
            "
DELETE FROM categories
WHERE id = $1 AND sphere_id = $2
            ",
            category_id as i64,
            sphere_id as i64,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|err| {
            log::error!("Couldn't delete category: {}", err);
            error!(SERVER, "Failed to delete category")
        })?;

        sqlx::query!(
            "
UPDATE categories
SET position = position - 1
WHERE sphere_id = $1 AND position > $2
            ",
            sphere_id as i64,
            current_position as i32,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|err| {
            log::error!("Couldn't update category positions after deletion: {}", err);
            error!(SERVER, "Failed to delete category")
        })?;

        transaction.commit().await.map_err(|err| {
            log::error!("Couldn't commit category delete transaction: {}", err);
            error!(SERVER, "Failed to delete category")
        })?;

        Ok(())
    }
}
