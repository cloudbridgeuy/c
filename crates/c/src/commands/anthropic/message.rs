use serde::{Deserialize, Serialize};

/// Stores a message.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Message {
    pub content: String,
    pub role: Role,
    pub pin: bool,
}

impl Message {
    /// Creates a new message.
    pub fn new(content: String, role: Role, pin: bool) -> Self {
        Self { content, role, pin }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    #[default]
    /// The user is a human
    Human,
    /// The user is a bot
    Assistant,
}
