use std::error::Error;

use async_trait::async_trait;
use openai::edits::{Edit, EditsApi};
use openai::error::OpenAi as OpenAiError;

use crate::{Cli, CommandError, CommandHandle, CommandResult, EditsCommands};

pub struct EditsCreateCommand {
    pub api: EditsApi,
}

impl EditsCreateCommand {
    pub fn new(cli: &Cli, command: &EditsCommands) -> Result<Self, Box<dyn Error>> {
        match command {
            EditsCommands::Create {
                model,
                input,
                instruction,
                n,
                temperature,
                top_p,
            } => {
                let api_key = cli
                    .openai_api_key
                    .as_ref()
                    .expect("No API Key provided")
                    .to_string();
                let mut api = EditsApi::new(api_key)?;
                api.model = model.to_owned();
                api.instruction = instruction.to_owned();
                api.n = *n;

                if let Some(input) = input.as_ref() {
                    api.input = input.to_owned();
                }

                temperature.map(|s| api.set_temperature(s));
                top_p.map(|s| api.set_top_p(s));

                Ok(Self { api })
            }
        }
    }
}

impl CommandResult for Edit {
    type ResultError = CommandError;

    fn print_raw<W: std::io::Write>(&self, mut w: W) -> Result<(), Self::ResultError> {
        match self.choices.first() {
            Some(choice) => {
                write!(w, "{}", choice.text)?;
                Ok(())
            }
            None => Err(CommandError::from(OpenAiError::NoChoices)),
        }
    }
}

#[async_trait]
impl CommandHandle<Edit> for EditsCreateCommand {
    type CallError = OpenAiError;

    async fn call(&self) -> Result<Edit, OpenAiError> {
        self.api.create().await
    }
}
