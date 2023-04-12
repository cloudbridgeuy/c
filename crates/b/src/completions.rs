use std::error::Error;

use async_trait::async_trait;
use serde_either::SingleOrVec;

use openai::completions::{Completions, CompletionsApi};
use openai::error::OpenAi as OpenAiError;

use crate::{Cli, CommandError, CommandHandle, CommandResult, CompletionsCommands};

pub struct CompletionsCreateCommand {
    pub api: CompletionsApi,
}

impl CompletionsCreateCommand {
    pub fn new(cli: &Cli, command: &CompletionsCommands) -> Result<Self, Box<dyn Error>> {
        match command {
            CompletionsCommands::Create {
                model,
                prompt,
                suffix,
                max_tokens,
                temperature,
                top_p,
                n,
                stream,
                logprobs,
                echo,
                stop,
                presence_penalty,
                frequency_penalty,
                best_of,
                user,
            } => {
                let api_key = cli
                    .api_key
                    .as_ref()
                    .expect("No API key provided")
                    .to_string();
                let mut api = CompletionsApi::new(api_key)?;
                api.prompt = Some(SingleOrVec::Vec(prompt.clone()));
                api.model = model.to_string();
                api.max_tokens = *max_tokens;
                api.n = *n;
                user.as_ref().map(|s| api.user = Some(s.to_string()));

                echo.map(|s| api.set_echo(s));
                suffix.as_ref().map(|s| api.set_suffix(s.to_string()));
                stream.map(|s| api.set_stream(s));
                logprobs.map(|s| api.set_logprobs(s));
                stop.as_ref()
                    .map(|s| api.set_stop(SingleOrVec::Vec(s.to_vec())));
                presence_penalty.map(|s| api.set_presence_penalty(s));
                frequency_penalty.map(|s| api.set_frequency_penalty(s));
                best_of.map(|s| api.set_best_of(s));
                temperature.map(|s| api.set_temperature(s));
                top_p.map(|s| api.set_top_p(s));

                Ok(Self { api })
            }
        }
    }
}

impl CommandResult for Completions {
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
impl CommandHandle<Completions> for CompletionsCreateCommand {
    type CallError = OpenAiError;

    async fn call(&self) -> Result<Completions, OpenAiError> {
        self.api.create().await
    }
}
