use std::collections::HashMap;
use std::error::Error;
use tokio_stream::StreamExt;

use async_trait::async_trait;
use serde_either::SingleOrVec;
use serde_json::from_str;

use openai::chats::{Chat, ChatMessage, ChatsApi};
use openai::error::OpenAi as OpenAiError;

use crate::utils::{read_from_stdin, Spinner};
use crate::{ChatsCommands, Cli, CommandError, CommandHandle, CommandResult};

pub struct ChatsCreateCommand {
    pub api: ChatsApi,
}

impl ChatsCreateCommand {
    pub fn new(cli: &Cli, command: &ChatsCommands) -> Result<Self, Box<dyn Error>> {
        match command {
            ChatsCommands::Create {
                model,
                session,
                prompt,
                system,
                max_tokens,
                temperature,
                top_p,
                n,
                stop,
                presence_penalty,
                frequency_penalty,
                user,
                logit_bias,
                min_available_tokens,
                max_supported_tokens,
                functions,
                function_call,
            } => {
                let api_key = cli
                    .openai_api_key
                    .as_ref()
                    .expect("No API Key provided")
                    .to_string();

                log::debug!("Session: {:?}", session);
                let mut api = if let Some(s) = session {
                    ChatsApi::new_with_session(api_key, s.to_owned())?
                } else {
                    ChatsApi::new(api_key)?
                };

                let message = match prompt {
                    Some(s) if s == "-" => ChatMessage {
                        content: Some(read_from_stdin()?),
                        role: "user".to_owned(),
                        ..Default::default()
                    },
                    Some(s) => ChatMessage {
                        content: Some(s.to_owned()),
                        role: "user".to_owned(),
                        ..Default::default()
                    },
                    None => ChatMessage {
                        content: Some("".to_owned()),
                        role: "user".to_owned(),
                        ..Default::default()
                    },
                };

                api.messages.push(message);

                if let Some(s) = system {
                    if api.messages.first().unwrap().role == "system" {
                        api.messages.remove(0);
                    }
                    api.messages.insert(
                        0,
                        ChatMessage {
                            content: Some(s.to_owned()),
                            role: "system".to_owned(),
                            ..Default::default()
                        },
                    );
                }

                if let Some(m) = model {
                    api.model = m.to_owned();
                }

                if let Some(f) = functions {
                    api.functions = match from_str(f) {
                        Ok(f) => f,
                        Err(e) => {
                            log::error!("Error parsing functions: {}", e);
                            return Err(Box::new(e));
                        }
                    };
                }

                if let Some(f) = function_call {
                    api.function_call = match from_str(f) {
                        Ok(f) => f,
                        Err(e) => {
                            log::error!("Error parsing function_call: {}", e);
                            return Err(Box::new(e));
                        }
                    };
                }

                log::debug!("Using model: {:?}", api.model);

                max_tokens.map(|s| api.max_tokens = Some(s));
                n.map(|s| api.n = Some(s));
                temperature.map(|s| api.set_temperature(s));
                top_p.map(|s| api.set_top_p(s));
                presence_penalty.map(|s| api.set_presence_penalty(s));
                frequency_penalty.map(|s| api.set_frequency_penalty(s));
                min_available_tokens.map(|s| api.min_available_tokens = Some(s));
                max_supported_tokens.map(|s| api.max_supported_tokens = Some(s));

                api.stream = Some(cli.stream);

                if &api.user != user {
                    api.user = user.to_owned();
                }

                stop.as_ref()
                    .map(|s| api.set_stop(SingleOrVec::Vec(s.to_vec())));

                if let Some(logit_bias) = logit_bias {
                    let mut map = api.logit_bias.unwrap_or(HashMap::new());
                    for (key, value) in logit_bias {
                        map.insert(key.to_owned(), *value);
                    }
                    api.logit_bias = Some(map);
                }

                Ok(Self { api })
            }
        }
    }
}

impl CommandResult for Chat {
    type ResultError = CommandError;

    fn print_raw<W: std::io::Write>(&self, mut w: W) -> Result<(), Self::ResultError> {
        match self.choices.first() {
            Some(choice) => {
                if let Some(content) = &choice.message.content {
                    writeln!(w, "{}", content)?;
                } else if let Some(fc) = &choice.message.function_call {
                    writeln!(w, "{}", fc.arguments)?;
                }
                Ok(())
            }
            None => Err(CommandError::from(OpenAiError::NoChoices)),
        }
    }
}

#[async_trait]
impl CommandHandle<Chat> for ChatsCreateCommand {
    type CallError = OpenAiError;

    async fn call(&self) -> Result<Chat, OpenAiError> {
        let mut spinner = Spinner::new(false);

        log::debug!("Stream is: {:?}", self.api.stream);

        if Some(true) == self.api.stream {
            log::debug!("Creating stream");

            let chunks = match self.api.create_stream().await {
                Ok(chunks) => chunks,
                Err(e) => {
                    log::error!("Error creating stream: {}", e);
                    return Err(OpenAiError::StreamError);
                }
            };

            tokio::pin!(chunks);

            while let Some(chunk) = chunks.next().await {
                if chunk.is_err() {
                    log::error!("Error reading stream");
                    spinner.err("Error reading stream");
                    return Err(OpenAiError::StreamError);
                }

                // spinner.ok();

                let chunk = chunk.unwrap();

                if let Some(choice) = chunk.choices.get(0) {
                    if let Some(delta) = &choice.delta {
                        if let Some(content) = &delta.content {
                            // print!("{}", content);
                            spinner.print(content);
                        }
                    }
                }
            }

            spinner.ok();
            Ok(openai::chats::Chat::default())
        } else {
            log::debug!("Creating chat");
            self.api.create().await
        }
    }
}
