use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{Emoji, ErrorResponse, Message, ReactionEmoji, ReactionEmojiReference};

impl Message {
    pub async fn remove_reaction(
        &mut self,
        emoji: ReactionEmojiReference,
        user_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<ReactionEmoji, ErrorResponse> {
        let reaction = match self
            .reactions
            .iter_mut()
            .find(|e| e.emoji.get_ref() == emoji)
        {
            Some(reaction) => reaction,
            None => {
                return Err(error!(
                    VALIDATION,
                    "user", "User isn't reacted with this emoji to this message"
                ));
            }
        };
        if !reaction.user_ids.contains(&user_id) {
            return Err(error!(
                VALIDATION,
                "user", "User isn't reacted with this emoji to this message"
            ));
        }

        match &emoji {
            ReactionEmojiReference::Custom(emoji) => sqlx::query!(
                "DELETE FROM reactions WHERE emoji_id = $1 AND message_id = $2 AND user_id = $3",
                *emoji as i64,
                self.id as i64,
                user_id as i64
            ),
            ReactionEmojiReference::Unicode(emoji) => sqlx::query!(
                "DELETE FROM reactions WHERE unicode_emoji = $1 AND message_id = $2 AND user_id = $3",
                emoji,
                self.id as i64,
                user_id as i64
            ),
        }
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to remove reaction from database: {}", err);
            error!(SERVER, "Failed to remove reaction")
        })?;

        reaction.user_ids.retain(|i| *i != user_id);
        if reaction.user_ids.is_empty() {
            self.reactions.retain(|i| i.emoji.get_ref() != emoji);
        }

        let full_emoji = match emoji {
            ReactionEmojiReference::Custom(id) => ReactionEmoji::Custom(Emoji::get(id, db).await?),
            ReactionEmojiReference::Unicode(emoji) => ReactionEmoji::Unicode(emoji), // yikes
        };

        Ok(full_emoji)
    }

    pub async fn clear_reactions(
        &mut self,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        if self.reactions.is_empty() {
            return Err(error!(
                VALIDATION,
                "reactions", "Message has no reactions to clear"
            ));
        }

        sqlx::query!(
            "
            DELETE FROM reactions
            WHERE message_id = $1
            ",
            self.id as i64
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to clear reactions for {}: {}", self.id, err);
            error!(SERVER, "Failed to clear reaction")
        })?;

        self.reactions = vec![];

        Ok(())
    }
}
