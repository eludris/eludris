use std::collections::HashMap;

use async_recursion::async_recursion;
use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, types::Json, Postgres, QueryBuilder, Row};

use crate::models::{
    Attachment, Embed, Emoji, ErrorResponse, File, Message, Reaction, ReactionEmoji, SphereChannel,
    Status, StatusType, User,
};

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
        let attachment_rows = sqlx::query!(
            "
            SELECT *, 
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
        for attachment_row in attachment_rows {
            let file = match File::get(attachment_row.file_id, "attachments", db).await {
                Some(file) => file,
                None => {
                    return Err(error!(
                        VALIDATION,
                        "attachment-file", "Attachment file has vanished..."
                    ))
                }
            };
            attachments.push(Attachment {
                file: file.get_file_data(),
                description: attachment_row.description,
                spoiler: attachment_row.spoiler,
            });
        }

        let mut reactions = HashMap::new();
        for row in sqlx::query!(
            "
            SELECT *
            FROM reactions
            WHERE message_id = $1
            ",
            id as i64
        )
        .fetch_all(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch reactions for message {}: {}", id, err);
            error!(SERVER, "Failed to fetch message data")
        })? {
            let emoji = if let Some(emoji) = row.emoji_id {
                ReactionEmoji::Custom(Emoji::get(emoji as u64, db).await?)
            } else {
                ReactionEmoji::Unicode(row.unicode_emoji.unwrap())
            };
            reactions
                .entry(emoji)
                .or_insert(vec![])
                .push(row.user_id as u64);
        }
        Ok(Self {
            id,
            author,
            content: row.content,
            reference,
            disguise: None,
            channel: SphereChannel::get(row.channel_id as u64, db).await?,
            attachments,
            embeds,
            reactions: reactions
                .into_iter()
                .map(|(emoji, user_ids)| Reaction { emoji, user_ids })
                .collect(),
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
            let attachment_rows = sqlx::query!(
                "
                SELECT *, 
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

            let mut attachments = vec![];
            for attachment_row in attachment_rows {
                let file = match File::get(attachment_row.file_id, "attachments", db).await {
                    Some(file) => file,
                    None => {
                        return Err(error!(
                            VALIDATION,
                            "attachment-file", "Attachment file has vanished..."
                        ))
                    }
                };
                attachments.push(Attachment {
                    file: file.get_file_data(),
                    description: attachment_row.description,
                    spoiler: attachment_row.spoiler,
                });
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
            let mut reactions = HashMap::new();
            for row in sqlx::query!(
                "
            SELECT *
            FROM reactions
            WHERE message_id = $1
            ",
                id as i64
            )
            .fetch_all(&mut **db)
            .await
            .map_err(|err| {
                log::error!("Couldn't fetch reactions for message {}: {}", id, err);
                error!(SERVER, "Failed to fetch message data")
            })? {
                let emoji = if let Some(emoji) = row.emoji_id {
                    ReactionEmoji::Custom(Emoji::get(emoji as u64, db).await?)
                } else {
                    ReactionEmoji::Unicode(row.unicode_emoji.unwrap())
                };
                reactions
                    .entry(emoji)
                    .or_insert(vec![])
                    .push(row.user_id as u64);
            }
            messages.push(Self {
                id,
                author,
                content: row.get("content"),
                reference,
                disguise: None,
                channel: SphereChannel::get(row.get::<i64, _>("channel_id") as u64, db).await?,
                attachments,
                embeds,
                reactions: reactions
                    .into_iter()
                    .map(|(emoji, user_ids)| Reaction { emoji, user_ids })
                    .collect(),
            })
        }
        messages.reverse();
        Ok(messages)
    }
}
