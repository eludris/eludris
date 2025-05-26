use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, query, Acquire, Postgres};

use crate::models::{ErrorResponse, File, Member, MemberEdit};

impl MemberEdit {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        if self.nickname.is_none()
            && self.sphere_avatar.is_none()
            && self.sphere_banner.is_none()
            && self.sphere_bio.is_none()
            && self.sphere_status.is_none()
        {
            return Err(error!(VALIDATION, "body", "At least one field must exist"));
        }
        if let Some(Some(nickname)) = &self.nickname {
            if nickname.is_empty() || nickname.len() > 32 {
                error!(
                    VALIDATION,
                    "nickname", "The member's nickname must be between 1 and 32 characters long"
                );
            }
        }
        if let Some(Some(bio)) = &self.sphere_bio {
            if bio.is_empty() || bio.len() > 4096 {
                error!(
                    VALIDATION,
                    "nickname", "The member's bio must be between 1 and 4096 characters long"
                );
            }
        }
        if let Some(Some(status)) = &self.sphere_status {
            if status.is_empty() || status.len() > 128 {
                error!(
                    VALIDATION,
                    "nickname", "The member's status must be between 1 and 128 characters long"
                );
            }
        }
        Ok(())
    }
}

impl Member {
    pub async fn edit<C: AsyncCommands>(
        id: u64,
        sphere_id: u64,
        edit: MemberEdit,
        db: &mut PoolConnection<Postgres>,
        requester_id: Option<u64>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        edit.validate()?;

        if Some(id) != requester_id
            && (edit.sphere_avatar.flatten().is_some()
                && edit.sphere_banner.clone().flatten().is_some()
                && edit.sphere_bio.clone().flatten().is_some()
                && edit.sphere_status.clone().flatten().is_some())
        {
            return Err(error!(
                VALIDATION,
                "body", "Other members can only set a user's nickname or reset their other fields"
            ));
        }

        query!(
            "
            SELECT nickname
            FROM members
            WHERE id = $1
            AND sphere_id = $2
            ",
            id as i64,
            sphere_id as i64
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch member {}'s sphere_id: {}", id, err);
            error!(SERVER, "Failed to fetch member")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;

        if let Some(Some(avatar)) = edit.sphere_avatar {
            if File::get(avatar, "member-avatars", &mut *db)
                .await
                .is_none()
            {
                return Err(error!(
                    VALIDATION,
                    "server_avatar",
                    "The members's avatar must be a valid file that exists in the member-avatars bucket"
                ));
            }
        }

        if let Some(Some(banner)) = edit.sphere_banner {
            if File::get(banner, "member-banners", &mut *db)
                .await
                .is_none()
            {
                return Err(error!(
                    VALIDATION,
                    "server_banner",
                    "The members's banner must be a valid file that exists in the member-banners bucket"
                ));
            }
        }

        let mut transaction = db.begin().await.map_err(|err| {
            log::error!("Couldn't start sphere edit transaction: {}", err);
            error!(SERVER, "Failed to edit member")
        })?;

        if let Some(ref nickname) = edit.nickname {
            sqlx::query!(
                "
         UPDATE members
         SET nickname = $1
         WHERE id = $2
         AND sphere_id = $3
                         ",
                *nickname,
                id as i64,
                sphere_id as i64
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't update {} members's nickname to {:?}: {}",
                    id,
                    nickname,
                    err
                );
                error!(SERVER, "Failed to edit member")
            })?;
        }

        if let Some(ref bio) = edit.sphere_bio {
            sqlx::query!(
                "
         UPDATE members
         SET sphere_bio = $1
         WHERE id = $2
         AND sphere_id = $3
                         ",
                *bio,
                id as i64,
                sphere_id as i64
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!("Couldn't update {} members's bio to {:?}: {}", id, bio, err);
                error!(SERVER, "Failed to edit member")
            })?;
        }

        if let Some(ref status) = edit.sphere_status {
            sqlx::query!(
                "
         UPDATE members
         SET sphere_status = $1
         WHERE id = $2
         AND sphere_id = $3
                         ",
                *status,
                id as i64,
                sphere_id as i64
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't update {} members's status to {:?}: {}",
                    id,
                    status,
                    err
                );
                error!(SERVER, "Failed to edit member")
            })?;
        }

        if let Some(avatar) = edit.sphere_avatar {
            sqlx::query!(
                "
         UPDATE members
         SET sphere_avatar = $1
         WHERE id = $2
         AND sphere_id = $3
                         ",
                avatar.map(|b| b as i64),
                id as i64,
                sphere_id as i64,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't update {} member's avatar to {:?}: {}",
                    id,
                    avatar,
                    err
                );
                error!(SERVER, "Failed to edit member")
            })?;
        }

        if let Some(banner) = edit.sphere_banner {
            sqlx::query!(
                "
         UPDATE members
         SET sphere_banner = $1
         WHERE id = $2
         AND sphere_id = $3
                         ",
                banner.map(|b| b as i64),
                id as i64,
                sphere_id as i64,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't update {} member's banner to {:?}: {}",
                    id,
                    banner,
                    err
                );
                error!(SERVER, "Failed to edit member")
            })?;
        }

        transaction.commit().await.map_err(|err| {
            log::error!("Couldn't commit member edit transaction: {}", err);
            error!(SERVER, "Failed to edit member")
        })?;

        Ok(Self::get(id, sphere_id, requester_id, db, cache).await?)
    }
}
