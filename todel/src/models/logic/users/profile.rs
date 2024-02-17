use sqlx::{pool::PoolConnection, Postgres, QueryBuilder};

use crate::{
    models::{ErrorResponse, File, UpdateUserProfile, User},
    Conf,
};

impl UpdateUserProfile {
    pub async fn validate(
        &self,
        conf: &Conf,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        if self.display_name.is_none()
            && self.bio.is_none()
            && self.status.is_none()
            && self.status_type.is_none()
            && self.avatar.is_none()
            && self.banner.is_none()
        {
            return Err(error!(VALIDATION, "body", "At least one field must exist"));
        }
        if let Some(Some(display_name)) = &self.display_name {
            if display_name.len() < 2 || display_name.len() > 32 {
                return Err(error!(
                    VALIDATION,
                    "display_name",
                    "The user's display name must be between 2 and 32 characters in length"
                ));
            }
        }
        if let Some(Some(bio)) = &self.bio {
            if bio.is_empty() || bio.len() > conf.oprish.bio_limit {
                return Err(error!(
                    VALIDATION,
                    "bio",
                    format!(
                        "The user's bio must be between 1 and {} characters in length",
                        conf.oprish.bio_limit
                    )
                ));
            }
        }
        if let Some(Some(status)) = &self.status {
            if status.is_empty() || status.len() > 150 {
                return Err(error!(
                    VALIDATION,
                    "status",
                    "The user's status name must be between 1 and 150 characters in length"
                ));
            }
        }
        if let Some(Some(avatar)) = self.avatar {
            if File::get(avatar, "avatars", &mut *db).await.is_none() {
                return Err(error!(
                    VALIDATION,
                    "avatar", "The user's avatar must be a valid file that must exist"
                ));
            }
        }
        if let Some(Some(banner)) = self.banner {
            if File::get(banner, "banners", &mut *db).await.is_none() {
                return Err(error!(
                    VALIDATION,
                    "banner", "The user's banner must be a valid file that must exist"
                ));
            }
        }
        Ok(())
    }
}

impl User {
    pub async fn update_profile(
        id: u64,
        profile: UpdateUserProfile,
        conf: &Conf,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        profile.validate(conf, &mut *db).await?;
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE users SET ");
        let mut seperated = query.separated(", ");
        if let Some(display_name) = profile.display_name {
            seperated
                .push("display_name = ")
                .push_bind_unseparated(display_name);
        }
        if let Some(bio) = profile.bio {
            seperated.push("bio = ").push_bind_unseparated(bio);
        }
        if let Some(status) = profile.status {
            seperated.push("status = ").push_bind_unseparated(status);
        }
        if let Some(status_type) = profile.status_type {
            seperated
                .push("status_type = ")
                .push_bind_unseparated(status_type);
        }
        if let Some(avatar) = profile.avatar {
            seperated
                .push("avatar = ")
                .push_bind_unseparated(avatar.map(|a| a as i64));
        }
        if let Some(banner) = profile.banner {
            seperated
                .push("banner = ")
                .push_bind_unseparated(banner.map(|b| b as i64));
        }
        query
            .push(" WHERE id = ")
            .push_bind(id as i64)
            .push(
                " RETURNING id, username, display_name, social_credit, status, status_type, bio, avatar, banner, badges, permissions, email, verified",
            )
            .build_query_as()
            .fetch_one(&mut **db)
            .await
            .map_err(|err| {
                log::error!("Couldn't update user profile: {}", err);
                error!(SERVER, "Failed to update user profile")
            })
    }
}
