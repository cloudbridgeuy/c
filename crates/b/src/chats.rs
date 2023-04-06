use std::error::Error;

use serde_either::SingleOrVec;

use openai::chats::{Chat, ChatMessage, ChatsApi};
use openai::error::OpenAi as OpenAiError;

use crate::{ChatsCommands, Cli, CommandHandle, CommandResult};

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
                api.messages = vec![ChatMessage {
                    content: prompt.to_owned().join(" "),
                    role: "user".to_owned(),
                }];
                api.model = model.to_owned();
                api.max_tokens = max_tokens.to_owned();
                api.n = *n;
                api.user = user.to_owned();
                api.stream = stream.to_owned();

                if let Some(stop) = stop {
                    api.set_stop(SingleOrVec::Vec(stop.to_owned()))?;
                }
                if let Some(temperature) = temperature {
                    api.set_temperature(*temperature)?;
                }
                if let Some(top_p) = top_p {
                    api.set_top_p(*top_p)?;
                }
                if let Some(presence_penalty) = presence_penalty {
                    api.set_presence_penalty(*presence_penalty)?;
                }
                if let Some(frequency_penalty) = frequency_penalty {
                    api.set_frequency_penalty(*frequency_penalty)?;
                }

                Ok(Self { api })
            }
        }
    }
}

impl CommandResult for Chat {
    fn print_raw(&self) -> Result<(), OpenAiError> {
        match self.choices.first() {
            Some(choice) => {
                println!("{}", choice.message.content);
                Ok(())
            }
            None => Err(OpenAiError::NoChoices),
        }
    }
}

impl CommandHandle<Chat> for ChatsCreateCommand {
    type CallError = OpenAiError;

    fn call(&self) -> Result<Chat, OpenAiError> {
        self.api.create()
    }
}
