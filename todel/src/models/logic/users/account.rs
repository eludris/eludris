use std::time::{Duration, SystemTime};

use argon2::{
    password_hash::{rand_core::CryptoRngCore, SaltString},
    PasswordHasher, PasswordVerifier,
};
use lazy_static::lazy_static;
use rand::Rng;
use redis::AsyncCommands;
use regex::Regex;
use sqlx::{pool::PoolConnection, Postgres, QueryBuilder, Row};

use crate::{
    ids::{IdGenerator, ELUDRIS_EPOCH},
    models::{
        CreatePasswordResetCode, EmailPreset, Emailer, ErrorResponse, PasswordDeleteCredentials,
        ResetPassword, Session, Status, StatusType, UpdateUser, User, UserCreate,
    },
    Conf,
};

pub fn validate_username(username: &str) -> Result<(), ErrorResponse> {
    lazy_static! {
        static ref USERNAME_REGEX: Regex =
            Regex::new(r"^[a-z0-9_-]+$").expect("Could not compile username regex");
    };
    if !USERNAME_REGEX.is_match(username) {
        Err(error!(
                VALIDATION,
                "username",
                "The user's username must only consist of lowercase letters, numbers, underscores and dashes"
            ))
    } else if username.len() < 2 || username.len() > 32 {
        Err(error!(
            VALIDATION,
            "username", "The user's username must be between 2 and 32 characters in length"
        ))
    } else if !username.chars().any(|f| f.is_alphabetic()) {
        Err(error!(
            VALIDATION,
            "username", "The user's username must have at least one alphabetical letter"
        ))
    } else {
        Ok(())
    }
}

pub fn validate_email(email: &str) -> Result<(), ErrorResponse> {
    lazy_static! {
        // https://stackoverflow.com/a/201378
        static ref EMAIL_REGEX: Regex = Regex::new(r#"^(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])$"#).expect("Could not compile email regex");
    };
    if !EMAIL_REGEX.is_match(email) {
        Err(error!(
            VALIDATION,
            "email", "The user's email must be valid"
        ))
    } else {
        Ok(())
    }
}

pub fn validate_password(password: &str) -> Result<(), ErrorResponse> {
    if password.len() < 8 {
        Err(error!(
            VALIDATION,
            "password", "The user's password must be be at least 8 characters long"
        ))
    } else {
        Ok(())
    }
}

impl UserCreate {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        validate_username(&self.username)?;
        validate_email(&self.email)?;
        validate_password(&self.password)
    }
}

impl UpdateUser {
    pub async fn validate(&self, db: &mut PoolConnection<Postgres>) -> Result<(), ErrorResponse> {
        if self.username.is_none() && self.email.is_none() && self.new_password.is_none() {
            return Err(error!(VALIDATION, "body", "At least one field must exist"));
        }
        if self.username.is_some() || self.email.is_some() {
            let mut query: QueryBuilder<Postgres> =
                QueryBuilder::new("SELECT username, email FROM users WHERE (");
            let mut seperated = query.separated(" OR ");
            if let Some(username) = &self.username {
                validate_username(username)?;
                seperated
                    .push("username = ")
                    .push_bind_unseparated(username);
            }
            if let Some(email) = &self.email {
                validate_email(email)?;
                seperated.push("email = ").push_bind_unseparated(email);
            }
            if let Some(row) = query
                .push(") AND is_deleted = FALSE")
                .build()
                .fetch_optional(&mut **db)
                .await
                .map_err(|err| {
                    log::error!("Couldn't fetch users from database: {}", err);
                    error!(SERVER, "Failed to validate payload")
                })?
            {
                if row.get::<Option<String>, _>("username") == self.username {
                    return Err(error!(CONFLICT, "username"));
                } else {
                    return Err(error!(CONFLICT, "email"));
                }
            };
        }
        if let Some(password) = &self.new_password {
            validate_password(password)?;
        }
        Ok(())
    }
}

impl ResetPassword {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        validate_email(&self.email)?;
        validate_password(&self.password)
    }
}

