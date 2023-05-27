use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, MySqlConnection};
use tokio::{fs, process::Command};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub eludris_dir: String,
}

pub fn get_conf_directory() -> Result<PathBuf> {
    // `ELUDRIS_CLI_CONF` here tries to follow `ELUDRIS_CONF` from `/todel/src/conf/mod.rs`
    match env::var("ELUDRIS_CLI_CONF") {
        Ok(dir) => Ok(PathBuf::try_from(dir).context(
            "Could not convert the `ELUDRIS_CLI_CONF` environment variable into a valid path",
        )?),
        Err(env::VarError::NotPresent) => Ok(ProjectDirs::from("", "eludris", "eludris")
            // According to the `directories` docs the error is raised when a home path isn't found
            // but that wouldn't make much sense for windows so we use `base` here.
            .context("Could not find a valid base directory")?
            .config_dir()
            .to_path_buf()),
        Err(env::VarError::NotUnicode(_)) => {
            bail!("The value of the `ELUDRIS_CLI_CONF` environment variable must be valid unicode")
        }
    }
}

pub async fn get_user_config() -> Result<Option<Config>> {
    let config_dir = get_conf_directory()?;

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .await
            .context("Could not create config directory")?;
    }

    let config_path = config_dir.join("Cli.toml");

    if !config_path.exists() {
        Ok(None)
    } else {
        let config = fs::read_to_string(config_path)
            .await
            .context("Could not read config file")?;
        Ok(Some(
            toml::from_str(&config).context("Could not parse config file")?,
        ))
    }
}

pub async fn update_config_file(config: &Config) -> Result<()> {
    let config_dir = get_conf_directory()?;

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .await
            .context("Could not create config directory")?;
    }

    let config_path = config_dir.join("Cli.toml");

    fs::write(
        config_path,
        toml::to_string(&config).context("Could not serialize default config")?,
    )
    .await
    .context("Could not find config file")?;
    Ok(())
}

pub fn check_eludris_exists(config: &Config) -> Result<bool> {
    let path = Path::new(&config.eludris_dir);
    if !path.is_dir() && path.exists() {
        bail!("An Eludris file exists but it is not a directory");
    }
    Ok(path.join("Eludris.toml").exists())
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

pub fn new_docker_command(config: &Config) -> Command {
    let mut command = Command::new("docker-compose");
    command
        .current_dir(&config.eludris_dir)
        .arg("-f")
        .arg("docker-compose.override.yml")
        .arg("-f")
        .arg("docker-compose.yml");
    command
}

pub async fn new_database_connection() -> Result<MySqlConnection> {
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
    config: &Config,
    client: &Client,
    name: &str,
    next: bool,
    save_name: Option<&str>,
) -> Result<()> {
    log::info!("Fetching {}", name);
    let file = client
        .get(format!(
            "https://raw.githubusercontent.com/eludris/eludris/{}/{}",
            if next { "next" } else { "main" },
            if name == "docker-compose.prebuilt.yml" && next {
                "docker-compose.next.yml"
            } else {
                name
            }
        ))
        .send()
        .await
        .context(
            "Failed to fetch necessary files for setup. Please check your connection and try again",
        )?
        .text()
        .await
        .context("Failed to fetch necessary files for setup")?;
    log::info!("Writing {}", name);
    fs::write(
        format!("{}/{}", config.eludris_dir, save_name.unwrap_or(name)),
        file,
    )
    .await
    .context("Could not write setup files")?;
    Ok(())
}
