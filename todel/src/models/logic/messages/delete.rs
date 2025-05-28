use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, Message};

impl Message {
    pub async fn delete(&self, db: &mut PoolConnection<Postgres>) -> Result<(), ErrorResponse> {
        sqlx::query!(
            "
            DELETE FROM messages
            WHERE id = $1
            AND channel_id = $2
            ",
            self.id as i64,
            self.channel.get_id() as i64
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to delete message {}: {}", self.id, err);
            error!(SERVER, "Failed to delete message")
        })?;
        Ok(())
    }
}
