use argon2::{PasswordHash, PasswordVerifier};
use sqlx::{pool::PoolConnection, Postgres};

use crate::models::{ErrorResponse, User};

impl User {
    pub async fn validate_password<V: PasswordVerifier>(
        id: u64,
        password: &str,
        verifier: &V,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        let hash = sqlx::query!(
            "
SELECT password
FROM users
WHERE id = $1
AND is_deleted = FALSE
            ",
            id as i64
        )
        .fetch_one(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Could not fetch the user's password: {}", err);
            error!(SERVER, "Failed to fetch the user's password")
        })?
        .password;
        verifier
            .verify_password(
                password.as_bytes(),
                &PasswordHash::new(&hash).map_err(|err| {
                    log::error!("Couldn't parse password hash: {}", err);
                    error!(SERVER, "Failed to validate the user's password")
                })?,
            )
            .map_err(|_| error!(UNAUTHORIZED))
    }
}
