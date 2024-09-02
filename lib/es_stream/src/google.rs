use eventsource_client as es;
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::Error;
use crate::requests::{Json, Requests};

// Chat Completions Api
const STREAM_GENERATE_CONTENT_TEMPLATE: &str =
    "/models/{{model}}:streamGenerateContent?alt=sse&key={{key}}";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Model,
    #[default]
    User,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Configuration options for model generation and outputs. Not all parameters are configurable for every model.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    /// The set of character sequences (up to 5) that will stop output generation. If specified, the API will stop at the first appearance of a stop_sequence. The stop sequence will not be included as part of the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// MIME type of the generated candidate text. Supported MIME types are: text/plain: (default) Text output. application/json: JSON response in the response candidates. Refer to the docs for a list of all supported text MIME types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,

    /// Number of generated responses to return.
    pub candidate_count: Option<u32>,

    /// The maximum number of tokens to include in a response candidate.
    pub max_output_tokens: Option<u32>,

    /// Controls the randomness of the output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// The maximum cumulative probability of tokens to consider when sampling.
    ///
    /// The model uses combined Top-k and Top-p (nucleus) sampling.
    ///
    /// Tokens are sorted based on their assigned probabilities so that only the most likely tokens are considered. Top-k sampling directly limits the maximum number of tokens to consider, while Nucleus sampling limits the number of tokens based on the cumulative probability.
    ///
    /// Note: The default value varies by Model and is specified by theModel.top_p attribute returned from the getModel function. An empty topK attribute indicates that the model doesn't apply top-k sampling and doesn't allow setting topK on requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// The maximum number of tokens to consider when sampling.
    ///
    /// Gemini models use Top-p (nucleus) sampling or a combination of Top-k and nucleus sampling. Top-k sampling considers the set of topK most probable tokens. Models running with nucleus sampling don't allow topK setting.
    ///
    /// Note: The default value varies by Model and is specified by theModel.top_p attribute returned from the getModel function. An empty topK attribute indicates that the model doesn't apply top-k sampling and doesn't allow setting topK on requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
}

/// A datatype containing media that is part of a multi-part Content message.
///
/// A Part consists of data which has an associated datatype. A Part can only contain one of the accepted types in Part.data.
///
/// A Part must have a fixed IANA MIME type identifying the type and subtype of the media if the inlineData field is filled with raw bytes.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    /// Inline text.
    pub text: String,
}

/// The base structured datatype containing multi-part content of a message.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    /// Ordered Parts that constitute a single message. Parts may have different MIME types.
    pub parts: Vec<Part>,

    /// The producer of the content. Must be either 'user' or 'model'.
    pub role: Role,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MessageBody {
    /// The name of the Model to use for generating the completion.
    #[serde(skip_serializing)]
    pub model: String,

    /// The content of the current conversation with the model.
    pub contents: Vec<Content>,

    /// Configuration options for model generation and outputs.
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Candidate {
    pub content: Content,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    pub candidates: Vec<Candidate>,
}

impl MessageBody {
    /// Creates a new `MessageBody`
    #[must_use]
    pub fn new(model: &str, contents: Vec<Content>) -> Self {
        Self {
            model: model.into(),
            contents,
            ..Default::default()
        }
    }
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

        let sub_url =
            STREAM_GENERATE_CONTENT_TEMPLATE.replace("{{model}}", message_body.model.as_str());

        let original_stream = match self.post_stream(sub_url, request_body) {
            Ok(stream) => stream,
            Err(e) => return Err(Error::EventsourceClient(e)),
        };

        let mapped_stream = original_stream.map(|item| {
            if item.is_err() {
                return Err(Error::EventsourceClient(item.err().unwrap()));
            }
            item.map(|event| match event {
                es::SSE::Connected(_) => String::default(),
                es::SSE::Event(ev) => match serde_json::from_str::<Root>(&ev.data) {
                    Ok(root) => {
                        if root.candidates[0].content.parts.is_empty() {
                            String::default()
                        } else {
                            root.candidates[0].content.parts[0].text.clone()
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
        let url = &(self.api_url.clone() + &sub_url);
        let url = url.replace("{{key}}", &self.auth.api_key);

        let client = es::ClientBuilder::for_url(&url)?
            .header("content-type", "application/json")?
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
