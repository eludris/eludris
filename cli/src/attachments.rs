use std::path::Path;

use anyhow::{bail, Context};
use derailed::new_database_connection;
use tokio::fs;

pub async fn remove(id: u128) -> anyhow::Result<()> {
    if !Path::new(&format!("/usr/derailed/files/attachments/{}", id)).exists() {
        bail!("Could not find attachment with id {}", id);
    }

    let mut database = new_database_connection().await?;
    sqlx::query!(
        "
DELETE FROM files
WHERE id = ?
AND bucket = 'attachments'
        ",
        id.to_string(),
    )
    .execute(&mut database)
    .await
    .context("Could not remove attachment from database")?;

    fs::remove_file(format!("/usr/derailed/files/attachments/{}", id))
        .await
        .context("Failed to remove file from filesystem")?;

    Ok(())
}
