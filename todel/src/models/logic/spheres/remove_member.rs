use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, Sphere};

impl Sphere {
    pub async fn remove_member(
        &self,
        user_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        if !self.has_member(user_id, db).await? {
            return Err(error!(VALIDATION, "sphere", "User isn't in this sphere"));
        }
        sqlx::query!(
            "
            DELETE FROM members
            WHERE id = $1
            AND sphere_id = $2
            ",
            user_id as i64,
            self.id as i64
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't remove member from sphere {}: {}", self.id, err);
            error!(SERVER, "Failed to remove member from sphere")
        })?;
        Ok(())
    }
}
