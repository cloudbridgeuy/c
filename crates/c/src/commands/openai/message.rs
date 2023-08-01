use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum Role {
    #[default]
    /// The user is a human
    User,
    /// The user is a bot
    Assistant,
    /// System message prompt
    System,
}

/// Stores a message.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    /// Creates a new message.
    pub fn new(content: String, role: Role) -> Self {
        Self { content, role }
    }
}

/// Stores a history message.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub role: Role,
    pub content: String,
    pub pin: bool,
}

impl HistoryMessage {
    /// Creates a new message.
    pub fn new(content: String, role: Role, pin: bool) -> Self {
        Self { content, role, pin }
    }
}

impl From<Message> for HistoryMessage {
    fn from(messages: Message) -> Self {
        HistoryMessage {
            role: messages.role,
            content: messages.content,
            pin: false, // Default to false
        }
    }
}

impl From<HistoryMessage> for Message {
    fn from(history_messages: HistoryMessage) -> Self {
        Message {
            role: history_messages.role,
            content: history_messages.content,
        }
    }
}
