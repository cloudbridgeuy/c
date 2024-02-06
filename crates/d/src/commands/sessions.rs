use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List the available sessions
    #[clap(name = "list")]
    List,
    /// Reads a session or a session message
    #[clap(name = "read")]
    Read(ReadOptions),
}

#[derive(Default, Clone, Parser, Debug)]
pub struct ReadOptions {
    /// Session name
    session: String,
    /// Session message id
    #[clap(short, long)]
    id: Option<String>,
}

#[derive(Debug, Parser)]
#[command(name = "sessions")]
#[command(about = "Manage sessions")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Runs the `sessions` command
pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::List) => list().await?,
        Some(Commands::Read(options)) => read(options).await?,
        None => {
            color_eyre::eyre::bail!("No subcommand provided. Use `d sessions help` to see the list of available subcommands.")
        }
    }

    Ok(())
}

/// Runs the `list` command
pub async fn list() -> Result<()> {
    let home = std::env::var("D_ROOT").unwrap_or(std::env::var("HOME")?);
    let path = format!("{home}/.d/sessions");

    // List all the `.yaml` files inside of `path` without the extension.
    let sessions = std::fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|entry| entry.ends_with(".yaml"))
        .map(|entry| entry.trim_end_matches(".yaml").to_string())
        .collect::<Vec<String>>();

    println!("{}", serde_json::to_string_pretty(&sessions)?);

    Ok(())
}

/// Runs the `read` command
pub async fn read(options: ReadOptions) -> Result<()> {
    match options.id {
        Some(id) => print_message(&options.session, &id),
        None => print_session(&options.session),
    }
}

fn print_session(session: &str) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&crate::sessions::Session::load(String::from(session))?)?
    );

    Ok(())
}

fn print_message(name: &str, id: &str) -> Result<()> {
    let session = &crate::sessions::Session::load(String::from(name))?;

    // Find the message inside `session.messages` who's id is `id`
    let messages = session.messages();
    let message = messages
        .iter()
        .find(|message| message.id == id)
        .ok_or_else(|| {
            color_eyre::eyre::eyre!("Message with id {} not found in session {}", id, name)
        })?;

    println!("{}", serde_json::to_string_pretty(&message)?);

    Ok(())
}
