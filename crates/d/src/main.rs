use clap::{Parser, Subcommand};
use color_eyre::eyre::bail;
use crossterm::cursor::Show;
use crossterm::execute;
use std::io::stdout;
use tokio::signal::unix::{signal, SignalKind};

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

    // Set up the signal handler for SIGINT
    let mut sigint = signal(SignalKind::interrupt())?;

    // Run your application logic in a separate async task
    let app_task = tokio::spawn(async {
        if let Err(e) = run().await {
            bail!("Application error: {}", e)
        }

        Ok(())
    });

    tokio::select! {
        _ = app_task => {
            // The application has finished running
        },
        _ = sigint.recv() => {
            if let Err(e) = execute!(stdout(), Show) {
                eprintln!("Failed to restore cursor: {}", e);
            }
        },
    }

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
