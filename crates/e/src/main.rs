use clap::Parser;

mod anthropic;
mod args;
mod error;
mod google;
mod openai;
mod prelude;
mod printer;

use crate::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let api: Api = args.globals.api.parse()?;

    match api {
        Api::OpenAi => openai::run(args).await,
        Api::Anthropic => anthropic::run(args).await,
        Api::Google => google::run(args).await,
    }
}
