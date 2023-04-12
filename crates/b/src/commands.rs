use openai::chats::Chat;
use openai::completions::Completions;
use openai::edits::Edit;
use serde::Serialize;

use crate::chats::ChatsCreateCommand;
use crate::completions::CompletionsCreateCommand;
use crate::edits::EditsCreateCommand;
use crate::tokenizer::{
    TokenizerDecodeCommand, TokenizerDecodeResult, TokenizerEncodeCommand, TokenizerEncodeResult,
};
use crate::{CommandError, CommandHandle, CommandResult};

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

impl CommandCallers {
    pub async fn call(self) -> Result<CommandResults, CommandError> {
        match self {
            CommandCallers::CompletionsCreate(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::CompletionsCreate(result)),
                Err(err) => Err(CommandError::from(err)),
            },
            CommandCallers::ChatsCreate(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::ChatsCreate(result)),
                Err(err) => Err(CommandError::from(err)),
            },
            CommandCallers::EditsCreate(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::EditsCreate(result)),
                Err(err) => Err(CommandError::from(err)),
            },
            CommandCallers::TokenizerDecode(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::TokenizerDecode(result)),
                Err(err) => Err(err),
            },
            CommandCallers::TokenizerEncode(command) => match command.call().await {
                Ok(result) => Ok(CommandResults::TokenizerEncode(result)),
                Err(err) => Err(err),
            },
        }
    }
}

impl CommandResult for CommandResults {
    type ResultError = CommandError;

    fn print_json<W: std::io::Write>(&self, w: W) -> Result<(), Self::ResultError> {
        match self {
            CommandResults::TokenizerEncode(result) => result.print_json(w),
            CommandResults::TokenizerDecode(result) => result.print_json(w),
            CommandResults::ChatsCreate(result) => result.print_json(w),
            CommandResults::EditsCreate(result) => result.print_json(w),
            CommandResults::CompletionsCreate(result) => result.print_json(w),
        }
    }

    fn print_yaml<W: std::io::Write>(&self, w: W) -> Result<(), Self::ResultError> {
        match self {
            CommandResults::TokenizerEncode(result) => result.print_yaml(w),
            CommandResults::TokenizerDecode(result) => result.print_yaml(w),
            CommandResults::ChatsCreate(result) => result.print_yaml(w),
            CommandResults::EditsCreate(result) => result.print_yaml(w),
            CommandResults::CompletionsCreate(result) => result.print_yaml(w),
        }
    }

    fn print_raw<W: std::io::Write>(&self, w: W) -> Result<(), Self::ResultError> {
        match self {
            CommandResults::TokenizerEncode(result) => result.print_raw(w),
            CommandResults::TokenizerDecode(result) => result.print_raw(w),
            CommandResults::ChatsCreate(result) => result.print_raw(w),
            CommandResults::EditsCreate(result) => result.print_raw(w),
            CommandResults::CompletionsCreate(result) => result.print_raw(w),
        }
    }
}
