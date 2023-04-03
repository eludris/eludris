use anyhow::Context;
use derailed::{check_user_permissions, new_docker_command};

pub async fn logs() -> anyhow::Result<()> {
    check_user_permissions()?;

    new_docker_command()
        .arg("logs")
        .arg("-f")
        .spawn()
        .context("Could not spawn stop command")?
        .wait()
        .await
        .context("Could not stop instance, you're on your own now soldier. Good luck :D")?;

    Ok(())
}
