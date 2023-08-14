use std::env;
use std::fs;
use std::path;

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

/// Chat LLM Vendor
#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Vendor {
    #[default]
    OpenAI,
    Anthropic,
    Google,
}

/// Chat LLM Role
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    #[default]
    Human,
    User,
    Assistant,
    System,
}

/// Represents a chat message
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Message {
    pub content: String,
    pub role: Role,
    pub pin: bool,
}

impl Message {
    /// Creates a new message
    pub fn new(content: String, role: Role, pin: bool) -> Self {
        Self { content, role, pin }
    }
}

/// Important data that are provided on each invocation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Meta {
    path: String,
    pub silent: bool,
    pub stream: bool,
    pub pin: bool,
    pub key: String,
    pub format: crate::Output,
}

/// Represents a chat session
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Session<T: Default> {
    id: String,
    vendor: Vendor,
    pub history: Vec<Message>,
    pub options: T,
    pub max_supported_tokens: u32,
    #[serde(skip)]
    pub meta: Meta,
}

impl<T: Default + Serialize + for<'a> Deserialize<'a>> Session<T> {
    /// Creates a new anonymous session
    pub fn anonymous(vendor: Vendor, options: T, max_supported_tokens: u32) -> Session<T> {
        let id = ulid::Ulid::new().to_string();
        let home = env::var("C_ROOT").unwrap_or(env::var("HOME").unwrap());
        let path = format!("{home}/.c/sessions/anonymous/{id}.yaml");
        Self {
            vendor,
            max_supported_tokens,
            options,
            meta: Meta {
                path,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Creates a new session
    pub fn new(id: String, vendor: Vendor, options: T, max_supported_tokens: u32) -> Session<T> {
        let home = env::var("C_ROOT").unwrap_or(env::var("HOME").unwrap());
        let path = format!("{home}/.c/sessions/{id}.yaml");
        Self {
            id,
            vendor,
            max_supported_tokens,
            options,
            meta: Meta {
                path,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Checks if a session exists
    pub fn exists(id: &str) -> bool {
        let home = env::var("C_ROOT").unwrap_or(env::var("HOME").unwrap());
        let path = format!("{home}/.c/sessions/{id}.yaml");
        fs::metadata(path).is_ok()
    }

    /// Tries to load a session from a file
    pub fn load(id: &str) -> Result<Session<T>> {
        let home = env::var("C_ROOT").unwrap_or(env::var("HOME")?);
        let path = format!("{home}/.c/sessions/{id}.yaml");

        let meta = Meta {
            path: path.clone(),
            ..Default::default()
        };

        let session = if fs::metadata(&path).is_ok() {
            let content = fs::read_to_string(&path)?;
            let mut session: Session<T> = serde_yaml::from_str(&content)?;
            session.meta = meta;
            session
        } else {
            Err(color_eyre::eyre::eyre!("Session not found"))?
        };

        Ok(session)
    }

    /// Saves the session to the filesystem
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
}

/// Chacks if a directory exists.
pub fn directory_exists(dir_name: &str) -> bool {
    let p = path::Path::new(dir_name);
    p.exists() && p.is_dir()
}
