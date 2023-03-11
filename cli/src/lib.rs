use std::{ffi::OsStr, path::Path, process::Stdio, time::Duration};

use anyhow::{anyhow, bail, Context};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use sqlx::{Connection, MySqlConnection};
use tokio::{fs, process::Command};
use users::{get_current_uid, get_user_by_uid};

pub fn check_user_permissions() -> anyhow::Result<()> {
    let user =
        get_user_by_uid(get_current_uid()).ok_or_else(|| anyhow!("Could not get user data"))?;
    if user.name() != OsStr::new("root") {
        log::info!("User is not root, bailing");
        bail!("You must be root to run this command. Try sudo?");
    }

    Ok(())
}

pub fn check_eludris_exists() -> anyhow::Result<bool> {
    let path = Path::new("/usr/eludris");
    if !path.is_dir() && path.exists() {
        bail!("An Eludris file exists but it is not a directory");
    }
    Ok(path.exists())
}

pub fn new_progress_bar(message: &str) -> ProgressBar {
    let bar = ProgressBar::new_spinner()
        .with_message(message.to_string())
        .with_prefix("~>")
        .with_style(
            ProgressStyle::with_template("{prefix:.yellow.bold} {spinner:.blue.bold} {msg}")
                .unwrap()
                .tick_strings(&[".    ", "..   ", "...  ", ".... ", "....."]),
        );
    bar.enable_steady_tick(Duration::from_millis(100));
    bar
}

pub fn end_progress_bar(bar: ProgressBar, message: &str) {
    bar.set_style(ProgressStyle::with_template("{prefix:.green.bold} {msg}").unwrap());
    bar.finish_with_message(message.to_string());
}

pub fn new_docker_command() -> Command {
    let mut command = Command::new("docker-compose");
    command // One can never be *too* sure
        .current_dir("/usr/eludris")
        .arg("-f")
        .arg("/usr/eludris/docker-compose.override.yml")
        .arg("-f")
        .arg("/usr/eludris/docker-compose.yml");
    command
}

pub async fn new_database_connection() -> anyhow::Result<MySqlConnection> {
    let stdout = Command::new("docker")
        .arg("inspect")
        .arg("-f")
        .arg("{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}")
        .arg("eludris-mariadb-1")
        .stdout(Stdio::piped())
        .output()
        .await
        .context("Could not fetch mariadb address, is the docker daemon running?")?
        .stdout;
    let address = String::from_utf8(stdout).context("Could not convert address to a string")?;

    MySqlConnection::connect(&format!("mysql://root:root@{}:3306/eludris", address))
        .await
        .context("Could not connect to database")
}

pub async fn download_file(
    client: &Client,
    name: &str,
    next: bool,
    save_name: Option<&str>,
) -> anyhow::Result<()> {
    log::info!("Fetching {}", name);
    let file = client
        .get(format!(
            "https://raw.githubusercontent.com/eludris/eludris/{}/{}",
            if next { "next" } else { "main" },
            name
        ))
        .send()
        .await
        .context(
            "Failed to fetch neccesary files for setup. Please check your connection and try again",
        )?
        .text()
        .await
        .context("Failed to fetch neccesary files for setup")?;
    log::info!("Writing {}", name);
    fs::write(format!("/usr/eludris/{}", save_name.unwrap_or(name)), file)
        .await
        .context("Could not write setup files")?;
    Ok(())
}
