use clap::{Parser, Subcommand};
use color_eyre::eyre::{bail, eyre};

mod commands;
mod models;
mod printer;
mod sessions;
mod shutdown;
mod similarity;
mod vector;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// OpenAI Chat AI API
    #[clap(name = "chat", alias = "c")]
    Chat(commands::chat::Options),
    /// OpenAI Embedding commands
    #[clap(name = "embeddings", alias = "e")]
    Embeddings(commands::embeddings::Options),
    /// Vector commands
    #[clap(name = "vector", alias = "v")]
    Vector(commands::vector::Cli),
}

#[derive(Debug, Parser)]
#[command(name = "d")]
#[command(about = "Interact with LLMs through the terminal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    env_logger::init();

    // Load the OpenAI API Key from the OPENAI_API_KEY environment variable.
    openai::set_key(std::env::var("OPENAI_API_KEY")?);

    // Create the shutdown handler
    let shutdown = shutdown::Shutdown::new()?;

    // Run app in separate async task
    tokio::spawn(async {
        if let Err(e) = run().await {
            bail!("Application error: {}", e)
        }

        Ok(())
    });

    shutdown.handle().await;

    Ok(())
}

async fn run() -> color_eyre::eyre::Result<()> {
    let result = match Cli::parse().command {
        Some(Commands::Chat(options)) => commands::chat::run(options).await,
        Some(Commands::Embeddings(options)) => commands::embeddings::run(options).await,
        Some(Commands::Vector(cli)) => commands::vector::run(cli).await,
        None => Err(eyre!(
            "No subcommand provided. Use --help to see available subcommands."
        )),
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