impl User {
    pub async fn create<H: PasswordHasher, R: CryptoRngCore, C: AsyncCommands>(
        user: UserCreate,
        hasher: &H,
        rng: &mut R,
        id_generator: &mut IdGenerator,
        conf: &Conf,
        mailer: &Emailer,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        user.validate()?;
        if let Some(existing_user) = sqlx::query!(
            "
SELECT username, email, is_deleted
FROM users
WHERE username = $1
OR email = $2
            ",
            user.username,
            user.email,
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!(
                "Failed to check if other users with the same identifiers exist: {}",
                err
            );
            error!(SERVER, "Could not create user")
        })? {
            if existing_user.is_deleted {
                sqlx::query!(
                    "
DELETE FROM users
WHERE username = $1
OR email= $2
                    ",
                    user.username,
                    user.email
                )
                .execute(&mut **db)
                .await
                .map_err(|err| {
                    log::error!("Failed to clean up pre-existing deleted user: {}", err);
                    error!(SERVER, "Could not create user")
                })?;
            } else if existing_user.username == user.username {
                return Err(error!(CONFLICT, "username"));
            } else {
                return Err(error!(CONFLICT, "email"));
            }
        }
        let id = id_generator.generate();

        if let Some(email) = &conf.email {
            let code = rng.gen_range(100000..999999);
            cache
                .set_ex::<_, _, ()>(format!("verification:{}", id), code, 604_800_000)
                .await
                .map_err(|err| {
                    log::error!("Failed to set verification code in cache: {}", err);
                    error!(SERVER, "Could not send verification email")
                })?;
            mailer
                .send_email(
                    &format!("{} <{}>", user.username, user.email),
                    EmailPreset::Verify {
                        username: &user.username,
                        code,
                    },
                    email,
                )
                .await?;
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
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to store user in database: {}", err);
            error!(SERVER, "Could not save user data")
        })?;
        Ok(Self {
            id,
            username: user.username,
            display_name: None,
            social_credit: 0,
            status: Status {
                status_type: StatusType::Offline,
                text: None,
            },
            bio: None,
            avatar: None,
            banner: None,
            badges: 0,
            permissions: 0,
            email: Some(user.email),
            verified: Some(conf.email.is_none()),
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
AND is_deleted = FALSE
            ",
            session.user_id as i64
        )
        .fetch_one(&mut **db)
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
        .execute(&mut **db)
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

    pub async fn resend_verification<C: AsyncCommands>(
        session: Session,
        mailer: &Emailer,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
        conf: &Conf,
    ) -> Result<(), ErrorResponse> {
        let verified = sqlx::query!(
            "
SELECT verified
FROM users
WHERE id = $1
AND is_deleted = FALSE
            ",
            session.user_id as i64
        )
        .fetch_one(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Could not fetch user data for verification: {}", err);
            error!(SERVER, "Couldn't check user verification status")
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
                error!(SERVER, "Couldn't check user verification status")
            })?;
        let user = sqlx::query!(
            "
SELECT username, email
FROM users
WHERE id = $1
            ",
            session.user_id as i64
        )
        .fetch_one(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch user email: {}", err);
            error!(SERVER, "Couldn't check user verification status")
        })?;

        if let Some(email) = &conf.email {
            mailer
                .send_email(
                    &format!("{} <{}>", user.username, user.email),
                    EmailPreset::Verify {
                        username: &user.username,
                        code: cache_code,
                    },
                    email,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn update<H: PasswordHasher, R: CryptoRngCore>(
        id: u64,
        update: UpdateUser,
        mailer: &Emailer,
        conf: &Conf,
        hasher: &H,
        rng: &mut R,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        update.validate(&mut *db).await?;
        Self::validate_password(id, &update.password, hasher, db).await?;
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE users SET ");
        let mut seperated = query.separated(", ");
        let username = Self::get_unfiltered(id, &mut *db).await?.username;
        if let Some(username) = &update.username {
            seperated
                .push("username = ")
                .push_bind_unseparated(username);
        }
        if let Some(email) = &update.email {
            seperated.push("email = ").push_bind_unseparated(email);
        }
        if let Some(new_password) = &update.new_password {
            let salt = SaltString::generate(rng);
            let hash = hasher
                .hash_password(new_password.as_bytes(), &salt)
                .map_err(|err| {
                    log::error!("Failed to hash password: {}", err);
                    error!(SERVER, "Could not hash password")
                })?
                .to_string();
            seperated.push("password = ").push_bind_unseparated(hash);
        }
        let user: Self = query
            .push(" WHERE id = ")
            .push_bind(id as i64)
            .push(
                " RETURNING id, username, display_name, social_credit, status, status_type, bio, avatar, banner, badges, permissions, email, verified",
            )
            .build_query_as()
            .fetch_one(&mut **db)
            .await
            .map_err(|err| {
                log::error!("Couldn't update user profile: {}", err);
                error!(SERVER, "Failed to update user profile")
            })?;
        if let Some(email) = &conf.email {
            mailer
                .send_email(
                    &format!(
                        "{} <{}>",
                        user.username,
                        user.email.as_ref().expect("Couldn't get user email")
                    ),
                    EmailPreset::UserUpdated {
                        old_username: update.username.as_ref().map(|_| username.as_str()),
                        username: update.username.as_deref().unwrap_or_else(|| &username),
                        new_email: update.email.as_deref(),
                        password: update.new_password.is_some(),
                    },
                    email,
                )
                .await?;
        }
        Ok(user)
    }

    pub async fn create_password_reset_code<R: CryptoRngCore, C: AsyncCommands>(
        create_code: CreatePasswordResetCode,
        rng: &mut R,
        conf: &Conf,
        mailer: &Emailer,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<(), ErrorResponse> {
        validate_email(&create_code.email)?;
        if let Some(email) = &conf.email {
            let username = sqlx::query!(
                "
SELECT username
FROM users
WHERE email = $1
                ",
                create_code.email,
            )
            .fetch_optional(&mut **db)
            .await
            .map_err(|err| {
                log::error!("Failed to fetch user data: {}", err);
                error!(SERVER, "Couldn't fetch user data")
            })?
            .ok_or_else(|| error!(NOT_FOUND))?
            .username;
            let code = rng.gen_range(100000..999999);
            cache
                .set_ex::<_, _, ()>(format!("password-reset:{}", create_code.email), code, 86400)
                .await
                .map_err(|err| {
                    log::error!("Failed to set verification code in cache: {}", err);
                    error!(SERVER, "Could not send verification email")
                })?;
            mailer
                .send_email(
                    &format!("{} <{}>", username, create_code.email),
                    EmailPreset::PasswordReset {
                        username: &username,
                        code,
                    },
                    email,
                )
                .await?;
            Ok(())
        } else {
            Err(error!(
                MISDIRECTED,
                "This instance doesn't have a configured email"
            ))
        }
    }

    pub async fn reset_password<H: PasswordHasher, R: CryptoRngCore, C: AsyncCommands>(
        reset: ResetPassword,
        hasher: &H,
        rng: &mut R,
        mailer: &Emailer,
        conf: &Conf,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<(), ErrorResponse> {
        reset.validate()?;
        let cache_code: u32 = cache
            .get(format!("password-reset:{}", reset.email))
            .await
            .map_err(|err| {
                log::error!("Failed to get code from cache: {}", err);
                error!(SERVER, "Couldn't reset the user's password")
            })?;
        if reset.code != cache_code {
            return Err(error!(VALIDATION, "code", "Incorrect password reset code"));
        }
        let salt = SaltString::generate(rng);
        let hash = hasher
            .hash_password(reset.password.as_bytes(), &salt)
            .map_err(|err| {
                log::error!("Failed to hash password: {}", err);
                error!(SERVER, "Could not hash password")
            })?
            .to_string();
        let user = sqlx::query!(
            "
UPDATE users
SET password = $1
WHERE email = $2
RETURNING username, email
            ",
            hash,
            reset.email
        )
        .fetch_one(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to set user password hash in database: {}", err);
            error!(SERVER, "Couldn't reset the user's pasword")
        })?;
        cache
            .del::<_, ()>(format!("password-reset:{}", reset.email))
            .await
            .map_err(|err| {
                log::error!(
                    "Failed to remove user password reset code from cache: {}",
                    err
                );
                error!(SERVER, "Couldn't reset the user's  password")
            })?;
        if let Some(email) = &conf.email {
            mailer
                .send_email(
                    &format!("{} <{}>", user.username, user.email),
                    EmailPreset::UserUpdated {
                        username: &user.username,
                        old_username: None,
                        new_email: None,
                        password: true,
                    },
                    email,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn delete<V: PasswordVerifier>(
        id: u64,
        delete: PasswordDeleteCredentials,
        verifier: &V,
        mailer: &Emailer,
        conf: &Conf,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        Self::validate_password(id, &delete.password, verifier, db).await?;
        let user = sqlx::query!(
            "
UPDATE users
SET is_deleted = TRUE
WHERE id = $1
RETURNING username, email
            ",
            id as i64
        )
        .fetch_one(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't mark user as deleted: {}", err);
            error!(SERVER, "Failed to delete user")
        })?;
        if let Some(email) = &conf.email {
            mailer
                .send_email(
                    &format!("{} <{}>", user.username, user.email),
                    EmailPreset::Delete {
                        username: &user.username,
                    },
                    email,
                )
                .await?;
        }
        Ok(())
    }
}

impl User {
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
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn clean_up_deleted(db: &mut PoolConnection<Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
DELETE FROM users
WHERE is_deleted = TRUE
            ",
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }
}
