use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, Message, ReactionEmojiReference, User};

impl Message {
    pub async fn remove_reaction(
        &mut self,
        emoji: ReactionEmojiReference,
        user: &User,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
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
        if !reaction.user_ids.contains(&user.id) {
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
                user.id as i64
            ),
            ReactionEmojiReference::Unicode(emoji) => sqlx::query!(
                "DELETE FROM reactions WHERE unicode_emoji = $1 AND message_id = $2 AND user_id = $3",
                emoji,
                self.id as i64,
                user.id as i64
            ),
        }
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to remove reaction from database: {}", err);
            error!(SERVER, "Failed to remove reaction")
        })?;

        reaction.user_ids.retain(|i| *i != user.id);
        if reaction.user_ids.is_empty() {
            self.reactions.retain(|i| i.emoji.get_ref() != emoji);
        }
        Ok(())
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

        Ok(())
    }
}
