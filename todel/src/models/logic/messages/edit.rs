use sqlx::{pool::PoolConnection, Acquire, Postgres};

use crate::models::{Attachment, Embed, ErrorResponse, File, Message, MessageEdit};

impl MessageEdit {
    pub fn validate(&mut self) -> Result<(), ErrorResponse> {
        if let Some(Some(ref mut content)) = self.content {
            *content = content.trim().to_string();
            if content.is_empty() {
                self.content = Some(None);
            } else if content.len() > 4096 {
                return Err(error!(
                    VALIDATION,
                    "content", "Message content has to be less than 4096 characters long"
                ));
            }
        }
        if self.content.is_none() && self.attachments.is_none() && self.embeds.is_none() {
            return Err(error!(
                VALIDATION,
                "body", "Message must contain at least either content, an attachment or an embed"
            ));
        }
        if let Some(attachments) = &self.attachments {
            if attachments.len() > 10 {
                return Err(error!(
                    VALIDATION,
                    "attachments", "Message can't contain more than 10 attachments"
                ));
            }
        }
        if let Some(embeds) = &self.embeds {
            if embeds.len() > 10 {
                return Err(error!(
                    VALIDATION,
                    "embeds", "Message can't contain more than 10 embeds"
                ));
            }
            for (i, embed) in embeds.iter().enumerate() {
                if embed.content.len() > 8192 {
                    return Err(error!(
                        VALIDATION,
                        format!("embed-{}.content", i),
                        "The embed's content can't be over 8196 characters long"
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Message {
    pub async fn edit(
        &mut self,
        mut edit: MessageEdit,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        edit.validate()?;

        if (edit.content.as_ref().is_some_and(|c| c.is_none()) || self.content.is_none())
            && (edit.attachments.as_ref().is_some_and(|a| a.is_empty())
                || self.attachments.is_empty())
            && (edit.embeds.as_ref().is_some_and(|e| e.is_empty()) || self.embeds.is_empty())
        {
            return Err(error!(
                VALIDATION,
                "body", "Final message must contain either content, an attachment or an embed"
            ));
        }

        // store here to avoid relying on side effects later
        let had_attachments = edit.attachments.is_some();

        let mut attachments = vec![];
        if let Some(attachment_creates) = edit.attachments {
            for (i, attachment_create) in attachment_creates.into_iter().enumerate() {
                let file = match File::get(attachment_create.file_id, "attachments", db).await {
                    Some(file) => file,
                    None => {
                        return Err(error!(
                            VALIDATION,
                            format!("attachments-{}", i),
                            "Attachment's file must be a valid file that exists in the attachments bucket"
                        ))
                    }
                };
                attachments.push(Attachment {
                    file: file.get_file_data(),
                    description: attachment_create.description,
                    spoiler: attachment_create.spoiler,
                });
            }
        }

        let mut transaction = db.begin().await.map_err(|err| {
            log::error!(
                "Couldn't start message edit transaction {}: {}",
                self.id,
                err
            );
            error!(SERVER, "Failed to edit message")
        })?;

        if let Some(content) = edit.content {
            sqlx::query!(
                "
                UPDATE messages
                SET content = $1
                WHERE id = $2
                ",
                content,
                self.id as i64,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!("Couldn't edit message content {}: {}", self.id, err);
                error!(SERVER, "Failed to edit message")
            })?;
            self.content = content;
        }

        if had_attachments {
            sqlx::query!(
                "
                DELETE FROM message_attachments
                WHERE message_id = $1
                ",
                self.id as i64,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!(
                    "Couldn't remove old message attachments {}: {}",
                    self.id,
                    err
                );
                error!(SERVER, "Failed to edit message")
            })?;
            for attachment in attachments.iter() {
                sqlx::query!(
                    "
                    INSERT INTO message_attachments(message_id, file_id, description, spoiler)
                    VALUES($1, $2, $3, $4)
                    ",
                    self.id as i64,
                    attachment.file.id as i64,
                    attachment.description,
                    attachment.spoiler
                )
                .execute(&mut *transaction)
                .await
                .map_err(|err| {
                    log::error!(
                        "Couldn't edit message attachment file {} to {}: {}",
                        attachment.file.id,
                        self.id,
                        err
                    );
                    error!(SERVER, "Failed to edit message")
                })?;
            }
            self.attachments = attachments;
        }

        if let Some(embeds) = edit.embeds {
            sqlx::query!(
                "
                DELETE FROM message_embeds
                WHERE message_id = $1
                ",
                self.id as i64,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| {
                log::error!("Couldn't remove old message embeds {}: {}", self.id, err);
                error!(SERVER, "Failed to edit message")
            })?;
            for embed in embeds.iter() {
                sqlx::query!(
                    "
                INSERT INTO message_embeds(message_id, embed)
                VALUES($1, $2)
                ",
                    self.id as i64,
                    serde_json::to_value(Embed::Custom(embed.clone())).unwrap(),
                )
                .execute(&mut *transaction)
                .await
                .map_err(|err| {
                    log::error!(
                        "Couldn't edit message embed {:?} to {}: {}",
                        embed,
                        self.id,
                        err
                    );
                    error!(SERVER, "Failed to edit message")
                })?;
            }
            self.embeds = embeds.into_iter().map(Embed::Custom).collect();
        }

        transaction.commit().await.map_err(|err| {
            log::error!(
                "Couldn't commit message edit transaction {}: {}",
                self.id,
                err
            );
            error!(SERVER, "Failed to edit message")
        })?;

        Ok(())
    }
}
