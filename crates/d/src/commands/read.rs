use clap::Parser;
use color_eyre::eyre::Result;

#[derive(Default, Clone, Parser, Debug)]
pub struct Options {
    /// Session name
    session: String,
    /// Session message id
    #[clap(short, long)]
    id: Option<String>,
}

/// Runs the `read` command
pub async fn run(options: Options) -> Result<()> {
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
