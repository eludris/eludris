use std::time::{Duration, SystemTime};

use argon2::{
    password_hash::{rand_core::CryptoRngCore, SaltString},
    PasswordHasher,
};
use lazy_static::lazy_static;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use rand::Rng;
use redis::AsyncCommands;
use regex::Regex;
use sqlx::{pool::PoolConnection, Postgres};

use crate::{
    ids::{IdGenerator, ELUDRIS_EPOCH},
    models::{ErrorResponse, Session, User, UserCreate},
    Conf,
};

impl UserCreate {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        lazy_static! {
            static ref USERNAME_REGEX: Regex =
                Regex::new(r"^[a-z0-9_-]+$").expect("Could not compile username regex");
            // https://stackoverflow.com/a/201378
            static ref EMAIL_REGEX: Regex = Regex::new(r#"^(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])$"#).expect("Could not compile email regex");
        };

        if !USERNAME_REGEX.is_match(&self.username) {
            Err(error!(
                VALIDATION,
                "username",
                "The user's username must only consist of lowercase letters, numbers, underscores and dashes"
            ))
        } else if self.username.len() < 2 || self.username.len() > 32 {
            Err(error!(
                VALIDATION,
                "username", "The user's username must be between 2 and 32 characters in length"
            ))
        } else if !EMAIL_REGEX.is_match(&self.email) {
            Err(error!(
                VALIDATION,
                "email", "The user's email must be valid"
            ))
        } else if self.password.len() < 8 {
            Err(error!(
                VALIDATION,
                "password", "The user's password must be be at least 8 characters long"
            ))
        } else {
            Ok(())
        }
    }
}

