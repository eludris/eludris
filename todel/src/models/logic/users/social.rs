use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, Status, StatusType, User};

impl User {
    #[allow(clippy::blocks_in_if_conditions)] // it's supposedly bad beacuse of code cleanness but
                                              // in this case it's cleaner
    pub async fn get<C: AsyncCommands>(
        id: u64,
        requester_id: Option<u64>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        sqlx::query!(
            r#"
SELECT id, username, display_name, social_credit, status, status_type as "status_type: StatusType", bio, avatar, banner, badges, permissions, email, verified
FROM users
WHERE id = $1
AND is_deleted = FALSE
            "#,
            id as i64
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't get user from database: {}", err);
            error!(SERVER, "Failed to get user data")
        })?
        .map(|u| async move {
            Ok(Self {
                id: u.id as u64,
                username: u.username,
                display_name: u.display_name,
                social_credit: u.social_credit,
                status: if  Some(id) == requester_id  ||
                    cache
                    .sismember::<_, _, bool>("sessions", u.id as u64)
                    .await
                    .map_err(|err| {
                        log::error!("Failed to determine if user is online: {}", err);
                        error!(SERVER, "Couldn't provide user data")
                    })? {
                        Status {
                        status_type: u.status_type,
                            text: u.status,
                        }
                } else {
                    Status {
                        status_type: StatusType::Offline,
                        text: None,
                    }
                },
                bio: u.bio,
                avatar: u.avatar.map(|a| a as u64),
                banner: u.banner.map(|b| b as u64),
                badges: u.badges as u64,
                permissions: u.permissions as u64,
                email: (Some(id) == requester_id).then_some(u.email),
                verified: (Some(id) == requester_id).then_some(u.verified)
            })
        })
        .ok_or_else(|| error!(NOT_FOUND))?.await
    }

    #[allow(clippy::blocks_in_if_conditions)]
    pub async fn get_username<C: AsyncCommands>(
        username: &str,
        requester_id: Option<u64>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        sqlx::query!(
            r#"
SELECT id, username, display_name, social_credit, status, status_type as "status_type: StatusType", bio, avatar, banner, badges, permissions, email, verified
FROM users
WHERE username = $1
AND is_deleted = FALSE
            "#,
            username
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't get user from database: {}", err);
            error!(SERVER, "Failed to get user data")
        })?
        .map(|u| async move {
            Ok(Self {
                id: u.id as u64,
                username: u.username,
                display_name: u.display_name,
                social_credit: u.social_credit,
                status: if Some(u.id as u64) == requester_id || cache
                    .sismember::<_, _, bool>("sessions", u.id as u64)
                    .await
                    .map_err(|err| {
                        log::error!("Failed to determine if user is online: {}", err);
                        error!(SERVER, "Couldn't provide user data")
                    })? {
                    Status {
                        status_type: u.status_type,
                        text: u.status,
                    }
                } else {
                    Status {
                        status_type: StatusType::Offline,
                        text: None,
                    }
                },
                bio: u.bio,
                avatar: u.avatar.map(|a| a as u64),
                banner: u.banner.map(|b| b as u64),
                badges: u.badges as u64,
                permissions: u.permissions as u64,
                email: (Some(u.id as u64) == requester_id).then_some(u.email),
                verified: (Some(u.id as u64) == requester_id).then_some(u.verified)
            })
        })
        .ok_or_else(|| error!(NOT_FOUND))?
        .await
    }
}
