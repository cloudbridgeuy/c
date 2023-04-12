use std::error::Error;
use std::io::Read;

use async_trait::async_trait;
use serde_either::SingleOrVec;

use openai::chats::{Chat, ChatMessage, ChatsApi};
use openai::error::OpenAi as OpenAiError;

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
                max_tokens,
                temperature,
                top_p,
                n,
                stream,
                stop,
                presence_penalty,
                frequency_penalty,
                user,
            } => {
                let api_key = cli
                    .api_key
                    .as_ref()
                    .expect("No API Key provided")
                    .to_string();
                let mut api = ChatsApi::new(api_key)?;

                if let Some(prompt) = prompt {
                    api.messages = vec![ChatMessage {
                        content: prompt.to_owned(),
                        role: "user".to_owned(),
                    }];
                } else {
                    api.messages = vec![ChatMessage {
                        content: "".to_owned(),
                        role: "user".to_owned(),
                    }];
                }

                let mut stdin = Vec::new();
                // Read from stdin if it's not a tty and don't forget to unlock `stdin`
                {
                    let mut stdin_lock = std::io::stdin().lock();
                    stdin_lock.read_to_end(&mut stdin)?;
                }

                if !stdin.is_empty() {
                    if prompt.is_none() {
                        api.messages[0].content = String::from_utf8_lossy(&stdin).to_string();
                    } else {
                        api.messages.insert(
                            0,
                            ChatMessage {
                                content: String::from_utf8_lossy(&stdin).to_string(),
                                role: "user".to_owned(),
                            },
                        );
                    }
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