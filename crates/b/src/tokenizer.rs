use async_trait::async_trait;
use gpt_tokenizer::Default as DefaultTokenizer;
use serde::Serialize;

use crate::{Cli, CommandError, CommandHandle, CommandResult};

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
    type ResultError = CommandError;

    fn print_raw<W: std::io::Write>(&self, mut w: W) -> Result<(), Self::ResultError> {
        for value in &self.value {
            write!(w, "{} ", value)?;
        }
        Ok(())
    }
}

#[async_trait]
impl CommandHandle<TokenizerEncodeResult> for TokenizerEncodeCommand {
    type CallError = CommandError;

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
    type ResultError = CommandError;

    fn print_raw<W: std::io::Write>(&self, mut w: W) -> Result<(), Self::ResultError> {
        write!(w, "{}", self.value)?;
        Ok(())
    }
}

#[async_trait]
impl CommandHandle<TokenizerDecodeResult> for TokenizerDecodeCommand {
    type CallError = CommandError;

    async fn call(&self) -> Result<TokenizerDecodeResult, Self::CallError> {
        let value = self.tokenizer.decode(&self.encoded);
        Ok(TokenizerDecodeResult { value })
    }
}
