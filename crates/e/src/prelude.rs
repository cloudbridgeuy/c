use futures::stream::{Stream, TryStreamExt};
use std::io::Write;

pub use crate::args::{Api, Args};
pub use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub async fn handle_stream(
    mut stream: impl Stream<Item = std::result::Result<String, es_stream::error::Error>>
        + std::marker::Unpin,
    quiet: bool,
) -> Result<()> {
    let mut sp = if !quiet {
        Some(spinners::Spinner::new(
            spinners::Spinners::OrangeBluePulse,
            "Loading...".into(),
        ))
    } else {
        None
    };

    while let Ok(Some(text)) = stream.try_next().await {
        if atty::is(atty::Stream::Stdout) && sp.is_some() {
            // TODO: Find a better way to clean the spinner from the terminal.
            sp.take().unwrap().stop();
            std::io::stdout().flush()?;
            crossterm::execute!(std::io::stdout(), crossterm::cursor::MoveToColumn(0))?;
            print!("                      ");
            crossterm::execute!(std::io::stdout(), crossterm::cursor::MoveToColumn(0))?;
        }

        print!("{text}");
        std::io::stdout().flush()?;
    }

    Ok(())
}
