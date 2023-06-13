use std::time::{Duration, SystemTime};

use argon2::{
    password_hash::{rand_core::CryptoRngCore, SaltString},
    PasswordHash, PasswordHasher, PasswordVerifier,
};
use lazy_static::lazy_static;
use rand::Rng;
use redis::AsyncCommands;
use regex::Regex;
use sqlx::{pool::PoolConnection, Database, Decode, Postgres, QueryBuilder, Row};

use crate::{
    ids::{IdGenerator, ELUDRIS_EPOCH},
    models::{
        ErrorResponse, File, PasswordDeleteCredentials, Session, Status, StatusType, UpdateUser,
        UpdateUserProfile, User, UserCreate,
    },
    Conf,
};

use super::{EmailPreset, Emailer};

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

impl UpdateUserProfile {
    pub async fn validate(
        &self,
        conf: &Conf,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        if self.display_name.is_none()
            && self.bio.is_none()
            && self.status.is_none()
            && self.status_type.is_none()
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
                .fetch_optional(db)
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

impl<'r, DB: Database> Decode<'r, DB> for Status
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(serde_json::from_str(<&str as Decode<DB>>::decode(value)?)
            .expect("Couldn't deserialize status type"))
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
        .fetch_optional(&mut *db)
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
                .execute(&mut *db)
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
AND is_deleted = FALSE
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

    #[allow(clippy::blocks_in_if_conditions)] // it's supposedly bad beacuse of code cleanness but
                                              // in this case it's cleaner
    pub async fn get<C: AsyncCommands>(
        id: u64,
        requester_id: Option<u64>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        sqlx::query!(
            r#"
SELECT id, username, display_name, social_credit, status, status_type as "status_type: StatusType", bio, avatar, banner, badges, permissions, email, verified
FROM users
WHERE id = $1
AND is_deleted = FALSE
            "#,
            id as i64
        )
        .fetch_optional(db)
        .await
        .map_err(|err| {
            log::error!("Couldn't get user from database: {}", err);
            error!(SERVER, "Failed to get user data")
        })?
        .map(|u| async move {
            Ok(Self {
                id: u.id as u64,
                username: u.username,
                display_name: u.display_name,
                social_credit: u.social_credit,
                status: if  Some(id) == requester_id  ||
                    cache
                    .sismember::<_, _, bool>("sessions", u.id as u64)
                    .await
                    .map_err(|err| {
                        log::error!("Failed to determine if user is online: {}", err);
                        error!(SERVER, "Couldn't provide user data")
                    })? {
                        Status {
                        status_type: u.status_type,
                            text: u.status,
                        }
                } else {
                    Status {
                        status_type: StatusType::Offline,
                        text: None,
                    }
                },
                bio: u.bio,
                avatar: u.avatar.map(|a| a as u64),
                banner: u.banner.map(|b| b as u64),
                badges: u.badges as u64,
                permissions: u.permissions as u64,
                email: (Some(id) == requester_id).then_some(u.email),
                verified: (Some(id) == requester_id).then_some(u.verified)
            })
        })
        .ok_or_else(|| error!(NOT_FOUND))?.await
    }

    #[allow(clippy::blocks_in_if_conditions)]
    pub async fn get_username<C: AsyncCommands>(
        username: &str,
        requester_id: Option<u64>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        sqlx::query!(
            r#"
SELECT id, username, display_name, social_credit, status, status_type as "status_type: StatusType", bio, avatar, banner, badges, permissions, email, verified
FROM users
WHERE username = $1
AND is_deleted = FALSE
            "#,
            username
        )
        .fetch_optional(db)
        .await
        .map_err(|err| {
            log::error!("Couldn't get user from database: {}", err);
            error!(SERVER, "Failed to get user data")
        })?
        .map(|u| async move {
            Ok(Self {
                id: u.id as u64,
                username: u.username,
                display_name: u.display_name,
                social_credit: u.social_credit,
                status: if Some(u.id as u64) == requester_id || cache
                    .sismember::<_, _, bool>("sessions", u.id as u64)
                    .await
                    .map_err(|err| {
                        log::error!("Failed to determine if user is online: {}", err);
                        error!(SERVER, "Couldn't provide user data")
                    })? {
                    Status {
                        status_type: u.status_type,
                        text: u.status,
                    }
                } else {
                    Status {
                        status_type: StatusType::Offline,
                        text: None,
                    }
                },
                bio: u.bio,
                avatar: u.avatar.map(|a| a as u64),
                banner: u.banner.map(|b| b as u64),
                badges: u.badges as u64,
                permissions: u.permissions as u64,
                email: (Some(u.id as u64) == requester_id).then_some(u.email),
                verified: (Some(u.id as u64) == requester_id).then_some(u.verified)
            })
        })
        .ok_or_else(|| error!(NOT_FOUND))?
        .await
    }

    pub async fn update<H: PasswordHasher, R: CryptoRngCore>(
        id: u64,
        user: UpdateUser,
        hasher: &H,
        rng: &mut R,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        user.validate(&mut *db).await?;
        Self::validate_password(id, &user.password, hasher, db).await?;
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE users SET ");
        let mut seperated = query.separated(", ");
        if let Some(username) = user.username {
            seperated
                .push("username = ")
                .push_bind_unseparated(username);
        }
        if let Some(email) = user.email {
            seperated.push("email = ").push_bind_unseparated(email);
        }
        if let Some(new_password) = user.new_password {
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
        query
            .push(" WHERE id = ")
            .push_bind(id as i64)
            .push(
                " RETURNING id, username, display_name, social_credit, status, status_type, bio, avatar, banner, badges, permissions, email, verified",
            )
            .build()
            .fetch_one(db)
            .await
            .map(|u| Self {
                id: u.get::<i64, _>("id") as u64,
                username: u.get("username"),
                display_name: u.get("display_name"),
                social_credit: u.get("social_credit"),
                status: Status {
                    status_type: u.get("status_type"),
                    text: u.get("status"),
                },
                bio: u.get("bio"),
                avatar: u.get::<Option<i64>, _>("avatar").map(|a| a as u64),
                banner: u.get::<Option<i64>, _>("banner").map(|b| b as u64),
                badges: u.get::<i64, _>("badges") as u64,
                permissions: u.get::<i64, _>("permissions") as u64,
                email: Some(u.get("email")),
                verified: Some(u.get("verified")),
            })
            .map_err(|err| {
                log::error!("Couldn't update user profile: {}", err);
                error!(SERVER, "Failed to update user profile")
            })
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
        if let Some(status_type) = profile.status_type {
            seperated
                .push("status_type = ")
                .push_bind_unseparated(status_type);
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
                " RETURNING id, username, display_name, social_credit, status, status_type, bio, avatar, banner, badges, permissions, email, verified",
            )
            .build()
            .fetch_one(db)
            .await
            .map(|u| Self {
                id: u.get::<i64, _>("id") as u64,
                username: u.get("username"),
                display_name: u.get("display_name"),
                social_credit: u.get("social_credit"),
                status: Status {
                    status_type: u.get("status_type"),
                    text: u.get("status"),
                },
                bio: u.get("bio"),
                avatar: u.get::<Option<i64>, _>("avatar").map(|a| a as u64),
                banner: u.get::<Option<i64>, _>("banner").map(|b| b as u64),
                badges: u.get::<i64, _>("badges") as u64,
                permissions: u.get::<i64, _>("permissions") as u64,
                email: Some(u.get("email")),
                verified: Some(u.get("verified")),
            })
            .map_err(|err| {
                log::error!("Couldn't update user profile: {}", err);
                error!(SERVER, "Failed to update user profile")
            })
    }

    pub async fn delete<V: PasswordVerifier>(
        id: u64,
        delete: PasswordDeleteCredentials,
        verifier: &V,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<(), ErrorResponse> {
        Self::validate_password(id, &delete.password, verifier, db).await?;
        sqlx::query!(
            "
UPDATE users
SET is_deleted = TRUE
WHERE id = $1
            ",
            id as i64
        )
        .execute(db)
        .await
        .map_err(|err| {
            log::error!("Couldn't mark user as deleted: {}", err);
            error!(SERVER, "Failed to delete user")
        })?;
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
        .execute(db)
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
