use anyhow::{bail, Context};
use derailed::{check_derailed_exists, check_user_permissions, end_progress_bar, new_progress_bar};
use tokio::fs;

pub async fn clean() -> anyhow::Result<()> {
    check_user_permissions()?;

    if !check_derailed_exists()? {
        bail!("Could not find an Derailed instance on this machine");
    }

    let bar = new_progress_bar("Removing old instance files...");
    fs::remove_dir_all("/usr/derailed")
        .await
        .context("Could not remove Derailed instance files")?;
    end_progress_bar(bar, "Removed old instance files");
    Ok(())
}
