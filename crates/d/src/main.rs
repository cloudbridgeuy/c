use clap::{Parser, Subcommand};
use color_eyre::eyre::bail;

mod commands;
mod printer;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// OpenAI Chat AI API
    #[clap(name = "openai", alias = "o")]
    OpenAi(commands::openai::CommandOptions),
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

    run().await?;
    Ok(())
}

async fn run() -> color_eyre::eyre::Result<()> {
    match Cli::parse().command {
        Some(Commands::OpenAi(options)) => crate::commands::openai::run(options).await?,
        None => {
            bail!("No subcommand provided. Use --help to see available subcommands.")
        }
    }

    Ok(())
}
