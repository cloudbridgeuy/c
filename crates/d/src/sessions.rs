use color_eyre::eyre::Result;
use openai::chat::{ChatCompletionMessage, ChatCompletionMessageRole};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Model;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    /// The id of the message.
    pub id: String,
    /// The role of the author of this message.
    pub role: ChatCompletionMessageRole,
    /// The contents of the message
    pub content: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub name: String,
    messages: Vec<Message>,
    pub model: Model,
    temperature: Option<f32>,
    top_p: Option<f32>,
    max_tokens: Option<u64>,
}

impl From<Message> for ChatCompletionMessage {
    fn from(message: Message) -> Self {
        ChatCompletionMessage {
            role: message.role,
            content: Some(message.content),
            function_call: None,
            name: None,
        }
    }
}

impl From<ChatCompletionMessage> for Message {
    fn from(message: ChatCompletionMessage) -> Self {
        Message {
            id: Uuid::new_v4().to_string(),
            role: message.role,
            content: message.content.unwrap_or_default(),
        }
    }
}

impl Session {
    pub fn new() -> Self {
        Session {
            messages: vec![Message {
                id: Uuid::new_v4().to_string(),
                role: ChatCompletionMessageRole::System,
                content: String::from(
                    "Format the response as markdown without enclosing backticks.",
                ),
            }],
            ..Default::default()
        }
    }

    /// Tries to save the session to the filesystem.
    pub fn save(&self) -> Result<()> {
        if self.name.is_empty() {
            return Ok(());
        }

        let home = std::env::var("D_ROOT").unwrap_or(std::env::var("HOME")?);
        let dir = format!("{home}/.d/sessions");
        let path = format!("{home}/.d/sessions/{}.yaml", &self.name.clone());

        if !std::path::Path::new(&dir).is_dir() {
            std::fs::create_dir_all(dir)?;
        }

        std::fs::write(path, serde_yaml::to_string(&self)?)?;

        Ok(())
    }

    /// Tries to load a session from the filesystem.
    pub fn load(name: String) -> Result<Self> {
        let home = std::env::var("D_ROOT").unwrap_or(std::env::var("HOME")?);
        let path = format!("{home}/.d/sessions/{name}.yaml");

        Ok(match std::fs::metadata(&path) {
            Ok(_) => {
                let content = std::fs::read_to_string(&path)?;
                let session: Session = serde_yaml::from_str(&content)?;
                session
            }
            Err(_) => {
                let mut session = Session::new();
                session.name = name;
                session
            }
        })
    }

    /// Updates the session system prompt.
    pub fn system(&mut self, system: String) {
        // Remove the first element of the messages if they have
        // `ChatCompletionMessageRole::System` as `role`.
        self.messages
            .retain(|message| !matches!(message.role, ChatCompletionMessageRole::System));
        // Set the first element of the vector to be the system message.
        self.messages.insert(
            0,
            Message {
                id: Uuid::new_v4().to_string(),
                role: ChatCompletionMessageRole::System,
                content: system,
            },
        );
    }

    /// Pushes a new message into the sessions message vector.
    pub fn push(&mut self, content: String, role: ChatCompletionMessageRole) {
        self.messages.push(Message {
            id: Uuid::new_v4().to_string(),
            role,
            content,
        });
    }

    /// Converts the Vec<Message> into a cloned Vec<ChatCompletionMessage>
    pub fn completion_messages(&self) -> Vec<ChatCompletionMessage> {
        self.messages
            .clone()
            .into_iter()
            .map(|message| message.into())
            .collect()
    }

    /// Gets the temperature.
    pub fn get_temperature(&self) -> f32 {
        self.temperature.unwrap_or(1.0)
    }

    /// Sets the temperature.
    pub fn set_temperature(&mut self, temperature: f32) -> Result<()> {
        match temperature {
            temperature if !(0.0..=2.0).contains(&temperature) => Err(
                color_eyre::eyre::format_err!("Temperature must be between 0 and 2"),
            ),
            _ => {
                self.temperature = Some(temperature);
                Ok(())
            }
        }
    }

    /// Gets the top_p
    pub fn get_top_p(&self) -> f32 {
        self.top_p.unwrap_or(1.0)
    }

    /// Sets the top_p
    pub fn set_top_p(&mut self, top_p: f32) -> Result<()> {
        match top_p {
            top_p if !(0.0..=1.0).contains(&top_p) => Err(color_eyre::eyre::format_err!(
                "top_p must be between 0 and 1"
            )),
            _ => {
                self.top_p = Some(top_p);
                Ok(())
            }
        }
    }

    /// Gets the max_tokens
    pub fn get_max_tokens(&self) -> u64 {
        self.max_tokens.unwrap_or(2048)
    }

    /// Sets the max_tokens
    pub fn set_max_tokens(&mut self, max_tokens: u64) -> Result<()> {
        self.max_tokens = Some(max_tokens);
        Ok(())
    }
}
