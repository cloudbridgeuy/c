use clap::Parser;

mod anthropic;
mod args;
mod error;
mod prelude;

use crate::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let api: Api = args.globals.api.parse()?;

    let result = match api {
        Api::OpenAi => todo!(),
        Api::Anthropic => anthropic::run(args).await,
    };

    if result.is_err() {
        println!("{:#?}", result);
    }

    Ok(())
}
