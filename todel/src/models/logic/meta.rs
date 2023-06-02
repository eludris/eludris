use hmac::{Hmac, Mac};
use rand::{CryptoRng, Rng};
use sha2::Sha256;
use sqlx::{Pool, Postgres};

// This struct is mainly used to keep all of the sqlx logic in todel, thus dramatically simplifying
// the handling of offline-mode sqlx builds.
//
// Wrapping a type also makes it easier to use it as a rocket guard.
#[derive(Debug)]
pub struct Secret(pub Hmac<Sha256>);

impl Secret {
    pub async fn get<R: Rng + CryptoRng>(db: &Pool<Postgres>, rng: &mut R) -> Result<Secret, ()> {
        let secret = match sqlx::query!("SELECT secret FROM meta")
            .fetch_optional(db)
            .await
        {
            Ok(Some(record)) => record.secret,
            Ok(None) => {
                let secret: Vec<u8> = (0..128).map(|_| rng.gen()).collect();

                if let Err(err) = sqlx::query!(
                    "
INSERT INTO meta(secret)
VALUES($1)
                    ",
                    secret
                )
                .execute(db)
                .await
                {
                    log::error!("Could not insert instance meta info into database: {err}");
                    return Err(());
                }
                secret
            }
            Err(err) => {
                log::error!("Could not fetch instance meta info: {}", err);
                return Err(());
            }
        };

        match Hmac::new_from_slice(&secret) {
            Ok(secret) => Ok(Secret(secret)),
            Err(err) => {
                log::error!("Could not generate an HMAC from the secret: {}", err);
                Err(())
            }
        }
    }

    pub async fn try_get(db: &Pool<Postgres>) -> Option<Self> {
        Some(Self(
            Hmac::new_from_slice(
                &sqlx::query!("SELECT secret FROM meta")
                    .fetch_optional(db)
                    .await
                    .ok()??
                    .secret,
            )
            .ok()?,
        ))
    }
}
