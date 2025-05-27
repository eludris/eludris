mod get;

use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, Postgres};

use crate::{
    ids::IdGenerator,
    models::{Embed, ErrorResponse, File, Message, MessageCreate, SphereChannel, User},
    Conf,
};

impl MessageCreate {
    pub async fn validate(&mut self, conf: &Conf) -> Result<(), ErrorResponse> {
        if let Some(ref mut content) = self.content {
            *content = content.trim().to_string();
            if content.is_empty() {
                self.content = None;
            } else if content.len() > conf.oprish.message_limit {
                return Err(error!(
                    VALIDATION,
                    "content",
                    format!(
                        "Message content has to be less than {} characters long",
                        conf.oprish.message_limit
                    )
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
        conf: &Conf,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        message.validate(conf).await?;
        let channel = SphereChannel::get(channel_id, db).await.map_err(|err| {
            if let ErrorResponse::NotFound { .. } = err {
                error!(VALIDATION, "channel", "Channel doesn't exist")
            } else {
                err
            }
        })?;
        let id = id_generator.generate();
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
        .execute(&mut **db)
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
        let mut attachments = vec![];
        for attachment_id in message.attachments {
            attachments.push(File::fetch_file_data(attachment_id, "attachments", db).await?);
        }
        Ok(Self {
            id,
            author: User::get(author_id, None, db, cache).await?,
            content: message.content,
            reference,
            disguise: message.disguise,
            channel,
            attachments: vec![],
            embeds: message
                .embeds
                .into_iter()
                .map(|e| Embed::Custom(e))
                .collect(),
        })
    }
}
