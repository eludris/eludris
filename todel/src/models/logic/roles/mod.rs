use sqlx::{pool::PoolConnection, Postgres};

use crate::{
    ids::IdGenerator,
    models::{ErrorResponse, Sphere, SpherePermissions, SphereRole, SphereRoleCreate},
};

impl SphereRoleCreate {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        if self.name.is_empty() || self.name.len() > 32 {
            return Err(error!(
                VALIDATION,
                "name", "The role's name must be between 1 and 32 characters in length"
            ));
        }
        if let Some(colour) = self.colour {
            if colour > 0xFFFFFF {
                return Err(error!(
                    VALIDATION,
                    "colour", "The role's colour value cannot be greater than 0xFFFFFF"
                ));
            }
        }
        if let Some(allowed) = &self.allowed_permissions {
            if *allowed > SpherePermissions::MAX {
                return Err(error!(
                    VALIDATION,
                    "allowed", "The role's allowed permission can't go beyond max permission value"
                ));
            }
        }
        if let Some(denied) = &self.denied_permissions {
            if *denied > SpherePermissions::MAX {
                return Err(error!(
                    VALIDATION,
                    "denied", "The role's denied permission can't go beyond max permission value"
                ));
            }
        }
        Ok(())
    }
}

impl Sphere {
    pub async fn create_role(
        &self,
        create: SphereRoleCreate,
        id_generator: &mut IdGenerator,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<SphereRole, ErrorResponse> {
        create.validate()?;

        let id = id_generator.generate();

        let role_count = sqlx::query!(
            "
SELECT COUNT(id)
FROM roles
WHERE sphere_id = $1
    AND is_deleted = FALSE
            ",
            self.id as i64
        )
        .fetch_one(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch sphere's role count: {}", err);
            error!(SERVER, "Failed to create role")
        })?
        .count
        .ok_or_else(|| {
            log::error!("Couldn't fetch sphere's role count",);
            error!(SERVER, "Failed to create role")
        })?;

        sqlx::query!(
            "
            INSERT INTO roles(id, sphere_id, position, name, colour, allowed, denied)
            VALUES($1, $2, $3, $4, $5, $6, $7)
            ",
            id as i64,
            self.id as i64,
            role_count as i32,
            create.name,
            create.colour.map(|c| c as i32),
            create.allowed_permissions.as_ref().map(|a| a.bits() as i64),
            create.denied_permissions.as_ref().map(|d| d.bits() as i64),
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to insert role into database: {}", err);
            error!(SERVER, "Failed to create role")
        })?;

        Ok(SphereRole {
            id,
            sphere_id: self.id,
            position: role_count as u32,
            name: create.name,
            colour: create.colour.unwrap_or(0),
            allowed_permissions: create
                .allowed_permissions
                .unwrap_or_else(SpherePermissions::empty),
            denied_permissions: create
                .denied_permissions
                .unwrap_or_else(SpherePermissions::empty),
        })
    }
}

impl SphereRole {
    pub async fn get(id: u64, db: &mut PoolConnection<Postgres>) -> Result<Self, ErrorResponse> {
        sqlx::query!(
            "
            SELECT *
            FROM roles
            WHERE id = $1
            AND is_deleted = FALSE
            ",
            id as i64
        )
        .fetch_one(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to get role into database {}: {}", id, err);
            error!(SERVER, "Failed to get role")
        })
        .map(|r| Self {
            id: r.id as u64,
            sphere_id: r.sphere_id as u64,
            position: r.position as u32,
            name: r.name,
            colour: r.colour as u32,
            allowed_permissions: SpherePermissions::from_bits(r.allowed as u64),
            denied_permissions: SpherePermissions::from_bits(r.allowed as u64),
        })
    }
}
