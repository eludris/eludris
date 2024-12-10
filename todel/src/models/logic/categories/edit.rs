use sqlx::{pool::PoolConnection, Acquire, Postgres};

use crate::models::{Category, CategoryEdit, ErrorResponse, Sphere};

impl CategoryEdit {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        if self.name.is_none() && self.position.is_none() {
            return Err(error!(
                VALIDATION,
                "body", "You must provide at least one of 'name' or 'position'"
            ));
        }
        if let Some(name) = &self.name {
            if name.is_empty() || name.len() > 32 {
                return Err(error!(
                    VALIDATION,
                    "name", "The category's name must be between 1 and 32 characters long"
                ));
            }
        }
        if let Some(position) = self.position {
            if position < 1 {
                return Err(error!(
                    VALIDATION,
                    "position", "The category's position must be 1 or greater."
                ));
            }
        }
        Ok(())
    }
}

impl Category {
    pub async fn edit(
        mut category: CategoryEdit,
        sphere_id: u64,
        category_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(Category, CategoryEdit), ErrorResponse> {
        category.validate()?;

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
                "category", "The default category cannot be edited"
            ));
        }

        let current_category = Category::get_unpopulated(category_id, db)
            .await
            .map_err(|err| {
                if let ErrorResponse::NotFound { .. } = err {
                    error!(VALIDATION, "category", "Category doesn't exist")
                } else {
                    err
                }
            })?;

        let mut transaction = db.begin().await.map_err(|err| {
            log::error!("Couldn't start category edit transaction: {}", err);
            error!(SERVER, "Failed to edit category")
        })?;

        if let Some(ref name) = category.name {
            sqlx::query!(
                // Guaranteed to not be deleted by get_unpopulated
                "
UPDATE categories
SET name = $1
WHERE id = $2
                ",
                name,
                category_id as i64,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!("Couldn't update category name: {}", err);
                error!(SERVER, "Failed to edit category")
            })?;
        }

        if let Some(mut position) = category.position {
            let category_count = sqlx::query!(
                "
SELECT COUNT(id)
FROM categories
WHERE sphere_id = $1
    AND is_deleted = FALSE
                ",
                sphere_id as i64
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

            if position >= category_count {
                // If there's 6 categories, the max possible position is 5 (0 through 5 are set).
                position = category_count - 1;
                category.position = Some(position);
            }

            sqlx::query!(
                "
UPDATE categories
SET position = CASE
    WHEN (position = $1) THEN $2
    WHEN ($1 > $2)       THEN position + (position BETWEEN $2 AND $1)::int
    ELSE                      position - (position BETWEEN $1 AND $2)::int
    END
WHERE sphere_id = $3
    AND is_deleted=FALSE
                ",
                current_category.position as i64,
                position as i64,
                sphere_id as i64,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!("Couldn't update category position: {}", err);
                error!(SERVER, "Failed to edit category")
            })?;
        }

        transaction.commit().await.map_err(|err| {
            log::error!("Couldn't commit category edit transaction: {}", err);
            error!(SERVER, "Failed to edit category")
        })?;

        let result = {
            let category = category.clone();
            Category {
                id: category_id,
                name: category.name.unwrap_or(current_category.name),
                position: category.position.unwrap_or(current_category.position),
                channels: vec![],
            }
        };

        Ok((result, category))
    }
}
