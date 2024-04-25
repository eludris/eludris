use sqlx::{pool::PoolConnection, Postgres};

use crate::{
    ids::IdGenerator,
    models::{ErrorResponse, File, Sphere, SphereCreate},
};

impl SphereCreate {
    pub async fn validate(&self, db: &mut PoolConnection<Postgres>) -> Result<(), ErrorResponse> {
        if self.slug.is_empty() || self.slug.len() > 32 {
            return Err(error!(
                VALIDATION,
                "slug", "The sphere's slug must be between 1 and 32 characters long"
            ));
        }
        if let Some(description) = &self.description {
            if description.is_empty() || description.len() > 4096 {
                return Err(error!(
                    VALIDATION,
                    "description",
                    "The sphere's description must be between 1 and 4096 characters long"
                ));
            }
        }
        if let Some(icon) = self.icon {
            if File::get(icon, "sphere-icons", &mut *db).await.is_none() {
                return Err(error!(
                    VALIDATION,
                    "icon", "The sphere's icon must be a valid file that must exist"
                ));
            }
        }
        if let Some(banner) = self.banner {
            if File::get(banner, "sphere-banners", &mut *db)
                .await
                .is_none()
            {
                return Err(error!(
                    VALIDATION,
                    "banner", "The sphere's banner must be a valid file that must exist"
                ));
            }
        }
        let sphere = sqlx::query!(
            "
SELECT id
FROM spheres
WHERE slug = $1
            ",
            self.slug
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!(
                "Failed to check if other spheres with the same slug exist: {}",
                err
            );
            error!(SERVER, "Couldn't create sphere")
        })?;
        if sphere.is_some() {
            return Err(error!(
                VALIDATION,
                "slug", "The sphere's slug must be unique"
            ));
        }
        Ok(())
    }
}

impl Sphere {
    pub async fn create(
        sphere: SphereCreate,
        owner_id: u64,
        id_generator: &mut IdGenerator,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        sphere.validate(db).await?;
        let _sphere = sqlx::query(
            r#"
INSERT INTO spheres(id, owner_id, sphere_type, slug, description, icon, banner)
VALUES($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id_generator.generate() as i64)
        .bind(owner_id as i64)
        .bind(sphere.sphere_type)
        .bind(sphere.slug)
        .bind(sphere.description)
        .bind(sphere.icon.map(|i| i as i64))
        .bind(sphere.banner.map(|b| b as i64))
        .execute(&mut **db)
        .await
        .unwrap();
        todo!()
    }
}
