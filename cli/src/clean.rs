use anyhow::{bail, Context};
use eludris::{check_eludris_exists, end_progress_bar, get_user_config, new_progress_bar};
use tokio::fs;

pub async fn clean() -> anyhow::Result<()> {
    let config = get_user_config()
        .await?
        .context("Could not find user config")?;

    if !check_eludris_exists(&config)? {
        bail!("Could not find an Eludris instance on this machine");
    }

    let bar = new_progress_bar("Removing old instance files...");
    fs::remove_dir_all(config.eludris_dir)
        .await
        .context("Could not remove Eludris instance files")?;
    end_progress_bar(bar, "Removed old instance files");
    Ok(())
}
