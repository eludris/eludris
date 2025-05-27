use async_recursion::async_recursion;
use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, types::Json, Postgres, QueryBuilder, Row};

use crate::models::{Embed, ErrorResponse, File, Message, SphereChannel, Status, StatusType, User};

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
        let attachment_ids = sqlx::query!(
            "
            SELECT attachment_id
            FROM message_attachments
            WHERE message_id = $1
            ",
            id as i64
        )
        .fetch_all(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch message attachments {}: {}", id, err);
            error!(SERVER, "Failed to fetch message data")
        })?;
        let embeds = sqlx::query!(
            r#"
            SELECT embed as "embed: Json<Embed>"
            FROM message_embeds
            WHERE message_id = $1
            "#,
            id as i64
        )
        .fetch_all(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch message embed {}: {}", id, err);
            error!(SERVER, "Failed to fetch message data")
        })?
        .into_iter()
        .map(|r| r.embed.0)
        .collect();
        let mut attachments = vec![];
        for attachment_id in attachment_ids {
            attachments.push(
                File::fetch_file_data(attachment_id.attachment_id as u64, "attachments", db)
                    .await?,
            );
        }
        Ok(Self {
            id: row.id as u64,
            author,
            content: row.content,
            reference,
            disguise: None,
            channel: SphereChannel::get(row.channel_id as u64, db).await?,
            attachments,
            embeds,
        })
    }

    pub async fn get_history<C: AsyncCommands>(
        channel_id: u64,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
        limit: u32,
        before: Option<u64>,
        after: Option<u64>,
    ) -> Result<Vec<Self>, ErrorResponse> {
        if !(1..=200).contains(&limit) {
            return Err(error!(
                VALIDATION,
                "limit", "Limit must be between 1 and 200, inclusive."
            ));
        }

        let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
            "
            SELECT *
            FROM messages
            WHERE channel_id =
            ",
        );
        query.push_bind(channel_id as i64);

        if let Some(id) = before {
            query.push(" AND id < ").push_bind(id as i64);
        };
        if let Some(id) = after {
            query.push(" AND id > ").push_bind(id as i64);
        };

        query
            .push(" ORDER BY id DESC ")
            .push(" LIMIT ")
            .push_bind(limit as i32);

        let rows = query.build().fetch_all(&mut **db).await.map_err(|err| {
            log::error!("Couldn't fetch channel history {}: {}", channel_id, err);
            error!(SERVER, "Failed to fetch channel history")
        })?;
        let mut messages = vec![];
        for row in rows {
            let id = row.get::<i64, _>("id") as u64;
            let author = match row.get::<Option<i64>, _>("author_id") {
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
            let reference = match row.get::<Option<i64>, _>("reference") {
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
            // not the most optimal way but whatever
            let attachment_ids = sqlx::query!(
                "
                SELECT attachment_id
                FROM message_attachments
                WHERE message_id = $1
                ",
                id as i64
            )
            .fetch_all(&mut **db)
            .await
            .map_err(|err| {
                log::error!("Couldn't fetch message attachments {}: {}", id, err);
                error!(SERVER, "Failed to fetch channel history")
            })?;
            let mut attachments = vec![];
            for attachment_id in attachment_ids {
                attachments.push(
                    File::fetch_file_data(attachment_id.attachment_id as u64, "attachments", db)
                        .await?,
                );
            }
            let embeds = sqlx::query!(
                r#"
            SELECT embed as "embed: Json<Embed>"
            FROM message_embeds
            WHERE message_id = $1
            "#,
                id as i64
            )
            .fetch_all(&mut **db)
            .await
            .map_err(|err| {
                log::error!("Couldn't fetch message embed {}: {}", id, err);
                error!(SERVER, "Failed to fetch message data")
            })?
            .into_iter()
            .map(|r| r.embed.0)
            .collect();
            messages.push(Self {
                id,
                author,
                content: row.get("content"),
                reference,
                disguise: None,
                channel: SphereChannel::get(row.get::<i64, _>("channel_id") as u64, db).await?,
                attachments,
                embeds,
            })
        }
        messages.reverse();
        Ok(messages)
    }
}
