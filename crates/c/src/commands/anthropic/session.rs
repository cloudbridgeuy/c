use std::env;
use std::fs;
use std::path;

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::message::{Message, Role};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Options {
    /// The maximum number of tokens supported by the model.
    pub max_supported_tokens: u32,
    /// Controls which version of Claude answers your request. Two model families are exposed
    /// Claude and Claude Instant.
    pub model: String,
    /// A maximum number of tokens to generate before stopping.
    pub max_tokens_to_sample: Option<u32>,
    /// Claude models stop on `\n\nHuman:`, and may include additional built-in stops sequences
    /// in the future. By providing the `stop_sequences` parameter, you may include additional
    /// strings that will cause the model to stop generation.
    pub stop_sequences: Option<Vec<String>>,
    /// Amount of randomness injected into the response. Ranges from 0 to 1. Use temp closer to
    /// 0 for analytical/multiple choice, and temp closer to 1 for creative and generative
    /// tasks.
    pub temperature: Option<f32>,
    /// Only sample fromt the top `K` options of each subsequent token. Used to remove "long
    /// tail" low probability responses. Defaults to -1, which disables it.
    pub top_k: Option<f32>,
    /// Does nucleus sampleing, in which we compute the cumulative distribution over all the
    /// options for each subsequent token in decreasing probability order and cut it off once
    /// it reaches a particular probability specified by the top_p. Defaults to -1, which
    /// disables it. Not that you should either alter *temperature* or *top_p* but not both.
    pub top_p: Option<f32>,
}

/// Important data that depends on the command invocation.
#[derive(Debug, Default, Serialize, Deserialize)]
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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Session {
    id: String,
    pub messages: Vec<Message>,
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

        if options.max_tokens_to_sample.is_some() {
            self.options.max_tokens_to_sample = options.max_tokens_to_sample;
        }

        if options.max_supported_tokens.is_some() {
            self.options.max_supported_tokens = options.max_supported_tokens.unwrap();
        }

        if options.temperature.is_some() {
            self.options.temperature = options.temperature;
        }

        if options.top_k.is_some() {
            self.options.top_k = options.top_k;
        }

        if options.top_p.is_some() {
            self.options.top_p = options.top_p;
        }

        if options.stop_sequences.is_some() {
            self.options.stop_sequences = options.stop_sequences;
        }

        if options.format.is_some() {
            self.meta.format = options.format.unwrap();
        }

        self.meta.key = options.anthropic_api_key;
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

    /// Returns a valid completion prompt from the list of messages.
    pub fn complete_prompt(&self) -> Result<String> {
        let max =
            self.options.max_supported_tokens - self.options.max_tokens_to_sample.unwrap_or(1000);
        let mut messages = self.messages.clone();
        messages.push(Message::new("".to_string(), Role::Assistant, false));

        tracing::event!(
            tracing::Level::INFO,
            "Creating a complete prompt that is less than {max} tokens long"
        );

        let prompt = loop {
            let prompt = join_messages(&messages);
            let tokens = token_length(&prompt) as u32;

            if tokens <= max {
                tracing::event!(
                    tracing::Level::INFO,
                    "Tokens ({tokens}) is less than max ({max}). Returning prompt",
                );
                break prompt;
            }

            if let Some((index, _)) = messages.iter().enumerate().find(|(_, m)| !m.pin) {
                tracing::event!(
                    tracing::Level::INFO,
                    "Tokens ({tokens}) is greater than max ({max}). Trying again with fewer messages",
                );
                messages.remove(index);
            } else {
                Err(color_eyre::eyre::format_err!(
                    "The prompt is larger than {max} and there are no messages to remove"
                ))?;
            }
        };

        Ok(prompt)
    }
}

/// Join messages
fn join_messages(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|m| format!("\n\n{:?}: {}", m.role, m.content))
        .collect::<Vec<String>>()
        .join("")
}

/// Chacks if a directory exists.
pub fn directory_exists(dir_name: &str) -> bool {
    let p = path::Path::new(dir_name);
    p.exists() && p.is_dir()
}

/// Token language of a prompt.
/// TODO: Make this better!
fn token_length(prompt: &str) -> usize {
    let words = prompt.split_whitespace().rev().collect::<Vec<&str>>();

    // Estimate the total tokens by multiplying words by 4/3
    words.len() * 4 / 3
}
