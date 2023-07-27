use clap::Parser;

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::fmt().init();

    run().await?;

    Ok(())
}

/// Run the program
async fn run() -> color_eyre::eyre::Result<()> {
    match c::Cli::parse().command {
        Some(c::Commands::Anthropic(options)) => c::commands::anthropic::run(options).await?,
        Some(c::Commands::OpenAi(options)) => c::commands::openai::run(options).await?,
        None => {
            color_eyre::eyre::bail!(
                "No subcommand provided. Use --help to see available subcommands."
            )
        }
    }

    Ok(())
}