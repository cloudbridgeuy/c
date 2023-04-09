use std::collections::HashMap;

use log;
use serde::{Deserialize, Serialize};
use serde_either::SingleOrVec;

use crate::client::Client;
use crate::error;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompletionsApi {
    #[serde(skip)]
    client: Client,
    // Completions Properties
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<SingleOrVec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    echo: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<SingleOrVec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_of: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Completions {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Choice {
    pub text: String,
    pub index: u32,
    pub logprobs: Option<f32>,
    pub finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

const DEFAULT_MODEL: &str = "text-davinci-003";

impl CompletionsApi {
    /// Creates a new CompletionsApi.
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
            model: String::from(DEFAULT_MODEL),
            ..Default::default()
        })
    }

    /// Gets the value of the echo.
    pub fn get_echo(self) -> Option<bool> {
        self.echo
    }

    /// Sets the value of the echo.
    pub fn set_echo(&mut self, echo: bool) -> Result<&mut Self, error::OpenAi> {
        // Can't run 'echo' with 'suffix'
        if let Some(_) = &self.suffix {
            if echo {
                return Err(error::OpenAi::InvalidEcho);
            }
        }
        self.echo = Some(echo);

        log::debug!("Set echo to {}", echo);

        Ok(self)
    }

    /// Gets the value of the stream.
    pub fn get_stream(self) -> Option<bool> {
        self.stream
    }

    /// Sets the value of the stream.
    pub fn set_stream(&mut self, stream: bool) -> Result<&mut Self, error::OpenAi> {
        // Can't run 'stream' with 'suffix'
        if let Some(_) = &self.best_of {
            if stream {
                return Err(error::OpenAi::InvalidStream);
            }
        }
        self.stream = Some(stream);

        log::debug!("Set stream to {}", stream);

        Ok(self)
    }

    /// Gets the value of the suffix.
    pub fn get_suffix(self) -> Option<String> {
        self.suffix
    }

    /// Sets the value of the suffix.
    pub fn set_suffix(&mut self, suffix: String) -> Result<&mut Self, error::OpenAi> {
        // Can't run 'suffix' with 'suffix'
        if let Some(_) = &self.echo {
            return Err(error::OpenAi::InvalidSuffix);
        }
        self.suffix = Some(suffix);

        log::debug!("Set suffix to {:?}", &self.suffix);

        Ok(self)
    }

    /// Gets the value of the best_of.
    pub fn get_best_of(self) -> Option<u32> {
        self.best_of
    }

    /// Sets the value of the best_of.
    pub fn set_best_of(&mut self, best_of: u32) -> Result<&mut Self, error::OpenAi> {
        // Can't run 'best_of' with 'best_of'
        if let Some(_) = &self.stream {
            return Err(error::OpenAi::InvalidBestOf);
        }
        self.best_of = Some(best_of);

        log::debug!("Set best_of to {:?}", &self.best_of);

        Ok(self)
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

    /// Gets the value of the logprobs.
    pub fn get_logprobs(self) -> Option<f32> {
        self.logprobs
    }

    /// Sets the value of the logprobs.
    pub fn set_logprobs(&mut self, logprobs: f32) -> Result<&mut Self, error::OpenAi> {
        if !(0.0..=5.0).contains(&logprobs) {
            return Err(error::OpenAi::InvalidLogProbs { logprobs });
        }
        self.logprobs = Some(logprobs);

        log::debug!("Set logprobs to {}", logprobs);

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

    /// Creates a completion for the provided parameters.
    pub fn create(&self) -> Result<Completions, error::OpenAi> {
        let request = match serde_json::to_string(&self) {
            Ok(request) => request,
            Err(err) => {
                return Err(error::OpenAi::SerializationError {
                    body: err.to_string(),
                });
            }
        };

        let body = match self.client.post("/completions", request) {
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

        let body: Completions = match serde_json::from_str(&body) {
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
