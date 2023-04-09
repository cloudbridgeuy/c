use std::error::Error;

use async_trait::async_trait;
use gpt_tokenizer::Default as DefaultTokenizer;
use serde::Serialize;

use crate::{Cli, CommandHandle, CommandResult};

#[derive(Debug)]
pub struct TokenizerError {
    message: String,
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TokenizerError {}

impl From<serde_json::Error> for TokenizerError {
    fn from(e: serde_json::Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for TokenizerError {
    fn from(e: serde_yaml::Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

pub struct TokenizerEncodeCommand {
    tokenizer: DefaultTokenizer,
    prompt: String,
}

#[derive(Serialize)]
pub struct TokenizerEncodeResult {
    pub value: Vec<u32>,
}

impl TokenizerEncodeCommand {
    pub fn new(_: &Cli, prompt: String) -> Self {
        Self {
            prompt,
            tokenizer: DefaultTokenizer::new(),
        }
    }
}

impl CommandResult for TokenizerEncodeResult {
    type ResultError = TokenizerError;

    fn print_raw(&self) -> Result<(), Self::ResultError> {
        // Implement the print_raw function for TokenizerEncodeResult
        for value in &self.value {
            print!("{} ", value);
        }
        Ok(())
    }
}

#[async_trait]
impl CommandHandle<TokenizerEncodeResult> for TokenizerEncodeCommand {
    type CallError = TokenizerError;

    async fn call(&self) -> Result<TokenizerEncodeResult, Self::CallError> {
        let value = self.tokenizer.encode(&self.prompt.to_string());
        Ok(TokenizerEncodeResult { value })
    }
}

pub struct TokenizerDecodeCommand {
    tokenizer: DefaultTokenizer,
    encoded: Vec<u32>,
}

#[derive(Serialize)]
pub struct TokenizerDecodeResult {
    pub value: String,
}

impl TokenizerDecodeCommand {
    pub fn new(_: &Cli, encoded: Vec<u32>) -> Self {
        Self {
            encoded,
            tokenizer: DefaultTokenizer::new(),
        }
    }
}

impl CommandResult for TokenizerDecodeResult {
    type ResultError = TokenizerError;

    fn print_raw(&self) -> Result<(), Self::ResultError> {
        print!("{}", self.value);
        Ok(())
    }
}

#[async_trait]
impl CommandHandle<TokenizerDecodeResult> for TokenizerDecodeCommand {
    type CallError = TokenizerError;

    async fn call(&self) -> Result<TokenizerDecodeResult, Self::CallError> {
        let value = self.tokenizer.decode(&self.encoded);
        Ok(TokenizerDecodeResult { value })
    }
}
