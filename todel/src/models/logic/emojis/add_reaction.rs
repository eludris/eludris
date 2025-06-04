use std::{collections::HashMap, fs};

use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{
    Emoji, ErrorResponse, Message, Reaction, ReactionEmoji, ReactionEmojiReference, User,
};

impl ReactionEmojiReference {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        if let Self::Unicode(emoji) = self {
            validate_unicode_emoji(emoji)?;
        }
        Ok(())
    }
}

pub fn validate_unicode_emoji(emoji: &str) -> Result<(), ErrorResponse> {
    // blocking but whatever, it's run only once anyway
    lazy_static! {
        static ref UNICODE_MAP: HashMap<String, String> = serde_json::from_str(
            &fs::read_to_string("static/emojis.json").expect("Failed to read emoji json")
        )
        .expect("Failed to parse emoji json");
    };
    if !UNICODE_MAP.contains_key(emoji) {
        return Err(error!(
            VALIDATION,
            "emoji", "Invalid unicode emoji. Only emojis in emojis.json are supported."
        ));
    }
    Ok(())
}

impl Message {
    pub async fn add_reaction(
        &mut self,
        emoji: ReactionEmojiReference,
        user: &User,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        let reaction = self
            .reactions
            .iter_mut()
            .find(|e| e.emoji.get_ref() == emoji); // yikes
        if let Some(reaction) = &reaction {
            if reaction.user_ids.contains(&user.id) {
                return Err(error!(
                    VALIDATION,
                    "user", "User already reacted with this emoji to this message"
                ));
            }
        }

        let full_emoji = match &emoji {
            ReactionEmojiReference::Custom(id) => ReactionEmoji::Custom(Emoji::get(*id, db).await?),
            ReactionEmojiReference::Unicode(emoji) => ReactionEmoji::Unicode(emoji.clone()), // yikes
        };

        match emoji {
            ReactionEmojiReference::Custom(emoji) => sqlx::query!(
                "INSERT INTO reactions(emoji_id, message_id, user_id) VALUES($1, $2, $3)",
                emoji as i64,
                self.id as i64,
                user.id as i64
            ),
            ReactionEmojiReference::Unicode(emoji) => sqlx::query!(
                "INSERT INTO reactions(unicode_emoji, message_id, user_id) VALUES($1, $2, $3)",
                emoji,
                self.id as i64,
                user.id as i64
            ),
        }
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to insert reaction into database: {}", err);
            error!(SERVER, "Failed to add reaction")
        })?;

        match reaction {
            Some(reaction) => reaction.user_ids.push(user.id),
            None => self.reactions.push({
                Reaction {
                    emoji: full_emoji,
                    user_ids: vec![self.id],
                }
            }),
        }
        Ok(())
    }
}
