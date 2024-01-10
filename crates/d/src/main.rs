mod ai;

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
    crate::ai::run().await?;

    Ok(())
}
