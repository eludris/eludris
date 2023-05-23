use anyhow::Context;
use console::Style;
use eludris::{
    check_eludris_exists, download_file, end_progress_bar, get_user_config, new_docker_command,
    new_progress_bar,
};
use reqwest::Client;

pub async fn update(next: bool) -> anyhow::Result<()> {
    let config = get_user_config()
        .await?
        .context("Could not find user config")?;

    if !check_eludris_exists(&config)? {
        println!(
            "{}",
            Style::new().yellow().apply_to(
                "You do not currently have an Eludris system. Run `eludris deploy` to create one"
            )
        );
        return Ok(());
    };

    let bar = new_progress_bar("Updating Docker-related files...");
    let client = Client::new();
    download_file(
        &config,
        &client,
        "docker-compose.prebuilt.yml",
        next,
        Some("docker-compose.yml"),
    )
    .await?;
    download_file(&config, &client, "docker-compose.override.yml", next, None).await?;
    end_progress_bar(bar, "Finished updating Docker-related files");

    let command = new_docker_command(&config)
        .arg("build")
        .spawn()
        .context("Could not rebuild instance")?
        .wait()
        .await
        .context("Instance failed to start")?;

    if command.success() {
        println!(
            "{}",
            Style::new()
                .green()
                .apply_to("Instance successfully updated!")
        );
    }
    Ok(())
}
