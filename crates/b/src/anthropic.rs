use std::error::Error;

use async_trait::async_trait;

use anthropic::complete::{Api, Model, Response};

use crate::utils::read_from_stdin;
use crate::{AnthropicCommands, Cli, CommandError, CommandHandle, CommandResult};

pub struct CompleteCreateCommand {
    pub api: Api,
}

impl CompleteCreateCommand {
    pub fn new(cli: &Cli, command: &AnthropicCommands) -> Result<Self, Box<dyn Error>> {
        match command {
            AnthropicCommands::Create {
                prompt,
                system,
                model,
                max_tokens_to_sample,
                stop_sequences,
                stream,
                temperature,
                top_k,
                top_p,
                session,
                max_supported_tokens,
            } => {
                let api_key = cli
                    .anthropic_api_key
                    .as_ref()
                    .expect("api key not set")
                    .to_string();

                let mut api = if let Some(s) = session {
                    log::debug!("loading session {}", s);
                    Api::new_with_session(api_key, s.to_owned())?
                } else {
                    log::debug!("creating new session");
                    Api::new(api_key)?
                };

                api.prompt.push_str("\n\nHuman: ");

                if prompt == "-" {
                    let stdin = read_from_stdin()?;
                    api.prompt.push_str(&stdin);
                } else {
                    api.prompt.push_str(prompt);
                }

                if let Some(m) = model {
                    let model: Model = Model::from(*m);

                    if api.model as u32 != model as u32 {
                        api.model = model;
                    };
                }

                log::debug!("model: {:?}", api.model);

                if system.is_some() {
                    api.system = system.clone();
                }

                max_tokens_to_sample.map(|s| api.max_tokens_to_sample = Some(s));
                max_supported_tokens.map(|s| api.max_supported_tokens = Some(s));
                stream.map(|s| api.stream = Some(s));
                temperature.map(|s| api.set_temperature(s));
                top_k.map(|s| api.set_top_k(s));
                top_p.map(|s| api.set_top_p(s));

                if let Some(s) = stop_sequences {
                    api.stop_sequences = Some(s.to_vec());
                }

                Ok(Self { api })
            }
        }
    }
}

impl CommandResult for Response {
    type ResultError = CommandError;

    fn print_raw<W: std::io::Write>(&self, mut w: W) -> Result<(), Self::ResultError> {
        writeln!(w, "{}", self.completion)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CompleteCreateCommandError;

impl std::error::Error for CompleteCreateCommandError {}

impl std::fmt::Display for CompleteCreateCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bad :(")
    }
}

#[async_trait]
impl CommandHandle<Response> for CompleteCreateCommand {
    type CallError = CommandError;

    async fn call(&self) -> Result<Response, Self::CallError> {
        match self.api.create().await {
            Ok(response) => Ok(response),
            Err(e) => Err(CommandError::AnthropicError {
                body: e.to_string(),
            }),
        }
    }
}
