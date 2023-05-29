use std::path::Path;

use anyhow::{bail, Context};
use eludris::{get_user_config, new_database_connection};
use tokio::fs;

pub async fn remove(id: u64) -> anyhow::Result<()> {
    let config = get_user_config()
        .await?
        .context("Could not find user config")?;
    if !Path::new(&format!("{}/files/attachments/{}", config.eludris_dir, id)).exists() {
        bail!("Could not find attachment with id {}", id);
    }

    let mut database = new_database_connection().await?;
    sqlx::query!(
        "
DELETE FROM files
WHERE id = $1
AND bucket = 'attachments'
        ",
        id as i64
    )
    .execute(&mut database)
    .await
    .context("Could not remove attachment from database")?;

    fs::remove_file(format!("{}/files/attachments/{}", config.eludris_dir, id))
        .await
        .context("Failed to remove file from filesystem")?;

    Ok(())
}
