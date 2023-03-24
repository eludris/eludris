mod attachments;
mod clean;
mod deploy;
mod logs;
mod static_attachments;
mod stop;

use std::{env, path::PathBuf};

use clap::{Parser, Subcommand};

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
    Deploy,
    /// Stops your Eludris instance
    Stop,
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
        id: u128,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.debug {
        0 => {}
        1 => env::set_var("RUST_LOG", "error"),
        2 => env::set_var("RUST_LOG", "warn"),
        3 => env::set_var("RUST_LOG", "debug"),
        _ => env::set_var("RUST_LOG", "trace"), // >= 4
    };
    env_logger::init();

    match cli.command {
        Commands::Deploy => deploy::deploy().await?,
        Commands::Stop => stop::stop().await?,
        Commands::Logs => logs::logs().await?,
        Commands::Static { command } => match command {
            StaticSubcommand::Add { path } => static_attachments::add(path).await?,
            StaticSubcommand::Remove { name } => static_attachments::remove(name).await?,
        },
        Commands::Attachments { command } => match command {
            AttachmentSubcommand::Remove { id } => attachments::remove(id).await?,
        },
        Commands::Clean => clean::clean().await?,
    }

    Ok(())
}
