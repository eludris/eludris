mod add_reaction;
mod delete;
mod edit;
mod remove_reaction;

use sqlx::{pool::PoolConnection, Postgres};

use crate::{
    ids::IdGenerator,
    models::{Emoji, EmojiCreate, ErrorResponse, File, Sphere},
};

impl EmojiCreate {
    pub fn validate(&self) -> Result<(), ErrorResponse> {
        if self.name.len() < 2 || self.name.len() > 32 {
            return Err(error!(
                VALIDATION,
                "name", "The emoji's name must be between 2 and 32 characters in length"
            ));
        }
        Ok(())
    }
}

impl Sphere {
    pub async fn add_emoji(
        &self,
        create: EmojiCreate,
        uploader_id: u64,
        id_generator: &mut IdGenerator,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Emoji, ErrorResponse> {
        create.validate()?;

        if File::get(create.file_id, "emojis", &mut *db)
            .await
            .is_none()
        {
            return Err(error!(
                VALIDATION,
                "file_id", "The emoji's file must be a valid file that exists in the emojis bucket"
            ));
        }

        let id = id_generator.generate();

        sqlx::query!(
            "
            INSERT INTO emojis(id, sphere_id, name, file_id, uploader_id)
            VALUES($1, $2, $3, $4, $5)
            ",
            id as i64,
            self.id as i64,
            create.name,
            create.file_id as i64,
            uploader_id as i64,
        )
        .execute(&mut **db)
        .await
        .map_err(|err| {
            log::error!(
                "Failed to insert emoji into database with file {}: {}",
                create.file_id,
                err
            );
            error!(SERVER, "Failed to create emoji")
        })?;

        Ok(Emoji {
            id,
            file_id: create.file_id,
            name: create.name,
            uploader_id,
            sphere_id: self.id,
        })
    }
}

impl Emoji {
    pub async fn get(id: u64, db: &mut PoolConnection<Postgres>) -> Result<Self, ErrorResponse> {
        sqlx::query!(
            "
            SELECT *
            FROM emojis
            WHERE is_deleted = FALSE
            "
        )
        .fetch_one(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Failed to get emoji into database {}: {}", id, err);
            error!(SERVER, "Failed to get emoji")
        })
        .map(|r| Self {
            id: r.id as u64,
            file_id: r.file_id as u64,
            name: r.name,
            uploader_id: r.uploader_id as u64,
            sphere_id: r.sphere_id as u64,
        })
    }
}
