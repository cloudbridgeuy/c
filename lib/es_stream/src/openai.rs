use eventsource_client as es;
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::error::Error;
use crate::requests::{Json, Requests};

// Chat Completions Api
const CHAT_API: &str = "/chat/completions";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Assistant,
    User,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MessageBody {
    /// ID of the model to use. See the model endpoint compatibility table for details on which models work with the Chat API.
    pub model: String,

    /// A list of messages comprising the conversation so far.
    pub messages: Vec<Message>,

    /// The maximum number of tokens to generate before stopping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Modify the likelihood of specified tokens appearing in the completion.
    ///
    /// Accepts a JSON object that maps tokens (specified by their token ID in the tokenizer) to an associated bias value from -100 to 100. Mathematically, the bias is added to the logits generated by the model prior to sampling. The exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood of selection; values like -100 or 100 should result in a ban or exclusive selection of the relevant token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    /// An object describing metadata about the request.
    /// Whether to incrementally stream the response using server-sent events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// An integer between 0 and 20 specifying the number of most likely tokens to return at each token position, each with an associated log probability. logprobs must be set to true if this parameter is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,

    /// The maximum number of tokens that can be generated in the chat completion.
    ///
    /// The total length of input tokens and generated tokens is limited by the model's context length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// How many chat completion choices to generate for each input message. Note that you will be charged based on the number of generated tokens across all of the choices. Keep n as 1 to minimize costs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// This feature is in Beta. If specified, our system will make a best effort to sample deterministically, such that repeated requests with the same seed and parameters should return the same result. Determinism is not guaranteed, and you should refer to the system_fingerprint response parameter to monitor changes in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u32>,

    /// Up to 4 sequences where the API will stop generating further tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// If set, partial message deltas will be sent, like in ChatGPT. Tokens will be sent as data-only server-sent events as they become available, with the stream terminated by a data: [DONE] message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    ///
    /// We generally recommend altering this or top_p but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or temperature but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl MessageBody {
    /// Creates a new `MessageBody`
    #[must_use]
    pub fn new(model: &str, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            stream: Some(true),
            ..Default::default()
        }
    }
}

/// A chat completion delta generated by the streamed model responses.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatCompletionChunkChoiceDelta {
    /// The contents of the chunk message.
    content: Option<String>,
}

/// Represents a content choice of a streamed chunk of a chat completion response returned by model, based on the provided input.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatCompletionChunkChoice {
    /// A chat completion delta generated by the streamed model responses.
    delta: ChatCompletionChunkChoiceDelta,
}

/// Represents a streamed chunk of a chat completion response returned by model, based on the provided input.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatCompletionChunk {
    /// A list of chat completion choices. Can contain more than one elements if n is greater than 1. Can also be empty for the last chunk if you set stream_options: {"include_usage": true}.
    pub choices: Vec<ChatCompletionChunkChoice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Auth {
    pub api_key: String,
}

impl Auth {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn from_env() -> Result<Self, Error> {
        let api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(key) => key,
            Err(_) => return Err(Error::AuthError("OPENAI_API_KEY not found".to_string())),
        };
        Ok(Self { api_key })
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    pub auth: Auth,
    pub api_url: String,
}

impl Client {
    pub fn new(auth: Auth, api_url: impl Into<String>) -> Self {
        Self {
            auth,
            api_url: api_url.into(),
        }
    }
}

impl Client {
    pub fn delta<'a>(
        &'a self,
        message_body: &'a MessageBody,
    ) -> Result<impl Stream<Item = Result<String, Error>> + 'a, Error> {
        log::debug!("message_body: {:#?}", message_body);

        let request_body = match serde_json::to_value(message_body) {
            Ok(body) => body,
            Err(e) => return Err(Error::Serde(e)),
        };
        log::debug!("request_body: {:#?}", request_body);

        let original_stream = match self.post_stream(CHAT_API.to_string(), request_body) {
            Ok(stream) => stream,
            Err(e) => return Err(Error::SseStreamCreation(Box::new(e))),
        };

        let mapped_stream = original_stream.map(|item| {
            item.map(|event| match event {
                es::SSE::Connected(_) => String::default(),
                es::SSE::Event(ev) => match serde_json::from_str::<ChatCompletionChunk>(&ev.data) {
                    Ok(mut chunk) => {
                        if chunk.choices.is_empty() {
                            String::default()
                        } else {
                            chunk.choices[0].delta.content.take().unwrap_or_default()
                        }
                    }
                    Err(_) => String::default(),
                },
                es::SSE::Comment(comment) => {
                    log::debug!("Comment: {:#?}", comment);
                    String::default()
                }
            })
            .map_err(Error::from)
        });

        Ok(mapped_stream)
    }
}

impl Requests for Client {
    fn post_stream(
        &self,
        sub_url: String,
        body: Json,
    ) -> Result<impl Stream<Item = Result<es::SSE, es::Error>>, es::Error> {
        let authorization: &str = &format!("Bearer {}", self.auth.api_key);

        let client = es::ClientBuilder::for_url(&(self.api_url.clone() + &sub_url))?
            .header("content-type", "application/json")?
            .header("authorization", authorization)?
            .method("POST".into())
            .body(body.to_string())
            .reconnect(
                es::ReconnectOptions::reconnect(true)
                    .retry_initial(false)
                    .delay(Duration::from_secs(1))
                    .backoff_factor(2)
                    .delay_max(Duration::from_secs(60))
                    .build(),
            )
            .build();

        Ok(crate::requests::tail(&client))
    }
}