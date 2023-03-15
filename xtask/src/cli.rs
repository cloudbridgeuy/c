use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name="xtasks")]
#[command(about="Run project tasks using rust instead of scripts")]
pub struct App {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Runs one of the project binaries
    Run {
        /// Name of the binary to run.
        #[arg(short, long)]
        name: String,
    },
    /// Builds one of the project binaries
    Build {
        /// Name of the binary to run.
        #[arg(short, long)]
        name: String,
    }
}

