use clap::Parser;

use b::chats::ChatsCreateCommand;
use b::commands::CommandCallers;
use b::completions::CompletionsCreateCommand;
use b::edits::EditsCreateCommand;
use b::utils::create_spinner;
use b::{Cli, CommandResult, Commands, Output};

#[tokio::main]
async fn main() -> Result<(), openai::error::OpenAi> {
    env_logger::init();

    let cli = Cli::parse();

    let command = match cli.command {
        Some(Commands::Chats { ref command }) => {
            let caller = ChatsCreateCommand::new(&cli, &command).expect("Failed to parse command");
            CommandCallers::ChatsCreate(caller)
        }
        Some(Commands::Edits { ref command }) => {
            let caller = EditsCreateCommand::new(&cli, &command).expect("Failed to parse command");
            CommandCallers::EditsCreate(caller)
        }
        Some(Commands::Completions { ref command }) => {
            let caller =
                CompletionsCreateCommand::new(&cli, &command).expect("Failed to parse command");
            CommandCallers::CompletionsCreate(caller)
        }
        None => {
            std::process::exit(1);
        }
    };

    // Create a spinner
    let spinner = create_spinner();

    let result = match command.call().await {
        Ok(result) => {
            spinner.finish_and_clear();
            result
        }
        Err(e) => {
            spinner.abandon_with_message(e.to_string());
            std::process::exit(1);
        }
    };

    match cli.output {
        Output::Json => {
            result.print_json()?;
        }
        Output::Yaml => {
            result.print_yaml()?;
        }
        Output::Raw => {
            result.print_raw()?;
        }
    }

    Ok(())
}
