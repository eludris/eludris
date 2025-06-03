use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{Emoji, EmojiEdit, ErrorResponse};

impl EmojiEdit {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        if self.name.len() < 2 || self.name.len() > 32 {
            return Err(error!(
                VALIDATION,
                "name", "The emoji's name must be between 2 and 32 characters in length"
            ));
        }
        Ok(())
    }
}

impl Emoji {
    pub async fn edit(
        &mut self,
        edit: EmojiEdit,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        sqlx::query!(
            "
            UPDATE emojis
            SET name = $1
            WHERE id = $2
            ",
            edit.name,
            self.id as i64
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to edit emoji {}: {}", self.id, err);
            error!(SERVER, "Failed to edit emoji")
        })?;
        Ok(())
    }
}
