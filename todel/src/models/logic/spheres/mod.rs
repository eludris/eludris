mod get;
mod join;

use regex::Regex;
use sqlx::{
    pool::PoolConnection,
    postgres::{any::AnyConnectionBackend, PgRow},
    FromRow, Postgres, Row,
};

use crate::{
    ids::IdGenerator,
    models::{
        Category, ChannelType, ErrorResponse, File, Sphere, SphereChannel, SphereCreate,
        TextChannel,
    },
};

impl FromRow<'_, PgRow> for Sphere {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.get::<i64, _>("id") as u64,
            owner_id: row.get::<i64, _>("owner_id") as u64,
            name: row.get("name"),
            slug: row.get("slug"),
            sphere_type: row.get("sphere_type"),
            description: row.get("description"),
            icon: row.get::<Option<i64>, _>("icon").map(|a| a as u64),
            banner: row.get::<Option<i64>, _>("banner").map(|a| a as u64),
            badges: row.get::<i64, _>("badges") as u64,
            categories: vec![],
            members: vec![],
        })
    }
}

impl SphereCreate {
    pub async fn validate(
        &mut self,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        lazy_static! {
            static ref SLUG_REGEX: Regex =
                Regex::new(r"^[a-z0-9_-]+$").expect("Could not compile slug regex");
        };
        if !SLUG_REGEX.is_match(&self.slug) {
            return Err(error!(
                VALIDATION,
                "slug",
                "The spheres's slug must only consist of lowercase letters, numbers, underscores and dashes"
            ));
        }
        if self.slug.is_empty() || self.slug.len() > 32 {
            return Err(error!(
                VALIDATION,
                "slug", "The sphere's slug must be between 1 and 32 characters long"
            ));
        }
        if !self.slug.chars().any(|f| f.is_alphabetic()) {
            return Err(error!(
                VALIDATION,
                "slug", "The spheres's slug must have at least one alphabetical letter"
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
        mut sphere: SphereCreate,
        owner_id: u64,
        id_generator: &mut IdGenerator,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        sphere.validate(db).await?;
        let sphere_count = sqlx::query!(
            "
SELECT COUNT(id)
FROM spheres
WHERE owner_id = $1
            ",
            owner_id as i64
        )
        .fetch_one(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch user's sphere count: {}", err);
            error!(SERVER, "Failed to create sphere")
        })?
        .count;
        if sphere_count >= Some(100) {
            return Err(error!(VALIDATION, "spheres", "User exceeded sphere limit"));
        }
        let sphere_id = id_generator.generate();
        db.begin().await.map_err(|err| {
            log::error!("Couldn't create a new sphere transaction: {}", err);
            error!(SERVER, "Failed to create sphere")
        })?;
        sqlx::query(
            "
INSERT INTO spheres(id, owner_id, sphere_type, slug, description, icon, banner)
VALUES($1, $2, $3, $4, $5, $6, $7)
            ",
        )
        .bind(sphere_id as i64)
        .bind(owner_id as i64)
        .bind(&sphere.sphere_type)
        .bind(&sphere.slug)
        .bind(&sphere.description)
        .bind(sphere.icon.map(|i| i as i64))
        .bind(sphere.banner.map(|b| b as i64))
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't create a new sphere: {}", err);
            error!(SERVER, "Failed to create sphere")
        })?;
        sqlx::query(
            "
INSERT INTO categories(id, sphere_id, name, position)
VALUES($1, $1, 'uncategorised', 0)
            ",
        )
        .bind(sphere_id as i64)
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't create default sphere category: {}", err);
            error!(SERVER, "Failed to create sphere")
        })?;
        let channel_id = id_generator.generate();
        sqlx::query(
            "
INSERT INTO channels(id, sphere_id, category_id, channel_type, name, position)
VALUES($1, $2, $2, $3, $4, 0)
            ",
        )
        .bind(channel_id as i64)
        .bind(sphere_id as i64)
        .bind(ChannelType::Text)
        .bind("general")
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't create default sphere channel: {}", err);
            error!(SERVER, "Failed to create sphere")
        })?;
        db.commit().await.map_err(|err| {
            log::error!("Couldn't commit new sphere transaction: {}", err);
            error!(SERVER, "Failed to create sphere")
        })?;

        let member = Self::join(sphere_id, owner_id, db).await?;

        Ok(Self {
            id: sphere_id,
            owner_id,
            slug: sphere.slug,
            name: None,
            description: sphere.description,
            icon: sphere.icon,
            banner: sphere.banner,
            badges: 0,
            sphere_type: sphere.sphere_type,
            categories: vec![Category {
                id: sphere_id, // Special case: category with sphere id is to be treated as uncategorised.
                name: "uncategorised".to_string(),
                position: 0,
                channels: vec![SphereChannel::Text(TextChannel {
                    id: channel_id,
                    sphere_id,
                    name: "general".to_string(),
                    topic: None,
                    position: 0,
                    category_id: None,
                })],
            }],
            members: vec![member],
        })
    }

    pub async fn has_member(
        sphere_id: u64,
        user_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<bool, ErrorResponse> {
        Ok(sqlx::query!(
            "
SELECT id
FROM members
WHERE id = $2
AND sphere_id = $1
            ",
            sphere_id as i64,
            user_id as i64
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!(
                "Couldn't check if user {} is member of sphere {}: {}",
                sphere_id,
                user_id,
                err
            );
            error!(SERVER, "Failed to check user membership")
        })?
        .is_some())
    }
}
