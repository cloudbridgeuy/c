//! See <https://github.com/matklad/cargo-xtask/>
//!
//! This binary defines various auxiliary build commands, which are not
//! expressible with just `cargo`.
//!
//! The binary is integrated into the `cargo` command line by using an
//! alias in `.cargo/config`.

mod cli;
mod run;
mod build;

use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::App::parse();

    match &cli.command {
        Some(cli::Commands::Run { name }) => {
            run::script(name)?;
            return Ok(());
        }
        Some(cli::Commands::Build { name }) => {
            build::script(name);
            return Ok(());
        }
        None => {
            panic!("No command specified");
        }
    }
}
