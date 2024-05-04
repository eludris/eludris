use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, Member, Sphere, User};

impl Sphere {
    pub async fn join(
        sphere_id: u64,
        user_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Member, ErrorResponse> {
        if sqlx::query!(
            "
SELECT id
FROM members
WHERE id = $1
AND sphere_id = $2
            ",
            user_id as i64,
            sphere_id as i64
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!(
                "Couldn't check if member {} is in sphere {}: {}",
                user_id,
                sphere_id,
                err
            );
            error!(SERVER, "Failed to join sphere")
        })?
        .is_some()
        {
            return Err(error!(
                VALIDATION,
                "sphere", "User is already in this sphere"
            ));
        }
        sqlx::query!(
            "
INSERT INTO members(id, sphere_id)
VALUES($1, $2)
            ",
            user_id as i64,
            sphere_id as i64
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't insert owner into new sphere: {}", err);
            error!(SERVER, "Failed to join sphere")
        })?;
        Ok(Member {
            sphere_id,
            user: User::get_unfiltered(user_id, db).await?,
            nickname: None,
            server_avatar: None,
        })
    }

    pub async fn join_slug(
        slug: String,
        user_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Member, ErrorResponse> {
        let sphere = Self::get_unpopulated_slug(slug, db).await?;
        if sqlx::query!(
            "
SELECT id
FROM members
WHERE id = $1
AND sphere_id = $2
            ",
            user_id as i64,
            sphere.id as i64
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!(
                "Couldn't check if member {} is in sphere {}: {}",
                user_id,
                sphere.id,
                err
            );
            error!(SERVER, "Failed to join sphere")
        })?
        .is_some()
        {
            return Err(error!(
                VALIDATION,
                "sphere", "User is already in this sphere"
            ));
        }
        sqlx::query!(
            "
INSERT INTO members(id, sphere_id)
VALUES($1, $2)
            ",
            user_id as i64,
            sphere.id as i64
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't insert owner into new sphere: {}", err);
            error!(SERVER, "Failed to join sphere")
        })?;
        Ok(Member {
            sphere_id: sphere.id,
            user: User::get_unfiltered(user_id, db).await?,
            nickname: None,
            server_avatar: None,
        })
    }
}
