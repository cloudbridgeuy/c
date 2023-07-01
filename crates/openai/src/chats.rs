use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};

use crate::client::Client;
use crate::error;
use crate::utils::{directory_exists, file_exists, get_home_directory};
use gpt_tokenizer::Default as DefaultTokenizer;
use log;
use serde::{Deserialize, Serialize};
use serde_either::SingleOrVec;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ChatsApi {
    #[serde(skip)]
    client: Client,
    // Chats Properties
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<SingleOrVec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<u32, f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_available_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_supported_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pin: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Chat {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub choices: Vec<ChatChoice>,
    pub usage: ChatUsage,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Chunk {
    pub id: String,
    pub object: String,
    pub created: u32,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: Option<ChunkMessage>,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChunkMessage {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

const DEFAULT_MODEL: &str = "gpt-3.5-turbo";

impl ChatsApi {
    /// Creates a new ChatsApi instance.
    pub fn new(api_key: String) -> Result<Self, error::OpenAi> {
        let client = match Client::new(api_key) {
            Ok(client) => client,
            Err(err) => {
                return Err(error::OpenAi::ClientError {
                    body: err.to_string(),
                });
            }
        };

        log::debug!("Created OpenAi HTTP Client");

        Ok(ChatsApi {
            client,
            model: String::from(DEFAULT_MODEL),
            messages: Vec::new(),
            ..Default::default()
        })
    }

    /// Creates a new ChatsApi instance by loading the sessions file
    pub fn new_with_session(api_key: String, session: String) -> Result<Self, error::OpenAi> {
        let session_file = get_sessions_file(&session)?;
        let mut chats_api = deserialize_sessions_file(&session_file)?;

        chats_api.client = match Client::new(api_key) {
            Ok(client) => client,
            Err(err) => {
                return Err(error::OpenAi::ClientError {
                    body: err.to_string(),
                });
            }
        };

        log::debug!("Created OpenAi HTTP Client");

        Ok(chats_api)
    }

    /// Stores the current session to a file.
    pub fn store_session(&self) -> Result<(), error::OpenAi> {
        if let Some(session) = &self.session {
            let session_file = get_sessions_file(session)?;
            serialize_sessions_file(&session_file, self)
        } else {
            Err(error::OpenAi::NoSession)
        }
    }

    /// Gets the value of the temperature.
    pub fn get_temperature(self) -> Option<f32> {
        self.temperature
    }

    /// Sets the value of the temperature.
    pub fn set_temperature(&mut self, temperature: f32) -> Result<&mut Self, error::OpenAi> {
        if !(0.0..=2.0).contains(&temperature) {
            return Err(error::OpenAi::InvalidTemperature { temperature });
        }
        self.temperature = Some(temperature);

        log::debug!("Set temperature to {}", temperature);

        Ok(self)
    }

    /// Gets the value of the top_p.
    pub fn get_top_p(self) -> Option<f32> {
        self.top_p
    }

    /// Sets the value of the top_p.
    pub fn set_top_p(&mut self, top_p: f32) -> Result<&mut Self, error::OpenAi> {
        if !(0.0..=2.0).contains(&top_p) {
            return Err(error::OpenAi::InvalidTopP { top_p });
        }
        self.top_p = Some(top_p);

        log::debug!("Set top_p to {}", top_p);

        Ok(self)
    }

    /// Gets the value of the stop.
    pub fn get_stop(self) -> Option<SingleOrVec<String>> {
        self.stop
    }

    /// Sets the value of the stop.
    pub fn set_stop(&mut self, stop: SingleOrVec<String>) -> Result<&mut Self, error::OpenAi> {
        match stop {
            SingleOrVec::Single(s) => {
                self.stop = Some(SingleOrVec::Single(s));
            }
            SingleOrVec::Vec(s) => {
                if s.len() <= 4 {
                    self.stop = Some(SingleOrVec::Vec(s));
                } else {
                    return Err(error::OpenAi::InvalidStop { stop: s.join(",") });
                }
            }
        }

        log::debug!("Set stop to {:?}", self.stop);

        Ok(self)
    }

    /// Gets the value of the presence_penalty.
    pub fn get_presence_penalty(self) -> Option<f32> {
        self.presence_penalty
    }

    /// Sets the value of the presence_penalty.
    pub fn set_presence_penalty(
        &mut self,
        presence_penalty: f32,
    ) -> Result<&mut Self, error::OpenAi> {
        if !(-2.0..=2.0).contains(&presence_penalty) {
            return Err(error::OpenAi::InvalidPresencePenalty { presence_penalty });
        }
        self.presence_penalty = Some(presence_penalty);

        log::debug!("Set presence_penalty to {}", presence_penalty);

        Ok(self)
    }

    /// Gets the value of the frequency_penalty.
    pub fn get_frequency_penalty(self) -> Option<f32> {
        self.frequency_penalty
    }

    /// Sets the value of the frequency_penalty.
    pub fn set_frequency_penalty(
        &mut self,
        frequency_penalty: f32,
    ) -> Result<&mut Self, error::OpenAi> {
        if !(-2.0..=2.0).contains(&frequency_penalty) {
            return Err(error::OpenAi::InvalidFrequencyPenalty { frequency_penalty });
        }
        self.frequency_penalty = Some(frequency_penalty);

        log::debug!("Set frequency_penalty to {}", frequency_penalty);

        Ok(self)
    }

    /// Creates a completion for the chat message in stream format.
    pub async fn create_stream(
        &self,
    ) -> Result<impl Stream<Item = Result<Chunk, error::OpenAi>>, error::OpenAi> {
        let mut api = &mut (*self).clone();

        let min_available_tokens = api.min_available_tokens.unwrap_or(750);
        let max_supported_tokens = api.max_supported_tokens.unwrap_or(4096);
        let session = api.session.clone();
        let messages = api.messages.clone();

        api.session = None;
        api.min_available_tokens = None;
        api.max_supported_tokens = None;
        api.messages = trim_messages(
            api.messages.clone(),
            max_supported_tokens - min_available_tokens,
        )?
        .iter()
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
            ..Default::default()
        })
        .collect();

        log::debug!("Trimmed messages to {:?}", api.messages);

        let request = match serde_json::to_string(api) {
            Ok(request) => request,
            Err(err) => {
                return Err(error::OpenAi::SerializationError {
                    body: err.to_string(),
                });
            }
        };

        log::debug!("Request: {}", request);

        log::debug!("Streaming completion");
        let mut event_source = match self.client.post_stream("/chat/completions", request).await {
            Ok(response) => response,
            Err(e) => {
                return Err(error::OpenAi::RequestError {
                    body: e.to_string(),
                })
            }
        };

        let (tx, rx) = mpsc::channel(100);
        let acc = Arc::new(Mutex::new(String::new()));
        let acc_clone = Arc::clone(&acc);

        tokio::spawn(async move {
            while let Some(ev) = event_source.next().await {
                match ev {
                    Err(_) => {
                        if tx.send(Err(error::OpenAi::StreamError)).await.is_err() {
                            return;
                        }
                    }
                    Ok(event) => match event {
                        reqwest_eventsource::Event::Open { .. } => {}
                        reqwest_eventsource::Event::Message(message) => {
                            log::debug!("Message: {:?}", message);

                            if message.data == "[DONE]" {
                                return;
                            }

                            let chunk: Chunk = match serde_json::from_str(&message.data) {
                                Err(_) => {
                                    if tx.send(Err(error::OpenAi::StreamError)).await.is_err() {
                                        return;
                                    }
                                    return;
                                }
                                Ok(output) => output,
                            };

                            log::debug!("Response: {:?}", chunk);

                            if let Some(choice) = &chunk.choices.get(0) {
                                if let Some(delta) = &choice.delta {
                                    if let Some(content) = &delta.content {
                                        let mut accumulator = acc.lock().await;
                                        accumulator.push_str(&content.clone());
                                    }
                                }
                            }

                            if tx.send(Ok(chunk)).await.is_err() {
                                return;
                            }
                        }
                    },
                }
            }
        });

        log::debug!("Checking for session, {:?}", session);
        if let Some(session) = session {
            let session_file = get_sessions_file(&session)?;
            api.session = Some(session);
            api.min_available_tokens = Some(min_available_tokens);
            api.max_supported_tokens = Some(max_supported_tokens);
            api.messages = messages;

            let data = acc_clone.lock().await;
            let data_string = &*data;

            api.messages.push(ChatMessage {
                content: Some(data_string.to_string()),
                role: "assistant".to_string(),
                ..Default::default()
            });
            serialize_sessions_file(&session_file, api)?;
        }

        Ok(ReceiverStream::from(rx))
    }

    /// Creates a completion for the chat message
    pub async fn create(&self) -> Result<Chat, error::OpenAi> {
        let mut api = &mut (*self).clone();

        let min_available_tokens = api.min_available_tokens.unwrap_or(750);
        let max_supported_tokens = api.max_supported_tokens.unwrap_or(4096);
        let session = api.session.clone();
        let messages = api.messages.clone();

        api.session = None;
        api.min_available_tokens = None;
        api.max_supported_tokens = None;
        api.messages = trim_messages(
            api.messages.clone(),
            max_supported_tokens - min_available_tokens,
        )?
        .iter()
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
            ..Default::default()
        })
        .collect();

        log::debug!("Trimmed messages to {:?}", api.messages);

        let request = match serde_json::to_string(api) {
            Ok(request) => request,
            Err(err) => {
                return Err(error::OpenAi::SerializationError {
                    body: err.to_string(),
                });
            }
        };

        log::debug!("Request: {}", request);

        let body = match self.client.post("/chat/completions", request).await {
            Ok(response) => match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    return Err(error::OpenAi::RequestError {
                        body: e.to_string(),
                    })
                }
            },
            Err(e) => {
                return Err(error::OpenAi::RequestError {
                    body: e.to_string(),
                })
            }
        };

        log::debug!("Response: {}", body);

        let body: Chat = match serde_json::from_str(&body) {
            Ok(body) => body,
            Err(e) => {
                return Err(error::OpenAi::RequestError {
                    body: e.to_string(),
                })
            }
        };

        log::debug!("Checking for session, {:?}", session);
        if let Some(session) = session {
            let session_file = get_sessions_file(&session)?;
            api.session = Some(session);
            api.min_available_tokens = Some(min_available_tokens);
            api.max_supported_tokens = Some(max_supported_tokens);
            api.messages = messages;
            api.messages
                .push(body.choices.first().unwrap().message.clone());
            serialize_sessions_file(&session_file, api)?;
        }

        Ok(body)
    }
}

