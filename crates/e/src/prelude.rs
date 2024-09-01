use futures::stream::{Stream, TryStreamExt};
use std::io::Write;

pub use crate::args::{Api, Args};
pub use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub async fn handle_stream(
    mut stream: impl Stream<Item = std::result::Result<String, es_stream::error::Error>>
        + std::marker::Unpin,
) -> Result<()> {
    while let Ok(Some(text)) = stream.try_next().await {
        print!("{text}");
        std::io::stdout().flush()?;
    }

    Ok(())
}
