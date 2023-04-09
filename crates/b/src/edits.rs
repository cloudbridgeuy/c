use std::error::Error;

use async_trait::async_trait;
use openai::edits::{Edit, EditsApi};
use openai::error::OpenAi as OpenAiError;

use crate::{Cli, CommandHandle, CommandResult, EditsCommands};

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
                    .api_key
                    .as_ref()
                    .expect("No API Key provided")
                    .to_string();
                let mut api = EditsApi::new(api_key)?;
                api.model = model.to_owned();
                api.instruction = instruction.to_owned();
                api.n = *n;

                input.as_ref().map(|s| api.input = s.to_owned());
                temperature.map(|s| api.set_temperature(s));
                top_p.map(|s| api.set_top_p(s));

                Ok(Self { api })
            }
        }
    }
}

impl CommandResult for Edit {
    type ResultError = OpenAiError;

    fn print_raw(&self) -> Result<(), OpenAiError> {
        match self.choices.first() {
            Some(choice) => {
                println!("{}", choice.text);
                Ok(())
            }
            None => Err(OpenAiError::NoChoices),
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