/// Get the path to the sessions file.
pub fn get_sessions_file(session: &str) -> Result<String, error::OpenAi> {
    log::debug!("Getting sessions file: {}", session);

    let home_dir = get_home_directory();

    log::debug!("Home directory: {}", home_dir);

    // Create the HOME directory if it doesn't exist
    if !directory_exists(&home_dir) {
        log::debug!("Creating home directory: {}", home_dir);
        create_dir_all(&home_dir).unwrap();
    }

    let sessions_file = format!("{}/{}", home_dir, session);

    // Create the sessions file if it doesn't exist
    if !file_exists(&sessions_file) {
        log::debug!("Creating sessions file: {}", sessions_file);
        File::create(&sessions_file).unwrap();
        let mut chats_api = ChatsApi::new(Default::default())?;
        chats_api.session = Some(session.to_string());
        chats_api.messages = Vec::new();
        serialize_sessions_file(&sessions_file, &chats_api)?;
    }

    log::debug!("Sessions file: {}", sessions_file);

    Ok(sessions_file)
}

/// Deserialize the sessions file.
pub fn deserialize_sessions_file(session_file: &str) -> Result<ChatsApi, error::OpenAi> {
    log::debug!("Deserializing sessions file: {}", session_file);

    let file = match File::open(session_file) {
        Ok(file) => file,
        Err(err) => {
            return Err(error::OpenAi::FileError {
                body: err.to_string(),
            });
        }
    };

    let reader = BufReader::new(file);

    let chats_api: ChatsApi = match serde_json::from_reader(reader) {
        Ok(chats_api) => chats_api,
        Err(err) => {
            return Err(error::OpenAi::DeserializationError {
                body: err.to_string(),
            });
        }
    };

    Ok(chats_api)
}

