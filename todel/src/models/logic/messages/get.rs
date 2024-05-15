use async_recursion::async_recursion;
use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, Message, SphereChannel, Status, StatusType, User};

impl Message {
    #[allow(clippy::multiple_bound_locations)] // happens thanks to the `async_recursion` macro
    #[async_recursion]
    pub async fn get<C: AsyncCommands>(
        id: u64,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        let row = sqlx::query!(
            "
SELECT *
FROM messages
WHERE id = $1
            ",
            id as i64
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch message data {}: {}", id, err);
            error!(SERVER, "Failed to fetch message data")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
        let author = match row.author_id {
            Some(id) => User::get(id as u64, None, db, cache).await?,
            None => User {
                id: 0,
                username: "deleted-user".to_string(),
                display_name: Some("Deleted User".to_string()),
                social_credit: 0,
                status: Status {
                    status_type: StatusType::Offline,
                    text: None,
                },
                bio: None,
                avatar: None,
                banner: None,
                badges: 0,
                permissions: 0,
                email: None,
                verified: None,
            },
        };
        let reference = match row.reference {
            Some(reference) => match Self::get(reference as u64, db, cache).await {
                Ok(message) => Some(Box::new(message)),
                Err(err) => {
                    if let ErrorResponse::NotFound { .. } = err {
                        return Err(error!(
                            VALIDATION,
                            "reference", "Referenced message doesn't exist"
                        ));
                    } else {
                        return Err(err);
                    }
                }
            },
            None => None,
        };
        Ok(Self {
            id: row.id as u64,
            author,
            content: row.content,
            reference,
            disguise: None,
            channel: SphereChannel::get(row.channel_id as u64, db).await?,
            attachments: vec![],
        })
    }

    pub async fn get_history<C: AsyncCommands>(
        channel_id: u64,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Vec<Self>, ErrorResponse> {
        let rows = sqlx::query!(
            "
SELECT *
FROM messages
WHERE channel_id = $1
ORDER BY id ASC
LIMIT 1000
            ",
            channel_id as i64
        )
        .fetch_all(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch channel history {}: {}", channel_id, err);
            error!(SERVER, "Failed to fetch channel history")
        })?;
        let mut messages = vec![];
        for row in rows {
            let author = match row.author_id {
                Some(id) => User::get(id as u64, None, db, cache).await?,
                None => User {
                    id: 0,
                    username: "deleted-user".to_string(),
                    display_name: Some("Deleted User".to_string()),
                    social_credit: 0,
                    status: Status {
                        status_type: StatusType::Offline,
                        text: None,
                    },
                    bio: None,
                    avatar: None,
                    banner: None,
                    badges: 0,
                    permissions: 0,
                    email: None,
                    verified: None,
                },
            };
            let reference = match row.reference {
                Some(reference) => match Self::get(reference as u64, db, cache).await {
                    Ok(message) => Some(Box::new(message)),
                    Err(err) => {
                        if let ErrorResponse::NotFound { .. } = err {
                            return Err(error!(
                                VALIDATION,
                                "reference", "Referenced message doesn't exist"
                            ));
                        } else {
                            return Err(err);
                        }
                    }
                },
                None => None,
            };
            messages.push(Self {
                id: row.id as u64,
                author,
                content: row.content,
                reference,
                disguise: None,
                channel: SphereChannel::get(row.channel_id as u64, db).await?,
                attachments: vec![],
            })
        }
        Ok(messages)
    }
}
