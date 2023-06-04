use std::time::{Duration, SystemTime};

use argon2::{
    password_hash::{rand_core::CryptoRngCore, SaltString},
    PasswordHash, PasswordHasher, PasswordVerifier,
};
use lazy_static::lazy_static;
use rand::Rng;
use redis::AsyncCommands;
use regex::Regex;
use sqlx::{pool::PoolConnection, Postgres, QueryBuilder, Row};

use crate::{
    ids::{IdGenerator, ELUDRIS_EPOCH},
    models::{ErrorResponse, File, Session, UpdateUserProfile, User, UserCreate},
    Conf,
};

use super::{EmailPreset, Emailer};

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
        } else if !self.username.chars().any(|f| f.is_alphabetic()) {
            Err(error!(
                VALIDATION,
                "username", "The user's username must have at least one alphabetical letter"
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

impl UpdateUserProfile {
    pub async fn validate(
        &self,
        conf: &Conf,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        if self.display_name.is_none()
            && self.bio.is_none()
            && self.status.is_none()
            && self.avatar.is_none()
            && self.banner.is_none()
        {
            return Err(error!(VALIDATION, "body", "At least one field must exist"));
        }
        if let Some(Some(display_name)) = &self.display_name {
            if display_name.len() < 2 || display_name.len() > 32 {
                return Err(error!(
                    VALIDATION,
                    "display_name",
                    "The user's display name must be between 2 and 32 characters in length"
                ));
            }
        }
        if let Some(Some(bio)) = &self.bio {
            if bio.is_empty() || bio.len() > conf.oprish.bio_limit {
                return Err(error!(
                    VALIDATION,
                    "bio",
                    format!(
                        "The user's bio must be between 1 and {} characters in length",
                        conf.oprish.bio_limit
                    )
                ));
            }
        }
        if let Some(Some(status)) = &self.status {
            if status.is_empty() || status.len() > 150 {
                return Err(error!(
                    VALIDATION,
                    "status",
                    "The user's status name must be between 1 and 150 characters in length"
                ));
            }
        }
        if let Some(Some(avatar)) = self.avatar {
            if File::get(avatar, "avatars", &mut *db).await.is_none() {
                return Err(error!(
                    VALIDATION,
                    "avatar", "The user's avatar must be a valid file that must exist"
                ));
            }
        }
        if let Some(Some(banner)) = self.avatar {
            if File::get(banner, "banner", &mut *db).await.is_none() {
                return Err(error!(
                    VALIDATION,
                    "banner", "The user's banner must be a valid file that must exist"
                ));
            }
        }
        Ok(())
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
                return Err(error!(CONFLICT, "email"));
            }
        }
        let id = id_generator.generate();

        if let Some(email) = &conf.email {
            let code = rng.gen_range(100000..999999);
            cache
                .set::<_, _, ()>(format!("verification:{}", id), code)
                .await
                .map_err(|err| {
                    log::error!("Failed to set verification code in cache: {}", err);
                    error!(SERVER, "Could not send verification email")
                })?;
            mailer
                .send_email(
                    &format!("{} <{}>", user.username, user.email),
                    EmailPreset::Verify { code },
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
        .execute(db)
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
            status: None,
            bio: None,
            avatar: None,
            banner: None,
            badges: 0,
            permissions: 0,
        })
    }

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
            ",
            id as i64
        )
        .fetch_one(&mut *db)
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

    pub async fn get(id: u64, db: &mut PoolConnection<Postgres>) -> Result<Self, ErrorResponse> {
        sqlx::query!(
            "
SELECT id, username, display_name, social_credit, status, bio, avatar, banner, badges, permissions
FROM users
WHERE id = $1
            ",
            id as i64
        )
        .fetch_optional(db)
        .await
        .map_err(|err| {
            log::error!("Couldn't get user from database: {}", err);
            error!(SERVER, "Failed to get user data")
        })?
        .map(|u| Self {
            id: u.id as u64,
            username: u.username,
            display_name: u.display_name,
            social_credit: u.social_credit,
            status: u.status,
            bio: u.bio,
            avatar: u.avatar.map(|a| a as u64),
            banner: u.banner.map(|b| b as u64),
            badges: u.badges as u64,
            permissions: u.permissions as u64,
        })
        .ok_or_else(|| error!(NOT_FOUND))
    }

    pub async fn get_username(
        username: &str,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        sqlx::query!(
            "
SELECT id, username, display_name, social_credit, status, bio, avatar, banner, badges, permissions
FROM users
WHERE username = $1
            ",
            username
        )
        .fetch_optional(db)
        .await
        .map_err(|err| {
            log::error!("Couldn't get user from database: {}", err);
            error!(SERVER, "Failed to get user data")
        })?
        .map(|u| Self {
            id: u.id as u64,
            username: u.username,
            display_name: u.display_name,
            social_credit: u.social_credit,
            status: u.status,
            bio: u.bio,
            avatar: u.avatar.map(|a| a as u64),
            banner: u.banner.map(|b| b as u64),
            badges: u.badges as u64,
            permissions: u.permissions as u64,
        })
        .ok_or_else(|| error!(NOT_FOUND))
    }

    pub async fn update_profile(
        id: u64,
        profile: UpdateUserProfile,
        conf: &Conf,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        profile.validate(conf, &mut *db).await?;
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE users SET ");
        let mut seperated = query.separated(", ");
        if let Some(display_name) = profile.display_name {
            seperated
                .push("display_name = ")
                .push_bind_unseparated(display_name);
        }
        if let Some(bio) = profile.bio {
            seperated.push("bio = ").push_bind_unseparated(bio);
        }
        if let Some(status) = profile.status {
            seperated.push("status = ").push_bind_unseparated(status);
        }
        if let Some(avatar) = profile.avatar {
            seperated
                .push("avatar = ")
                .push_bind_unseparated(avatar.map(|a| a as i64));
        }
        if let Some(banner) = profile.banner {
            seperated
                .push("banner = ")
                .push_bind_unseparated(banner.map(|b| b as i64));
        }
        query
            .push(" WHERE id = ")
            .push_bind(id as i64)
            .push(
                " RETURNING id, username, display_name, social_credit, status, bio, avatar, banner, badges, permissions",
            )
            .build()
            .fetch_one(db)
            .await
            .map(|u| Self {
                id: u.get::<i64, _>("id") as u64,
                username: u.get("username"),
                display_name: u.get("display_name"),
                social_credit: u.get("social_credit"),
                status: u.get("status"),
                bio: u.get("bio"),
                avatar: u.get::<Option<i64>, _>("avatar").map(|a| a as u64),
                banner: u.get::<Option<i64>, _>("banner").map(|b| b as u64),
                badges: u.get::<i64, _>("badges") as u64,
                permissions: u.get::<i64, _>("permissions") as u64,
            })
            .map_err(|err| {
                log::error!("Couldn't update user profile: {}", err);
                error!(SERVER, "Failed to update user profile")
            })
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
