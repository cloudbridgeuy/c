use clap::Parser;
use std::str::FromStr;

mod error;
mod prelude;

use crate::prelude::*;

#[derive(Debug, clap::Args)]
pub struct Globals {
    /// The API provider to use.
    #[clap(short, long, default_value = "anthropic", env = "E_API")]
    api: String,
}

pub enum API {
    OpenAi,
    Anthropic,
}

// From string to API enum
impl FromStr for API {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "OpenAi" => Ok(API::OpenAi),
            "Anthropic" => Ok(API::Anthropic),
            _ => Err(Error::InvalidAPI),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "e")]
#[command(about = "Interact with LLMs through the terminal")]
pub struct Args {
    #[clap(flatten)]
    globals: Globals,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    println!("{}", args.globals.api);

    Ok(())
}
