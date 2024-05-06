use async_recursion::async_recursion;
use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, Message, SphereChannel, User};

impl Message {
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
            log::error!("Couldn't fetch channel data {}: {}", id, err);
            error!(SERVER, "Failed to fetch channel data")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
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
            author: User::get(id, None, db, cache).await?,
            content: row.content,
            reference,
            disguise: None,
            channel: SphereChannel::get(row.channel_id as u64, db).await?,
            attachments: vec![],
        })
    }
}
