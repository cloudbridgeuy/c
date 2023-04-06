use openai::chats::Chat;
use openai::completions::Completions;
use openai::error::OpenAi as OpenAiError;
use serde::Serialize;

use crate::chats::ChatsCreateCommand;
use crate::completions::CompletionsCreateCommand;
use crate::{CommandHandle, CommandResult};

pub enum CommandCallers {
    ChatsCreate(ChatsCreateCommand),
    CompletionsCreate(CompletionsCreateCommand),
}

#[derive(Serialize)]
pub enum CommandResults {
    ChatsCreate(Chat),
    CompletionsCreate(Completions),
}

impl CommandCallers {
    pub fn call(self) -> Result<CommandResults, OpenAiError> {
        match self {
            CommandCallers::CompletionsCreate(command) => match command.call() {
                Ok(result) => Ok(CommandResults::CompletionsCreate(result)),
                Err(err) => Err(err),
            },
            CommandCallers::ChatsCreate(command) => match command.call() {
                Ok(result) => Ok(CommandResults::ChatsCreate(result)),
                Err(err) => Err(err),
            },
        }
    }
}

impl CommandResult for CommandResults {
    fn print_json(&self) -> Result<(), OpenAiError> {
        match self {
            CommandResults::ChatsCreate(result) => result.print_json(),
            CommandResults::CompletionsCreate(result) => result.print_json(),
        }
    }

    fn print_yaml(&self) -> Result<(), OpenAiError> {
        match self {
            CommandResults::ChatsCreate(result) => result.print_yaml(),
            CommandResults::CompletionsCreate(result) => result.print_yaml(),
        }
    }

    fn print_raw(&self) -> Result<(), OpenAiError> {
        match self {
            CommandResults::ChatsCreate(result) => result.print_raw(),
            CommandResults::CompletionsCreate(result) => result.print_raw(),
        }
    }
}
