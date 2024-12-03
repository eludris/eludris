use std::collections::HashMap;

use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, FromRow, Postgres, Row};

use crate::models::{
    Category, ErrorResponse, Member, Sphere, SphereChannel, Status, StatusType, User,
};

impl Sphere {
    pub(crate) async fn populate_channels(
        &mut self,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        let channels: Vec<SphereChannel> = sqlx::query_as(
            "
SELECT *
FROM channels
WHERE sphere_id = $1
AND is_deleted = FALSE
            ",
        )
        .bind(self.id as i64)
        .fetch_all(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch channels for {} sphere: {}", self.slug, err);
            error!(SERVER, "Failed to get sphere")
        })?;

        let mut categories: HashMap<u64, Category> = sqlx::query_as(
            "
SELECT *
FROM categories
WHERE sphere_id = $1
            ",
        )
        .bind(self.id as i64)
        .fetch_all(&mut **db)
        .await
        .map_err(|err| {
            log::error!(
                "Couldn't fetch categories for {} sphere: {}",
                self.slug,
                err
            );
            error!(SERVER, "Failed to get sphere")
        })?
        .into_iter()
        .map(|category: Category| (category.id, category))
        .collect();

        for channel in channels {
            let category_id = match &channel {
                SphereChannel::Text(category) => &category.category_id,
                SphereChannel::Voice(category) => &category.category_id,
            };
            categories
                .get_mut(&category_id.unwrap())
                .unwrap()
                .channels
                .push(channel);
        }

        self.categories = categories.into_values().collect();
        Ok(())
    }

    pub(crate) async fn populate_members<C: AsyncCommands>(
        &mut self,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<(), ErrorResponse> {
        let rows = sqlx::query(
            "
SELECT *
FROM members
JOIN users ON members.id = users.id
WHERE sphere_id = $1
AND members.is_deleted = FALSE
AND users.is_deleted = FALSE
            ",
        )
        .bind(self.id as i64)
        .fetch_all(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch members for {} sphere: {}", self.slug, err);
            error!(SERVER, "Failed to get sphere")
        })?;
        let mut members = vec![];
        for row in rows {
            let mut user = User::from_row(&row).map_err(|err| {
                log::error!("Couldn't fetch channels for  {} sphere: {}", self.slug, err);
                error!(SERVER, "Failed to get sphere")
            })?;
            user.email = None;
            user.verified = None;
            if !cache
                .sismember::<_, _, bool>("sessions", user.id)
                .await
                .map_err(|err| {
                    log::error!("Failed to determine if user is online: {}", err);
                    error!(SERVER, "Couldn't provide user data")
                })?
            {
                user.status = Status {
                    status_type: StatusType::Offline,
                    text: None,
                }
            }
            members.push(Member {
                user,
                sphere_id: self.id,
                nickname: row.get("nickname"),
                server_avatar: row.get::<Option<i64>, _>("server_avatar").map(|a| a as u64),
            })
        }
        self.members = members;
        Ok(())
    }

    pub async fn get<C: AsyncCommands>(
        id: u64,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        let mut sphere: Self = sqlx::query_as(
            "
SELECT *
FROM spheres
WHERE id = $1
            ",
        )
        .bind(id as i64)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch {} sphere: {}", id, err);
            error!(SERVER, "Failed to get sphere")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
        sphere.populate_channels(db).await?;
        sphere.populate_members(db, cache).await?;
        Ok(sphere)
    }

    pub async fn get_slug<C: AsyncCommands>(
        slug: String,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        let mut sphere: Self = sqlx::query_as(
            "
SELECT *
FROM spheres
WHERE slug = $1
AND is_deleted = FALSE
            ",
        )
        .bind(&slug)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch {} sphere: {}", slug, err);
            error!(SERVER, "Failed to get sphere")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
        sphere.populate_channels(db).await?;
        sphere.populate_members(db, cache).await?;
        Ok(sphere)
    }

    pub async fn get_unpopulated(
        id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        let sphere = sqlx::query_as(
            "
SELECT *
FROM spheres
WHERE id = $1
            ",
        )
        .bind(id as i64)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch {} sphere: {}", id, err);
            error!(SERVER, "Failed to get sphere")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
        Ok(sphere)
    }

    pub async fn get_unpopulated_slug(
        slug: String,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        let sphere = sqlx::query_as(
            "
SELECT *
FROM spheres
WHERE slug = $1
            ",
        )
        .bind(&slug)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch {} sphere: {}", slug, err);
            error!(SERVER, "Failed to get sphere")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
        Ok(sphere)
    }
}
