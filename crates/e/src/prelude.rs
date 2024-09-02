use futures::stream::{Stream, TryStreamExt};
use std::io::Write;

pub use crate::args::{Api, Args};
pub use crate::config::Config;
pub use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub async fn handle_stream(
    mut stream: impl Stream<Item = std::result::Result<String, es_stream::error::Error>>
        + std::marker::Unpin,
    quiet: bool,
) -> Result<()> {
    let mut previous_output = String::new();
    let mut accumulated_content_bytes: Vec<u8> = Vec::new();

    let is_terminal = atty::is(atty::Stream::Stdout);

    let mut sp = if !quiet && is_terminal {
        Some(spinners::Spinner::new(
            spinners::Spinners::OrangeBluePulse,
            "Loading...".into(),
        ))
    } else {
        None
    };

    while let Ok(Some(text)) = stream.try_next().await {
        if is_terminal && sp.is_some() {
            // TODO: Find a better way to clean the spinner from the terminal.
            sp.take().unwrap().stop();
            std::io::stdout().flush()?;
            crossterm::execute!(std::io::stdout(), crossterm::cursor::MoveToColumn(0))?;
            print!("                      ");
            crossterm::execute!(std::io::stdout(), crossterm::cursor::MoveToColumn(0))?;
        }

        if !is_terminal {
            // If not a terminal, print each instance of `text` directly to `stdout`
            print!("{}", text);
            std::io::stdout().flush()?;
            continue;
        }

        accumulated_content_bytes.extend_from_slice(text.as_bytes());

        let output = crate::printer::CustomPrinter::new()?
            .input_from_bytes(&accumulated_content_bytes)
            .print()?;

        let unprinted_lines = output
            .lines()
            .skip(if previous_output.lines().count() == 0 {
                0
            } else {
                previous_output.lines().count() - 1
            })
            .collect::<Vec<_>>()
            .join("\n");

        crossterm::execute!(std::io::stdout(), crossterm::cursor::MoveToColumn(0))?;
        print!("{unprinted_lines}");
        std::io::stdout().flush()?;

        // Update the previous output
        previous_output = output;
    }

    Ok(())
}
