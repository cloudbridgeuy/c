//! See <https://github.com/matklad/cargo-xtask/>
//!
//! This binary defines various auxiliary build commands, which are not
//! expressible with just `cargo`.
//!
//! The binary is integrated into the `cargo` command line by using an
//! alias in `.cargo/config`.

mod cli;
mod scripts;
mod utils;

use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::App::parse();

    match &cli.command {
        Some(command) => {
            match command {
                cli::Commands::Run(args) => {
                    scripts::run(args)
                }
                cli::Commands::Build(args) => {
                    scripts::build(args)
                }
                cli::Commands::Publish(args) => {
                    scripts::publish(args)
                }
                cli::Commands::Github(args) => {
                    scripts::github(args)
                }
                cli::Commands::Install(args) => {
                    scripts::install(args)
                }
            }
        }
        None => {
            println!("No command specified.");
            std::process::exit(1);
        }
    }
}
