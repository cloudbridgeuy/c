use std::env;
use std::fs;
use std::path;

use color_eyre::eyre::Result;
use gpt_tokenizer::Default as DefaultTokenizer;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::message::HistoryMessage;
use super::message::Message;
use super::message::Role;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Options {
    pub model: String,
    /// The maximum number of tokens to generate in the completion.
    pub max_tokens: Option<u32>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the
    /// output more random, while lower valies like 0.2 will make it more focused and
    /// deterministic. It's generally recommended to alter this or `top_p` but not both.
    pub temperature: Option<f32>,
    /// An alternative sampling with temperature, called nucleus sampling, where the model
    /// considers the results of the tokens with `top_p` probability mass. So, 0.1 means only
    /// the tokens comprising the top 10% probability mass are considered. It's generally
    /// recommended to alter this or `temperature` but not both.
    pub top_p: Option<f32>,
    /// How many completions to generate for each prompt.
    pub n: Option<u32>,
    /// Up to 4 sequences where the API will stop generating further tokens. The returned text
    /// will not contain the stop sequence.
    pub stop: Option<Vec<String>>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they
    /// appear in the text so far, increasing the model's likelihood to talk about new topics.
    pub presence_penalty: Option<f32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their
    /// existing frequency in the text so far, decreasing the model's likelihood to repeat the
    /// same line verbatim.
    pub frequency_penalty: Option<f32>,
    /// Modify the likelihood of specified tokens appearing in the completion.
    pub logit_bias: Option<Vec<(u32, f32)>>,
    /// A user identifier representing your end-user, which can help OpenAI to monitor and
    /// detect abuse.
    pub user: Option<String>,
    /// The minimum available tokens left to the Model to construct the completion message.
    pub min_available_tokens: Option<u32>,
    /// The maximum number of tokens supporte by the model.
    pub max_supported_tokens: Option<u32>,
    /// A list of functions the model may generate JSON inputs for, provided as JSON.
    pub functions: Option<String>,
    /// Controls how the model responds to function calls. "none" means the model does not call
    /// a function, and responds to the end-user. "auto" means the model can pick between an
    /// end-user or calling a function. Specifying a particular function via `{"name":
    /// "my_function" }` forces the model to call that function. "none" is the default when no
    /// functions are present. "auto" is the default if functions are present.
    pub function_call: Option<String>,
}

/// Important data that depends on the command invocation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Meta {
    path: String,
    pub silent: bool,
    pub stream: bool,
    pub pin: bool,
    pub key: String,
    pub format: crate::Output,
}

impl Meta {
    pub fn new(path: String) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }
}

/// Represents a chat session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    id: String,
    pub messages: Vec<HistoryMessage>,
    pub options: Options,
    #[serde(skip)]
    pub meta: Meta,
}

impl Session {
    /// Creates a new session
    fn new(id: String, path: String) -> Self {
        Self {
            id,
            meta: Meta::new(path),
            ..Default::default()
        }
    }

    /// Creates a new session from the anthropic command options.
    pub fn from(options: super::Options) -> Result<Self> {
        let mut session = if let Some(id) = options.session.clone() {
            Self::load(id)?
        } else {
            let id = Ulid::new().to_string();
            let home = env::var("C_ROOT").unwrap_or(env::var("HOME")?);
            let path = format!("{home}/.c/sessions/anonymous/{id}.yaml");
            Self::new(id, path)
        };

        session.merge_options(options)?;

        Ok(session)
    }

    /// Tries to load a session from the filesystem.
    pub fn load(id: String) -> Result<Self> {
        let home = env::var("C_ROOT").unwrap_or(env::var("HOME")?);
        let path = format!("{home}/.c/sessions/{id}.yaml");

        let meta = Meta {
            path: path.clone(),
            ..Default::default()
        };

        let session = if fs::metadata(&path).is_ok() {
            let mut session: Session = serde_yaml::from_str(&fs::read_to_string(&path)?)?;
            session.meta = meta;
            session
        } else {
            Self::new(id, path)
        };

        Ok(session)
    }

    /// Merges an options object into the session options.
    pub fn merge_options(&mut self, options: super::Options) -> Result<()> {
        if options.model.is_some() {
            self.options.model = options.model.unwrap().as_str().to_string();
        }

        if options.max_tokens.is_some() {
            self.options.max_tokens = options.max_tokens;
        }

        if options.max_supported_tokens.is_some() {
            self.options.max_supported_tokens = options.max_supported_tokens;
        }

        if options.temperature.is_some() {
            self.options.temperature = options.temperature;
        }

        if options.top_p.is_some() {
            self.options.top_p = options.top_p;
        }

        if options.stop.is_some() {
            self.options.stop = options.stop;
        }

        if options.presence_penalty.is_some() {
            self.options.presence_penalty = options.presence_penalty;
        }

        if options.frequency_penalty.is_some() {
            self.options.frequency_penalty = options.frequency_penalty;
        }

        if options.logit_bias.is_some() {
            self.options.logit_bias = options.logit_bias;
        }

        if options.user.is_some() {
            self.options.user = options.user;
        }

        if options.min_available_tokens.is_some() {
            self.options.min_available_tokens = options.min_available_tokens;
        }

        if options.format.is_some() {
            self.meta.format = options.format.unwrap();
        }

        self.meta.key = options.openai_api_key;
        self.meta.stream = options.stream;
        self.meta.silent = options.silent;
        self.meta.pin = options.pin;

        Ok(())
    }

    /// Saves the session to the filesystem.
    pub fn save(&self) -> Result<()> {
        tracing::event!(
            tracing::Level::INFO,
            "saving session to {:?}",
            self.meta.path
        );
        let parent = path::Path::new(&self.meta.path)
            .parent()
            .unwrap()
            .to_str()
            .unwrap();

        if !directory_exists(parent) {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.meta.path, serde_yaml::to_string(&self)?)?;
        Ok(())
    }

    /// Returns a valid list of messages for the completion to work.
    pub fn complete_messages(&self) -> Result<Vec<Message>> {
        let tokenizer = DefaultTokenizer::new();
        let min = std::cmp::max(
            self.options.min_available_tokens.unwrap_or(1000),
            self.options.max_tokens.unwrap_or(0),
        );
        let max = self.options.max_supported_tokens.unwrap_or(4096) - min;
        let messages = trim_messages(self.messages.clone(), max, &tokenizer)?;

        Ok(messages)
    }
}

/// Trim messages until the total number of tokens inside is less than the maximum.
fn trim_messages(
    mut messages: Vec<HistoryMessage>,
    max: u32,
    tokenizer: &DefaultTokenizer,
) -> Result<Vec<Message>> {
    let total_tokens: usize = messages
        .iter()
        .map(|m| tokenizer.encode(&m.content).len())
        .sum();

    if total_tokens as u32 <= max {
        let messages: Vec<Message> = messages.into_iter().map(Message::from).collect();

        return Ok(messages);
    }

    if let Some((index, _)) = messages
        .iter()
        .enumerate()
        .find(|(_, m)| m.role != Role::System && !m.pin)
    {
        messages.remove(index);
        trim_messages(messages, max, tokenizer)
    } else {
        Err(color_eyre::eyre::format_err!(
            "Could not trim messages to fit the maximum number of tokens."
        ))
    }
}

/// Chacks if a directory exists.
pub fn directory_exists(dir_name: &str) -> bool {
    let p = path::Path::new(dir_name);
    p.exists() && p.is_dir()
}
