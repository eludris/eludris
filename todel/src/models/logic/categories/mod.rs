mod get;

use sqlx::{pool::PoolConnection, postgres::PgRow, Acquire, FromRow, Postgres, Row};

use crate::{
    ids::IdGenerator,
    models::{Category, CategoryCreate, CategoryEdit, ErrorResponse, Sphere},
};

impl FromRow<'_, PgRow> for Category {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Category {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            position: row.get::<i32, _>("position") as u32,
            channels: vec![],
        })
    }
}

impl CategoryCreate {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        if self.name.is_empty() || self.name.len() > 32 {
            return Err(error!(
                VALIDATION,
                "name", "The category's name must be between 1 and 32 characters long"
            ));
        }
        Ok(())
    }
}

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
    pub async fn create(
        category: CategoryCreate,
        sphere_id: u64,
        id_generator: &mut IdGenerator,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Category, ErrorResponse> {
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

        let category_count = sqlx::query!(
            "
SELECT COUNT(id)
FROM categories
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

        let category_id = id_generator.generate();
        sqlx::query(
            "
INSERT INTO categories(id, sphere_id, name, position)
VALUES($1, $2, $3, $4)
            ",
        )
        .bind(category_id as i64)
        .bind(sphere_id as i64)
        .bind(&category.name)
        .bind(category_count)
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't create category: {}", err);
            error!(SERVER, "Failed to create category")
        })?;

        Ok(Category {
            id: category_id,
            name: category.name,
            position: category_count as u32,
            channels: vec![],
        })
    }

    pub async fn edit(
        category: CategoryEdit,
        sphere_id: u64,
        category_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Category, ErrorResponse> {
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

        let new_name = category.name;
        let mut new_position = category.position;

        let mut transaction = db.begin().await.map_err(|err| {
            log::error!("Couldn't start category edit transaction: {}", err);
            error!(SERVER, "Failed to edit category")
        })?;

        if let Some(ref name) = new_name {
            sqlx::query!(
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

        if let Some(mut position) = new_position {
            let category_count = sqlx::query!(
                "
SELECT COUNT(id)
FROM categories
WHERE sphere_id = $1
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
                new_position = Some(position);
            }

            sqlx::query!(
                "
UPDATE categories
SET position = handle_edit_position($1, $2, position)
WHERE sphere_id = $3
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

        Ok(Category {
            id: category_id,
            name: new_name.unwrap_or(current_category.name),
            position: new_position.unwrap_or(current_category.position),
            channels: vec![],
        })
    }

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
