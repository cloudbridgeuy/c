use clap::Parser;

use b::chats::ChatsCreateCommand;
use b::commands::CommandCallers;
use b::completions::CompletionsCreateCommand;
use b::edits::EditsCreateCommand;
use b::tokenizer::{TokenizerDecodeCommand, TokenizerEncodeCommand};
use b::utils::Spinner;
use b::{Cli, CommandError, CommandResult, Commands, Output, TokenizerCommands};

#[tokio::main]
async fn main() -> Result<(), CommandError> {
    env_logger::init();

    let cli = Cli::parse();

    let command = match cli.command {
        Some(Commands::Chats { ref command }) => CommandCallers::ChatsCreate(
            ChatsCreateCommand::new(&cli, command).expect("Failed to parse command"),
        ),
        Some(Commands::Edits { ref command }) => CommandCallers::EditsCreate(
            EditsCreateCommand::new(&cli, command).expect("Failed to parse command"),
        ),
        Some(Commands::Completions { ref command }) => CommandCallers::CompletionsCreate(
            CompletionsCreateCommand::new(&cli, command).expect("Failed to parse command"),
        ),
        Some(Commands::Tokenizer { ref command }) => match command {
            TokenizerCommands::Encode { ref prompt } => CommandCallers::TokenizerEncode(
                TokenizerEncodeCommand::new(&cli, prompt.to_string()),
            ),
            TokenizerCommands::Decode { ref encoded } => {
                CommandCallers::TokenizerDecode(TokenizerDecodeCommand::new(&cli, encoded.to_vec()))
            }
        },
        None => {
            std::process::exit(1);
        }
    };

    let spinner = Spinner::new(cli.silent);

    let result = match command.call().await {
        Ok(result) => {
            spinner.ok();
            result
        }
        Err(e) => {
            spinner.err(&e.to_string());
            std::process::exit(1);
        }
    };

    match cli.output {
        Output::Json => {
            result.print_json(std::io::stdout())?;
        }
        Output::Yaml => {
            result.print_yaml(std::io::stdout())?;
        }
        Output::Raw => {
            result.print_raw(std::io::stdout())?;
        }
    }

    Ok(())
}
