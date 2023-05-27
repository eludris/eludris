mod attachments;
mod clean;
mod deploy;
mod logs;
mod static_attachments;
mod stop;
mod update;

use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::{Parser, Subcommand};
use console::Style;
use dialoguer::{theme, Input};
use eludris::{get_config_directory, get_user_config, update_config_file, Config};
use tokio::fs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
#[command(next_line_help = true)]
struct Cli {
    /// Turn debugging information on.
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploys your Eludris instance
    Deploy {
        /// Use a development Eludris instance
        #[arg(long)]
        next: bool,
    },
    /// Stops your Eludris instance
    Stop,
    /// Updates your Eludris instance
    Update {
        /// Update to the latest development version of Eludris
        #[arg(long)]
        next: bool,
    },
    /// Shows you your instance's logs
    Logs,
    /// Static attachment related commands
    Static {
        #[command(subcommand)]
        command: StaticSubcommand,
    },
    /// Attachment related commands
    Attachments {
        #[command(subcommand)]
        command: AttachmentSubcommand,
    },
    /// Removes all info related to your Eludris instance
    #[command(alias = "clear")]
    Clean,
    /// Returns the CLI's current config directory
    ConfDir,
}

#[derive(Subcommand)]
enum StaticSubcommand {
    /// Adds a static attachment
    Add {
        /// Path of the file you want to add
        path: PathBuf,
    },
    /// Removes a static attachment
    Remove {
        /// Name of the attachment you want to remove
        name: String,
    },
}
#[derive(Subcommand)]
enum AttachmentSubcommand {
    /// Removes an attachment
    Remove {
        /// The id of the attchment to be removed
        id: u64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    dotenvy::dotenv().ok();

    match cli.debug {
        0 => {}
        1 => env::set_var("RUST_LOG", "error"),
        2 => env::set_var("RUST_LOG", "warn"),
        3 => env::set_var("RUST_LOG", "debug"),
        _ => env::set_var("RUST_LOG", "trace"), // >= 4
    };
    env_logger::init();

    if get_user_config().await?.is_none() {
        let path_input = Input::with_theme(&theme::ColorfulTheme {
            prompt_prefix: Style::new().yellow().bold().apply_to("~>".to_string()),
            success_prefix: Style::new().green().bold().apply_to("~>".to_string()),
            error_prefix: Style::new().red().bold().apply_to("~>".to_string()),
            ..Default::default()
        })
        .with_prompt(
            "Enter where you want your Eludris instance's files to be (note: leaving it as /usr/eludris will require root previlages in the future)",
        )
        .default("/usr/eludris".to_string())
        .interact_text()
        .context("Could not prompt user")?.replace('~', &env::var("HOME").context("Could not find home path")?);
        if !fs::try_exists(&path_input).await? {
            fs::create_dir_all(&path_input).await?;
        }
        let config = Config {
            eludris_dir: Path::new(&path_input).canonicalize()?.display().to_string(),
        };
        update_config_file(&config).await?;
    }

    match cli.command {
        Commands::Deploy { next } => deploy::deploy(next).await?,
        Commands::Stop => stop::stop().await?,
        Commands::Update { next } => update::update(next).await?,
        Commands::Logs => logs::logs().await?,
        Commands::Static { command } => match command {
            StaticSubcommand::Add { path } => static_attachments::add(path).await?,
            StaticSubcommand::Remove { name } => static_attachments::remove(name).await?,
        },
        Commands::Attachments { command } => match command {
            AttachmentSubcommand::Remove { id } => attachments::remove(id).await?,
        },
        Commands::Clean => clean::clean().await?,
        Commands::ConfDir => println!("{}", get_config_directory()?.display()),
    }

    Ok(())
}
