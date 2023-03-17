use clap::{Args, Parser, Subcommand};

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
    Run(RunArgs),
    /// Builds one of the project binaries
    Build(BuildArgs),
    /// Builds a binary and installs it at the given path
    Install(InstallArgs),
    /// Publishes a package to crates.io
    Publish(PublishArgs),
    /// Creates a new GitHub release
    Github(GithubArgs),
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Name of the binary to run.
    #[arg(short, long)]
    pub name: String,
}

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Name of the binary to run.
    #[arg(short, long)]
    pub name: String,

    /// Release flag
    #[arg(short, long)]
    pub release: bool,
}

#[derive(Args, Debug)]
pub struct PublishArgs {
    /// Name of the library to publish.
    #[arg(short, long)]
    pub name: String,

    /// Dry run flag.
    #[arg(short, long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Name of the binary to run.
    #[arg(short, long)]
    pub name: String,

    /// Path to install the binary to.
    #[arg(short, long)]
    pub path: String,
}

#[derive(Args, Debug)]
pub struct GithubArgs {
    /// Name of the binary to run.
    #[arg(short, long)]
    pub name: String,
}

