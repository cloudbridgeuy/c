use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use serde_either::SingleOrVec;

use openai::chats::{Chat, ChatMessage, ChatsApi};
use openai::error::OpenAi as OpenAiError;

use crate::utils::read_from_stdin;
use crate::{ChatsCommands, Cli, CommandError, CommandHandle, CommandResult};

pub struct ChatsCreateCommand {
    pub api: ChatsApi,
}

impl ChatsCreateCommand {
    pub fn new(cli: &Cli, command: &ChatsCommands) -> Result<Self, Box<dyn Error>> {
        match command {
            ChatsCommands::Create {
                model,
                prompt,
                system,
                max_tokens,
                temperature,
                top_p,
                n,
                stream,
                stop,
                presence_penalty,
                frequency_penalty,
                user,
                logit_bias,
            } => {
                let api_key = cli
                    .api_key
                    .as_ref()
                    .expect("No API Key provided")
                    .to_string();
                let mut api = ChatsApi::new(api_key)?;

                match prompt {
                    Some(s) if s == "-" => {
                        api.messages = vec![ChatMessage {
                            content: read_from_stdin()?,
                            role: "user".to_owned(),
                        }];
                    }
                    Some(s) => {
                        api.messages = vec![ChatMessage {
                            content: s.to_owned(),
                            role: "user".to_owned(),
                        }];
                    }
                    None => {
                        api.messages = vec![ChatMessage {
                            content: "".to_owned(),
                            role: "user".to_owned(),
                        }];
                    }
                }

                if let Some(s) = system {
                    api.messages.insert(
                        0,
                        ChatMessage {
                            content: s.to_owned(),
                            role: "system".to_owned(),
                        },
                    );
                }

                api.model = model.to_owned();
                api.max_tokens = max_tokens.to_owned();
                api.n = *n;
                api.user = user.to_owned();
                api.stream = stream.to_owned();

                stop.as_ref()
                    .map(|s| api.set_stop(SingleOrVec::Vec(s.to_vec())));
                temperature.map(|s| api.set_temperature(s));
                top_p.map(|s| api.set_top_p(s));
                presence_penalty.map(|s| api.set_presence_penalty(s));
                frequency_penalty.map(|s| api.set_frequency_penalty(s));

                if let Some(logit_bias) = logit_bias {
                    let mut map = HashMap::new();
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
                write!(w, "{}", choice.message.content)?;
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
        self.api.create().await
    }
}
