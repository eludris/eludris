use sqlx::{pool::PoolConnection, Acquire, Postgres};

use crate::models::{ErrorResponse, File, Sphere, SphereEdit, SphereType};

impl SphereEdit {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        if self.name.is_none()
            && self.sphere_type.is_none()
            && self.description.is_none()
            && self.icon.is_none()
            && self.banner.is_none()
        {
            return Err(error!(
                VALIDATION,
                "body",
                "At least one of 'name', 'topic', 'position' or 'category_id' must be provided."
            ));
        }
        if let Some(Some(name)) = &self.name {
            if name.is_empty() || name.len() > 32 {
                return Err(error!(
                    VALIDATION,
                    "name", "The sphere's name must be less between 1 and 32 characters long"
                ));
            }
        }
        if let Some(sphere_type) = &self.sphere_type {
            if sphere_type != &SphereType::Hybrid {
                return Err(error!(
                    VALIDATION,
                    "type",
                    "Spheres can only be upgraded to HYBRID and cannot be downgraded" // done
                ));
            }
        }
        if let Some(Some(description)) = &self.description {
            if description.is_empty() || description.len() > 4096 {
                return Err(error!(
                    VALIDATION,
                    "description",
                    "The sphere's description must be between 1 and 4096 characters long"
                ));
            }
        }
        Ok(())
    }
}

impl Sphere {
    pub async fn edit(
        edit: SphereEdit,
        sphere_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        edit.validate()?;

        let channel = Self::get_unpopulated(sphere_id, db).await.map_err(|err| {
            if let ErrorResponse::NotFound { .. } = err {
                error!(VALIDATION, "sphere", "Sphere doesn't exist")
            } else {
                err
            }
        })?;

        if let Some(Some(icon)) = edit.icon {
            if File::get(icon, "sphere-icons", &mut *db).await.is_none() {
                return Err(error!(
                    VALIDATION,
                    "icon",
                    "The sphere's icon must be a valid file that exists in the sphere-icons bucket"
                ));
            }
        }

        if let Some(Some(banner)) = edit.banner {
            if File::get(banner, "sphere-banners", &mut *db)
                .await
                .is_none()
            {
                return Err(error!(
                    VALIDATION,
                    "icon",
                    "The sphere's banner must be a valid file that exists in the sphere-banners bucket"
                ));
            }
        }

        let mut transaction = db.begin().await.map_err(|err| {
            log::error!("Couldn't start sphere edit transaction: {}", err);
            error!(SERVER, "Failed to edit sphere")
        })?;

        if let Some(ref name) = edit.name {
            sqlx::query!(
                "
UPDATE spheres
SET name = $1
WHERE id = $2
                ",
                *name,
                sphere_id as i64
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't update {} sphere's name to {:?}: {}",
                    sphere_id,
                    name,
                    err
                );
                error!(SERVER, "Failed to edit sphere")
            })?;
        }

        if let Some(ref sphere_type) = edit.sphere_type {
            sqlx::query(
                "
UPDATE spheres
SET sphere_type = $1
WHERE id = $2
                ",
            )
            .bind(sphere_type)
            .bind(sphere_id as i64)
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't update {} sphere's type to {}: {}",
                    sphere_id,
                    sphere_type,
                    err
                );
                error!(SERVER, "Failed to edit sphere")
            })?;
        }

        if let Some(ref description) = edit.description {
            sqlx::query!(
                "
UPDATE spheres
SET description = $1
WHERE id = $2
                ",
                *description,
                sphere_id as i64
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't update {} sphere's description to {:?}: {}",
                    sphere_id,
                    description,
                    err
                );
                error!(SERVER, "Failed to edit sphere")
            })?;
        }

        if let Some(icon) = edit.icon {
            sqlx::query!(
                "
UPDATE spheres
SET icon = $1
WHERE id = $2
                ",
                icon.map(|i| i as i64),
                sphere_id as i64
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't update {} sphere's icon to {:?}: {}",
                    sphere_id,
                    icon,
                    err
                );
                error!(SERVER, "Failed to edit sphere")
            })?;
        }

        if let Some(banner) = edit.banner {
            sqlx::query!(
                "
UPDATE spheres
SET banner = $1
WHERE id = $2
                ",
                banner.map(|b| b as i64),
                sphere_id as i64
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't update {} sphere's banner to {:?}: {}",
                    sphere_id,
                    banner,
                    err
                );
                error!(SERVER, "Failed to edit sphere")
            })?;
        }

        transaction.commit().await.map_err(|err| {
            log::error!("Couldn't commit sphere edit transaction: {}", err);
            error!(SERVER, "Failed to edit sphere")
        })?;

        Ok(Self {
            id: channel.id,
            owner_id: channel.owner_id,
            name: edit.name.unwrap_or(channel.name),
            slug: channel.slug,
            sphere_type: edit.sphere_type.unwrap_or(channel.sphere_type),
            description: edit.description.unwrap_or(channel.description),
            icon: edit.icon.unwrap_or(channel.icon),
            banner: edit.banner.unwrap_or(channel.banner),
            badges: channel.badges,
            categories: vec![],
            members: vec![],
        })
    }
}
