use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use console::Style;
use derailed::check_user_permissions;
use tokio::fs;

pub async fn add(path: PathBuf) -> anyhow::Result<()> {
    check_user_permissions()?;

    if !path.exists() {
        bail!(
            "{}",
            Style::new()
                .red()
                .apply_to(format!("Could not find file {}", path.display()))
        );
    }
    let destination_path = Path::new("/usr/derailed/files/static")
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
    check_user_permissions()?;

    if !Path::new(&format!("/usr/derailed/files/static/{}", name)).exists() {
        bail!(
            "{}",
            Style::new()
                .red()
                .apply_to(format!("Static file {} does not exist", name))
        );
    }
    fs::remove_file(format!("/usr/derailed/files/static/{}", name))
        .await
        .context("Could not remove static file")?;
    Ok(())
}
