use anyhow::Context;
use eludris::{get_user_config, new_docker_command};

pub async fn logs() -> anyhow::Result<()> {
    let config = get_user_config()
        .await?
        .context("Could not find user config")?;

    new_docker_command(&config)
        .arg("logs")
        .arg("-f")
        .spawn()
        .context("Could not spawn stop command")?
        .wait()
        .await
        .context("Could not stop instance, you're on your own now soldier. Good luck :D")?;

    Ok(())
}