/// Serialize the sessions file
pub fn serialize_sessions_file(
    session_file: &str,
    chats_api: &ChatsApi,
) -> Result<(), error::OpenAi> {
    log::debug!("Serializing sessions file: {}", session_file);

    let file = match File::create(session_file) {
        Ok(file) => file,
        Err(err) => {
            return Err(error::OpenAi::FileError {
                body: err.to_string(),
            });
        }
    };

    let writer = BufWriter::new(file);

    match serde_json::to_writer_pretty(writer, &chats_api) {
        Ok(_) => Ok(()),
        Err(err) => Err(error::OpenAi::SerializationError {
            body: err.to_string(),
        }),
    }
}

/// Trim messages until the total number of tokenizers inside is less than the maximum.
fn trim_messages(
    mut messages: Vec<ChatMessage>,
    max: u32,
) -> Result<Vec<ChatMessage>, error::OpenAi> {
    let tokenizer = DefaultTokenizer::new();

    let total_tokens: usize = messages
        .iter()
        .map(|m| {
            if let Some(content) = &m.content {
                tokenizer.encode(content).len()
            } else {
                0
            }
        })
        .sum();

    if total_tokens as u32 <= max {
        return Ok(messages);
    }

    if let Some((index, _)) = messages
        .iter()
        .enumerate()
        .find(|(_, m)| m.role != "system" && Some(true) != m.pin)
    {
        messages.remove(index);
        trim_messages(messages, max)
    } else {
        Err(error::OpenAi::TrimError)
    }
}
