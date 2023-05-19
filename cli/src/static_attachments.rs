use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use console::Style;
use eludris::get_user_config;
use tokio::fs;

pub async fn add(path: PathBuf) -> anyhow::Result<()> {
    let config = get_user_config()
        .await?
        .context("Could not find user config")?;

    if !path.exists() {
        bail!(
            "{}",
            Style::new()
                .red()
                .apply_to(format!("Could not find file {}", path.display()))
        );
    }
    let destination_path = Path::new(&format!("{}/files/static", config.eludris_dir))
        .join(path.file_name().context("Could not extract file name")?);
    if destination_path.exists() {
        bail!(
            "{}",
            Style::new()
                .red()
                .apply_to("A static file with the same name already exists")
        );
    }
    fs::copy(path, destination_path)
        .await
        .context("Could not make static attachment")?;
    Ok(())
}

pub async fn remove(name: String) -> anyhow::Result<()> {
    let config = get_user_config()
        .await?
        .context("Could not find user config")?;

    if !Path::new(&format!("{}/files/static/{}", config.eludris_dir, name)).exists() {
        bail!(
            "{}",
            Style::new()
                .red()
                .apply_to(format!("Static file {} does not exist", name))
        );
    }
    fs::remove_file(format!("{}/files/static/{}", config.eludris_dir, name))
        .await
        .context("Could not remove static file")?;
    Ok(())
}
