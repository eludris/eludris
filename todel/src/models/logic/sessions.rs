use std::net::IpAddr;

use argon2::{PasswordHash, PasswordVerifier};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, types::ipnetwork::IpNetwork, Postgres};

use crate::{
    ids::IdGenerator,
    models::{ErrorResponse, Session, SessionCreate, SessionCreated},
};

use super::Secret;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionTokenClaims {
    user_id: u64,
    session_id: u64,
}

impl SessionCreate {
    pub fn ensure_valid(&mut self) {
        self.platform = self.platform.to_lowercase();
        self.client = self.client.to_lowercase();
    }
}

impl Session {
    pub async fn create<V: PasswordVerifier>(
        mut session: SessionCreate,
        ip: IpAddr,
        secret: &Secret,
        verifier: &V,
        id_generator: &mut IdGenerator,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<SessionCreated, ErrorResponse> {
        session.ensure_valid();
        let user = sqlx::query!(
            "
SELECT id, password
FROM users
WHERE username = $1
OR email = $1
            ",
            session.identifier
        )
        .fetch_optional(&mut *db)
        .await
        .map_err(|err| {
            log::error!("Could not fetch the user's password: {}", err);
            error!(SERVER, "Failed to fetch the user's password")
        })?
        .ok_or_else(|| error!(NOT_FOUND))?;
        verifier
            .verify_password(
                session.password.as_bytes(),
                &PasswordHash::new(&user.password).map_err(|err| {
                    log::error!("Couldn't parse password hash: {}", err);
                    error!(SERVER, "Failed to validate the user's password")
                })?,
            )
            .map_err(|_| error!(UNAUTHORIZED))?;
        let id = id_generator.generate();
        sqlx::query!(
            "
INSERT INTO sessions(id, user_id, platform, client, ip)
VALUES($1, $2, $3, $4, $5)
            ",
            id as i64,
            user.id as i64,
            session.platform,
            session.client,
            IpNetwork::from(ip),
        )
        .execute(db)
        .await
        .map_err(|err| {
            log::error!("Failed to store session in database: {}", err);
            error!(SERVER, "Could not save session data")
        })?;
        let claims = SessionTokenClaims {
            user_id: user.id as u64,
            session_id: id,
        };
        let token = claims.sign_with_key(&secret.0).map_err(|err| {
            log::error!("Couldn't sign JWT: {}", err);
            error!(SERVER, "Failed to generate a token for the user")
        })?;
        Ok(SessionCreated {
            token,
            session: Session {
                id,
                user_id: user.id as u64,
                platform: session.platform,
                client: session.client,
                ip,
            },
        })
    }

    pub async fn validate_token(
        token: &str,
        secret: &Secret,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Session, ErrorResponse> {
        let claims: SessionTokenClaims = token
            .verify_with_key(&secret.0)
            .map_err(|_| error!(UNAUTHORIZED))?;
        let session = sqlx::query!(
            "
SELECT *
FROM sessions
WHERE id = $1
AND user_id = $2
            ",
            claims.session_id as i64,
            claims.user_id as i64
        )
        .fetch_optional(db)
        .await
        .map_err(|err| {
            log::error!("Could not fetch the user's session: {}", err);
            error!(SERVER, "Failed to fetch the user's session")
        })?
        .map(|s| Session {
            id: s.id as u64,
            user_id: s.user_id as u64,
            platform: s.platform,
            client: s.client,
            ip: s.ip.ip(),
        })
        .ok_or_else(|| error!(UNAUTHORIZED))?; // no such session exists
        Ok(session)
    }
}
