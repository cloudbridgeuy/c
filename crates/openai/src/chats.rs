use std::collections::HashMap;

use log;
use serde::{Deserialize, Serialize};
use serde_either::SingleOrVec;

use crate::client::Client;
use crate::error;

#[derive(Debug, Serialize, Deserialize, Default)]
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
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
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
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

const DEFAULT_MODEL: &str = "gpt-3.5-turbo";

impl ChatsApi {
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

    /// Creates a completion for the chat message
    pub async fn create(&self) -> Result<Chat, error::OpenAi> {
        let request = match serde_json::to_string(&self) {
            Ok(request) => request,
            Err(err) => {
                return Err(error::OpenAi::SerializationError {
                    body: err.to_string(),
                });
            }
        };

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

        Ok(body)
    }
}
