mod delete;
mod edit;
mod get;
#[cfg(feature = "http")]
mod populate_embeds;

use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, Acquire, Postgres};

use crate::{
    ids::IdGenerator,
    models::{Attachment, Embed, ErrorResponse, File, Message, MessageCreate, SphereChannel, User},
};

impl MessageCreate {
    pub fn validate(&mut self) -> Result<(), ErrorResponse> {
        if let Some(ref mut content) = self.content {
            *content = content.trim().to_string();
            if content.is_empty() {
                self.content = None;
            } else if content.len() > 4096 {
                return Err(error!(
                    VALIDATION,
                    "content", "Message content has to be less than 4096 characters long"
                ));
            }
        }
        if self.content.is_none() && self.attachments.is_empty() && self.embeds.is_empty() {
            return Err(error!(
                VALIDATION,
                "body", "Message must contain at least either content, an attachment or an embed"
            ));
        }
        if self.attachments.len() > 10 {
            return Err(error!(
                VALIDATION,
                "attachments", "Message can't contain more than 10 attachments"
            ));
        }
        if self.embeds.len() > 10 {
            return Err(error!(
                VALIDATION,
                "embeds", "Message can't contain more than 10 embeds"
            ));
        }
        for (i, embed) in self.embeds.iter().enumerate() {
            if embed.content.len() > 8192 {
                return Err(error!(
                    VALIDATION,
                    format!("embed-{}.content", i),
                    "The embed's content can't be over 8196 characters long"
                ));
            }
        }
        if let Some(disguise) = &self.disguise {
            if let Some(name) = &disguise.name {
                if name.len() < 2 || name.len() > 32 {
                    return Err(error!(
                        VALIDATION,
                        "disguise.name",
                        "The user's disguise name must be between 2 and 32 characters in length"
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Message {
    pub async fn create<C: AsyncCommands>(
        mut message: MessageCreate,
        channel_id: u64,
        author_id: u64,
        id_generator: &mut IdGenerator,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        message.validate()?;
        let channel = SphereChannel::get(channel_id, db).await.map_err(|err| {
            if let ErrorResponse::NotFound { .. } = err {
                error!(VALIDATION, "channel", "Channel doesn't exist")
            } else {
                err
            }
        })?;
        let id = id_generator.generate();
        let reference = match message.reference {
            Some(reference) => match Self::get(reference, db, cache).await {
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
        let author = User::get(author_id, None, db, cache).await?;

        // gather attachment files pre-transaction
        // (consider if this should be in Attachment::create)
        let mut attachment_files = vec![];
        for (i, attachment_create) in message.attachments.into_iter().enumerate() {
            attachment_create.validate()?;

            let attachment_file =
                match File::get(attachment_create.file_id, "attachments", db).await {
                    Some(file) => file,
                    None => {
                        return Err(error!(
                            VALIDATION,
                            format!("attachments-{}", i),
                            "File doesn't exist"
                        ))
                    }
                };
            attachment_files.push(attachment_file.get_file_data());
        }

        let mut transaction = db.begin().await.map_err(|err| {
            log::error!("Couldn't start message create transaction: {}", err);
            error!(SERVER, "Failed to create message")
        })?;
        sqlx::query!(
            "
INSERT INTO messages(id, channel_id, author_id, content, reference)
VALUES($1, $2, $3, $4, $5)
            ",
            id as i64,
            channel_id as i64,
            author_id as i64,
            message.content,
            message.reference.map(|r| r as i64),
        )
        .execute(&mut *transaction)
        .await
        .map_err(|err| {
            log::error!(
                "Couldn't create message by {} on {}: {}",
                author_id,
                channel_id,
                err
            );
            error!(SERVER, "Failed to create message")
        })?;

        let mut attachments = vec![];
        for (i, attachment_create) in message.attachments.into_iter().enumerate() {
            sqlx::query!(
                "
                INSERT INTO message_attachments(message_id, file_id, description, spoiler)
                VALUES($1, $2, $3, $4)
                ",
                id as i64,
                attachment_create.file_id as i64,
                attachment_create.description as String,
                attachment_create.spoiler as bool
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't add message attachment for file_id {} to {}: {}",
                    attachment_create.file_id,
                    id,
                    err
                );
                error!(SERVER, "Failed to create message attachment")
            })?;

            attachments.push(Attachment {
                file: attachment_files[i],
                description: attachment_create.description,
                spoiler: attachment_create.spoiler,
            });
        }
        for embed in message.embeds.iter() {
            sqlx::query!(
                "
                INSERT INTO message_embeds(message_id, embed)
                VALUES($1, $2)
                ",
                id as i64,
                serde_json::to_value(Embed::Custom(embed.clone())).unwrap(),
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!("Couldn't add message embed {:?} to {}: {}", embed, id, err);
                error!(SERVER, "Failed to create message")
            })?;
        }

        transaction.commit().await.map_err(|err| {
            log::error!("Couldn't commit message create transaction: {}", err);
            error!(SERVER, "Failed to create message")
        })?;

        Ok(Self {
            id,
            author,
            content: message.content,
            reference,
            disguise: message.disguise,
            channel,
            attachments,
            embeds: message.embeds.into_iter().map(Embed::Custom).collect(),
            reactions: vec![],
        })
    }
}