impl User {
    pub async fn create<H: PasswordHasher, R: CryptoRngCore, C: AsyncCommands>(
        user: UserCreate,
        hasher: &H,
        rng: &mut R,
        id_generator: &mut IdGenerator,
        conf: &Conf,
        mailer: &Option<AsyncSmtpTransport<Tokio1Executor>>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<User, ErrorResponse> {
        user.validate()?;
        if let Some(existing_user) = sqlx::query!(
            "
SELECT username, email
FROM users
WHERE username = $1
OR email = $2
            ",
            user.username,
            user.email,
        )
        .fetch_optional(&mut *db)
        .await
        .map_err(|err| {
            log::error!(
                "Failed to check if other users with the same identifiers exist: {}",
                err
            );
            error!(SERVER, "Could not create user")
        })? {
            if existing_user.username == user.username {
                return Err(error!(CONFLICT, "username"));
            } else {
                return Err(error!(CONFLICT, "password"));
            }
        }
        let id = id_generator.generate();

        if let Some(email) = &conf.email {
            let code = rng.gen_range(0..999999);
            cache
                .set::<_, _, ()>(format!("verification:{}", id), code)
                .await
                .map_err(|err| {
                    log::error!("Failed to set verification code in cache: {}", err);
                    error!(SERVER, "Could not send verification email")
                })?;
            let message = Message::builder()
                .from(
                    format!("{} <{}>", email.name, email.address)
                        .parse()
                        .map_err(|err| {
                            log::error!("Failed to build email message: {}", err);
                            error!(SERVER, "Could not send verification email")
                        })?,
                )
                .to(format!("{} <{}>", user.username, user.email)
                    .parse()
                    .map_err(|err| {
                        log::error!("Failed to build email message: {}", err);
                        error!(SERVER, "Could not send verification email")
                    })?)
                .subject("Verify your Eludris account")
                .body(format!("Your verification code is {}", code))
                .map_err(|err| {
                    log::error!("Failed to build email message: {}", err);
                    error!(SERVER, "Could not send verification email")
                })?;
            mailer
                .as_ref()
                .unwrap()
                .send(message)
                .await
                .map_err(|err| {
                    log::error!("Failed to send email: {}", err);
                    error!(SERVER, "Could not send verification email")
                })?;
        }

        let salt = SaltString::generate(rng);
        let hash = hasher
            .hash_password(user.password.as_bytes(), &salt)
            .map_err(|err| {
                log::error!("Failed to hash password: {}", err);
                error!(SERVER, "Could not hash password")
            })?
            .to_string();
        sqlx::query!(
            "
INSERT INTO users(id, username, verified, email, password)
VALUES($1, $2, $3, $4, $5)
            ",
            id as i64,
            user.username,
            conf.email.is_none(),
            user.email,
            hash
        )
        .execute(db)
        .await
        .map_err(|err| {
            log::error!("Failed to store user in database: {}", err);
            error!(SERVER, "Could not save user data")
        })?;
        Ok(User {
            id,
            username: user.username,
            display_name: None,
            social_credit: 0,
            status: None,
            bio: None,
            avatar: None,
            banner: None,
            badges: 0,
            permissions: 0,
        })
    }

    pub async fn verify<C: AsyncCommands>(
        code: u32,
        session: Session,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<(), ErrorResponse> {
        let verified = sqlx::query!(
            "
SELECT verified
FROM users
WHERE id = $1
            ",
            session.user_id as i64
        )
        .fetch_one(&mut *db)
        .await
        .map_err(|err| {
            log::error!("Could not fetch user data for verification: {}", err);
            error!(SERVER, "Couldn't verify user")
        })?
        .verified;
        if verified {
            return Err(error!(VALIDATION, "code", "User is already verified"));
        }
        let cache_code: u32 = cache
            .get(format!("verification:{}", session.user_id))
            .await
            .map_err(|err| {
                log::error!("Failed to get code from cache: {}", err);
                error!(SERVER, "Couldn't verify user")
            })?;
        if code != cache_code {
            return Err(error!(VALIDATION, "code", "Incorrect verification code"));
        }
        sqlx::query!(
            "
UPDATE users
SET verified = TRUE
WHERE id = $1
            ",
            session.user_id as i64
        )
        .execute(db)
        .await
        .map_err(|err| {
            log::error!("Failed to set user verification in database: {}", err);
            error!(SERVER, "Couldn't verify user")
        })?;
        cache
            .del::<_, ()>(format!("verification:{}", session.user_id))
            .await
            .map_err(|err| {
                log::error!("Failed to remove user code from cache: {}", err);
                error!(SERVER, "Couldn't verify user")
            })?;
        Ok(())
    }

    pub async fn clean_up_unverified(db: &mut PoolConnection<Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
DELETE FROM users
WHERE verified = FALSE
AND $1 - (id >> 16) > 604800000 -- seven days
            ",
            SystemTime::now()
                .duration_since(*ELUDRIS_EPOCH)
                .unwrap_or_else(|_| Duration::ZERO)
                .as_millis() as i64
        )
        .execute(db)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::UserCreate;

    macro_rules! test_user_create_error {
        (username: $username:expr) => {
            let user = UserCreate {
                username: $username.to_string(),
                email: "yendri@llamoyendri.io".to_string(),
                password: "autentícame por favor".to_string(),
            };
            assert!(user.validate().is_err());
        };
        (email: $email:expr) => {
            let user = UserCreate {
                username: "yendri".to_string(),
                email: $email.to_string(),
                password: "autentícame por favor".to_string(),
            };
            assert!(user.validate().is_err());
        };
        (password: $password:expr) => {
            let user = UserCreate {
                username: "yendri".to_string(),
                email: "yendri@llamoyendri.io".to_string(),
                password: $password.to_string(),
            };
            assert!(user.validate().is_err());
        };
    }

    #[test]
    fn validate_user_create() {
        let user = UserCreate {
            username: "yendri".to_string(),
            email: "yendri@llamoyendri.io".to_string(),
            password: "autentícame por favor".to_string(),
        };

        assert!(user.validate().is_ok());

        test_user_create_error!(username: "y"); // one character
        test_user_create_error!(username: "yendri_jesus_sanchez_gonzalez1988"); // too long
        test_user_create_error!(username: "yendri sanchez"); // spaces
        test_user_create_error!(username: "sánchez"); // unicode
        test_user_create_error!(username: "Yendri"); // capital letters

        test_user_create_error!(email: "no"); // invalid email

        test_user_create_error!(password: "1234"); // too short
    }
}
