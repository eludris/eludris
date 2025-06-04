use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{Emoji, ErrorResponse};

impl Emoji {
    pub async fn delete(self, db: &mut PoolConnection<Postgres>) -> Result<(), ErrorResponse> {
        sqlx::query!(
            "
            UPDATE emojis
            SET is_deleted = TRUE
            WHERE id = $1
            ",
            self.id as i64
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!(
                "Failed to get delete emoji from database {}: {}",
                self.id,
                err
            );
            error!(SERVER, "Failed to delete emoji")
        })?;
        Ok(())
    }

    pub async fn clean_up_deleted(db: &mut PoolConnection<Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
DELETE FROM emojis
WHERE is_deleted = TRUE
            ",
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }
}
