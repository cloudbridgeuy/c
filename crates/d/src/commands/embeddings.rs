use clap::Parser;
use color_eyre::eyre::Result;
use openai::embeddings::Embedding;
use std::io::Write;

use crate::constants::MODEL;

#[derive(Default, Clone, Parser, Debug)]
pub struct Options {
    /// Input text to get embeddings for.
    input: String,
}

pub async fn run(options: Options) -> Result<()> {
    let mut sp: Option<spinners::Spinner> = None;

    if atty::is(atty::Stream::Stdout) {
        sp = Some(spinners::Spinner::new(
            spinners::Spinners::OrangeBluePulse,
            "Loading...".into(),
        ));
    }

    let embedding = Embedding::create(MODEL, &options.input, &String::default()).await?;

    if atty::is(atty::Stream::Stdout) && sp.is_some() {
        sp.take().unwrap().stop();
        // Flush `stdout`
        std::io::stdout().flush()?;
    }

    // Concatenate all the values of embedding into a string separated by a comma
    let embedding = embedding
        .vec
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");

    println!("{embedding}");

    Ok(())
}
