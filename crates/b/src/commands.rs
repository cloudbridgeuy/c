use std::error::Error;

use openai::chats::Chat;
use openai::completions::Completions;
use openai::edits::Edit;
use openai::error::OpenAi as OpenAiError;
use serde::Serialize;

use crate::chats::ChatsCreateCommand;
use crate::completions::CompletionsCreateCommand;
use crate::edits::EditsCreateCommand;
use crate::tokenizer::{
    TokenizerDecodeCommand, TokenizerDecodeResult, TokenizerEncodeCommand, TokenizerEncodeResult,
    TokenizerError,
};
use crate::{CommandHandle, CommandResult};

pub enum CommandCallers {
    TokenizerDecode(TokenizerDecodeCommand),
    TokenizerEncode(TokenizerEncodeCommand),
    ChatsCreate(ChatsCreateCommand),
    EditsCreate(EditsCreateCommand),
    CompletionsCreate(CompletionsCreateCommand),
}

#[derive(Serialize)]
pub enum CommandResults {
    TokenizerDecode(TokenizerDecodeResult),
    TokenizerEncode(TokenizerEncodeResult),
    ChatsCreate(Chat),
    EditsCreate(Edit),
    CompletionsCreate(Completions),
}

#[derive(Debug)]
pub enum CommandsError {
    Tokenizer(TokenizerError),
    OpenAi(OpenAiError),
}

impl std::fmt::Display for CommandsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Error for CommandsError {}

impl From<serde_json::Error> for CommandsError {
    fn from(e: serde_json::Error) -> Self {
        Self::OpenAi(OpenAiError::SerializationError {
            body: e.to_string(),
        })
    }
}

impl From<serde_yaml::Error> for CommandsError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::OpenAi(OpenAiError::SerializationError {
            body: e.to_string(),
        })
    }
}

impl CommandCallers {
    pub async fn call(self) -> Result<CommandResults, CommandsError> {
        match self {
            CommandCallers::CompletionsCreate(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::CompletionsCreate(result)),
                Err(err) => Err(CommandsError::OpenAi(err)),
            },
            CommandCallers::ChatsCreate(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::ChatsCreate(result)),
                Err(err) => Err(CommandsError::OpenAi(err)),
            },
            CommandCallers::EditsCreate(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::EditsCreate(result)),
                Err(err) => Err(CommandsError::OpenAi(err)),
            },
            CommandCallers::TokenizerDecode(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::TokenizerDecode(result)),
                Err(err) => Err(CommandsError::Tokenizer(err)),
            },
            CommandCallers::TokenizerEncode(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::TokenizerEncode(result)),
                Err(err) => Err(CommandsError::Tokenizer(err)),
            },
        }
    }
}

impl CommandResult for CommandResults {
    type ResultError = CommandsError;

    fn print_json(&self) -> Result<(), CommandsError> {
        match self {
            CommandResults::TokenizerEncode(result) => {
                result.print_json().map_err(|e| CommandsError::Tokenizer(e))
            }
            CommandResults::TokenizerDecode(result) => {
                result.print_json().map_err(|e| CommandsError::Tokenizer(e))
            }
            CommandResults::ChatsCreate(result) => {
                result.print_json().map_err(|e| CommandsError::OpenAi(e))
            }
            CommandResults::EditsCreate(result) => {
                result.print_json().map_err(|e| CommandsError::OpenAi(e))
            }
            CommandResults::CompletionsCreate(result) => {
                result.print_json().map_err(|e| CommandsError::OpenAi(e))
            }
        }
    }

    fn print_yaml(&self) -> Result<(), CommandsError> {
        match self {
            CommandResults::TokenizerEncode(result) => {
                result.print_yaml().map_err(|e| CommandsError::Tokenizer(e))
            }
            CommandResults::TokenizerDecode(result) => {
                result.print_yaml().map_err(|e| CommandsError::Tokenizer(e))
            }
            CommandResults::ChatsCreate(result) => {
                result.print_yaml().map_err(|e| CommandsError::OpenAi(e))
            }
            CommandResults::EditsCreate(result) => {
                result.print_yaml().map_err(|e| CommandsError::OpenAi(e))
            }
            CommandResults::CompletionsCreate(result) => {
                result.print_yaml().map_err(|e| CommandsError::OpenAi(e))
            }
        }
    }

    fn print_raw(&self) -> Result<(), CommandsError> {
        match self {
            CommandResults::TokenizerEncode(result) => {
                result.print_raw().map_err(|e| CommandsError::Tokenizer(e))
            }
            CommandResults::TokenizerDecode(result) => {
                result.print_raw().map_err(|e| CommandsError::Tokenizer(e))
            }
            CommandResults::ChatsCreate(result) => {
                result.print_raw().map_err(|e| CommandsError::OpenAi(e))
            }
            CommandResults::EditsCreate(result) => {
                result.print_raw().map_err(|e| CommandsError::OpenAi(e))
            }
            CommandResults::CompletionsCreate(result) => {
                result.print_raw().map_err(|e| CommandsError::OpenAi(e))
            }
        }
    }
}
