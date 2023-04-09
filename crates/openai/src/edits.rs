use log;
use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EditsApi {
    #[serde(skip)]
    client: Client,
    // Edits Properties
    pub model: String,
    pub input: String,
    pub instruction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Edit {
    pub object: String,
    pub created: u64,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Choice {
    pub text: String,
    pub index: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

const DEFAULT_MODEL: &str = "code-davinci-edit-001";

impl EditsApi {
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

        Ok(Self {
            client,
            model: DEFAULT_MODEL.to_string(),
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

    /// Creates an edit from the provided parameters.
    pub fn create(&self) -> Result<Edit, error::OpenAi> {
        let request = match serde_json::to_string(&self) {
            Ok(request) => request,
            Err(err) => {
                return Err(error::OpenAi::SerializationError {
                    body: err.to_string(),
                });
            }
        };

        let body = match self.client.post("/edits", request) {
            Ok(response) => match response.text() {
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

        let body: Edit = match serde_json::from_str(&body) {
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
