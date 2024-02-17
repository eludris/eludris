use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, Status, StatusType, User};

impl User {
    pub async fn get_unfiltered(
        id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        sqlx::query_as(
            r#"
SELECT *
FROM users
WHERE id = $1
AND is_deleted = FALSE
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't get user from database: {}", err);
            error!(SERVER, "Failed to get user data")
        })?
        .ok_or_else(|| error!(NOT_FOUND))
    }

    #[allow(clippy::blocks_in_conditions)] // it's supposedly bad beacuse of code cleanness but
                                           // in this case it's cleaner
    pub async fn get<C: AsyncCommands>(
        id: u64,
        requester_id: Option<u64>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        let mut user = Self::get_unfiltered(id, db).await?;
        if Some(id) != requester_id {
            user.email = None;
            user.verified = None;
            if !cache
                .sismember::<_, _, bool>("sessions", id)
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
        }
        Ok(user)
    }

    #[allow(clippy::blocks_in_conditions)]
    pub async fn get_username<C: AsyncCommands>(
        username: &str,
        requester_id: Option<u64>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        let mut user: Self = sqlx::query_as(
            r#"
SELECT *
FROM users
WHERE username = $1
AND is_deleted = FALSE
            "#,
        )
        .bind(username)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't get user from database: {}", err);
            error!(SERVER, "Failed to get user data")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
        if Some(user.id) != requester_id {
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
        }
        Ok(user)
    }
}
