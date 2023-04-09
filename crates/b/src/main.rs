use clap::Parser;
use log;

use b::chats::ChatsCreateCommand;
use b::commands::CommandCallers;
use b::completions::CompletionsCreateCommand;
use b::edits::EditsCreateCommand;
use b::{Cli, CommandResult, Commands, Output};

fn main() -> Result<(), openai::error::OpenAi> {
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

    let result = match command.call() {
        Ok(result) => result,
        Err(e) => {
            log::error!("{:#?}", e);
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
