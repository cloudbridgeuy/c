use openai::chats::Chat;
use openai::completions::Completions;
use openai::edits::Edit;
use openai::error::OpenAi as OpenAiError;
use serde::Serialize;

use crate::chats::ChatsCreateCommand;
use crate::completions::CompletionsCreateCommand;
use crate::edits::EditsCreateCommand;
use crate::{CommandHandle, CommandResult};

pub enum CommandCallers {
    ChatsCreate(ChatsCreateCommand),
    EditsCreate(EditsCreateCommand),
    CompletionsCreate(CompletionsCreateCommand),
}

#[derive(Serialize)]
pub enum CommandResults {
    ChatsCreate(Chat),
    EditsCreate(Edit),
    CompletionsCreate(Completions),
}

impl CommandCallers {
    pub async fn call(self) -> Result<CommandResults, OpenAiError> {
        match self {
            CommandCallers::CompletionsCreate(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::CompletionsCreate(result)),
                Err(err) => Err(err),
            },
            CommandCallers::ChatsCreate(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::ChatsCreate(result)),
                Err(err) => Err(err),
            },
            CommandCallers::EditsCreate(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::EditsCreate(result)),
                Err(err) => Err(err),
            },
        }
    }
}

impl CommandResult for CommandResults {
    fn print_json(&self) -> Result<(), OpenAiError> {
        match self {
            CommandResults::ChatsCreate(result) => result.print_json(),
            CommandResults::EditsCreate(result) => result.print_json(),
            CommandResults::CompletionsCreate(result) => result.print_json(),
        }
    }

    fn print_yaml(&self) -> Result<(), OpenAiError> {
        match self {
            CommandResults::ChatsCreate(result) => result.print_yaml(),
            CommandResults::EditsCreate(result) => result.print_yaml(),
            CommandResults::CompletionsCreate(result) => result.print_yaml(),
        }
    }

    fn print_raw(&self) -> Result<(), OpenAiError> {
        match self {
            CommandResults::EditsCreate(result) => result.print_raw(),
            CommandResults::ChatsCreate(result) => result.print_raw(),
            CommandResults::CompletionsCreate(result) => result.print_raw(),
        }
    }
}
