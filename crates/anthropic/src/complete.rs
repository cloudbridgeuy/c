use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};

use color_eyre::eyre::{anyhow, Result};
use fs::{directory_exists, file_exists, get_home_directory};
use serde::{self, Deserialize, Serialize};

use crate::client::Client;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Model {
    #[default]
    ClaudeV1,
    ClaudeV1_100k,
    ClaudeInstantV1,
    ClaudeInstantV1_100k,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Chunk {
    event: String,
    data: ChunkData,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ChunkData {
    completion: Option<String>,
    stop_reason: Option<String>,
    model: Option<String>,
    error: Option<ChunkError>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ChunkError {
    _type: String,
    message: String,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeV1 => "claude-v1",
            Self::ClaudeV1_100k => "claude-v1-100k",
            Self::ClaudeInstantV1 => "claude-instant-v1",
            Self::ClaudeInstantV1_100k => "claude-instant-v1-100k",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Api {
    #[serde(skip)]
    client: Client,
    // Complete Properties (https://console.anthropic.com/docs/api/reference)
    pub model: Model,
    pub prompt: String,
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_to_sample: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_supported_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ChatCreateBody {
    pub model: Model,
    pub prompt: String,
    pub max_tokens_to_sample: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Metadata {
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Response {
    pub completion: String,
    pub stop_reason: String,
}

impl Api {
    /// Creates a new Complete API instance.
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::new(api_key)?;

        Ok(Self {
            client,
            ..Default::default()
        })
    }

    /// Creates a ChatCreateBody from the API body.
    pub fn create_chat_body(&self) -> ChatCreateBody {
        ChatCreateBody {
            max_tokens_to_sample: self.max_tokens_to_sample,
            metadata: self.metadata.clone(),
            model: self.model,
            prompt: self.prompt.clone(),
            stop_sequences: self.stop_sequences.clone(),
            stream: self.stream,
            temperature: self.temperature,
            top_k: self.top_k,
            top_p: self.top_p,
        }
    }

    /// Creates a new Complete API instance by loading options from a sessions file.
    pub fn new_with_session(api_key: String, session: String) -> Result<Self> {
        let session_file = get_session_file(&session)?;

        tracing::event!(
            tracing::Level::INFO,
            "Deserializing session file {}",
            session
        );

        let mut complete_api = deserialize_sessions_file(&session_file)?;

        tracing::event!(
            tracing::Level::INFO,
            "Creating anthropic client with session {}",
            session
        );

        complete_api.client = Client::new(api_key)?;

        Ok(complete_api)
    }

    /// Stores the current session to a file.
    pub fn store_session(&self) -> Result<()> {
        if let Some(session) = &self.session {
            let session_file = get_session_file(session)?;
            serialize_sessions_file(&session_file, self)
        } else {
            Err(anyhow!("no session found"))
        }
    }

    /// Gets the temperature value
    pub fn get_temperature(&self) -> Option<f32> {
        self.temperature
    }

    /// Sets the temperature value
    pub fn set_temperature(&mut self, temperature: f32) -> Result<&mut Self> {
        tracing::event!(
            tracing::Level::INFO,
            "seting temperature to {}",
            temperature
        );

        if !(0.0..=1.0).contains(&temperature) {
            return Err(anyhow!("temperature must be between 0.0 and 1.0"));
        }

        self.temperature = Some(temperature);

        Ok(self)
    }

    /// Gets the top_k value
    pub fn get_top_k(&self) -> Option<f32> {
        self.top_k
    }

    /// Sets the top_k value
    pub fn set_top_k(&mut self, top_k: f32) -> Result<&mut Self> {
        tracing::event!(tracing::Level::INFO, "seting top_k to {}", top_k);

        if top_k != -1.0 && top_k < 0.0 {
            return Err(anyhow!("top_k must be -1 or greater than zero"));
        }

        self.top_k = Some(top_k);

        Ok(self)
    }

    /// Gets the top_p value
    pub fn get_top_p(&self) -> Option<f32> {
        self.top_p
    }

    /// Sets the top_p value
    pub fn set_top_p(&mut self, top_p: f32) -> Result<&mut Self> {
        tracing::event!(tracing::Level::INFO, "seting top_p to {}", top_p);

        if top_p != -1.0 && top_p < 0.0 {
            return Err(anyhow!("top_p must be -1 or greater than zero"));
        }

        self.top_p = Some(top_p);

        Ok(self)
    }

    /// Creates a new chat session with the given prompt, streaming the result.
    // pub async fn create_stream(&self) -> Result<impl Stream<Item = Result<Chunk>>> {
    pub async fn create(&mut self) -> Result<Response> {
        tracing::event!(tracing::Level::INFO, "Creating new streaming chat session");
        let mut prompt = self.prompt.clone();

        let mut max =
            self.max_supported_tokens.unwrap_or(4096) - self.max_tokens_to_sample.unwrap_or(750);

        if self.system.is_some() {
            tracing::event!(tracing::Level::INFO, "system: {:?}", self.system);
            prompt = prepare_prompt(format!("{}\n{}", self.system.as_ref().unwrap(), prompt));
            max -= token_length(self.system.as_ref().unwrap().to_string()) as u32;
        }

        tracing::event!(tracing::Level::INFO, "Trimming prompt...");
        prompt = trim_prompt(prompt, max)?;
        tracing::event!(tracing::Level::INFO, "prompt: {}", prompt);

        tracing::event!(tracing::Level::INFO, "Creating request...");
        let mut body = self.create_chat_body();
        body.prompt = prompt;
        let request = serde_json::to_string(&body)?;
        tracing::event!(tracing::Level::INFO, "request: {}", request);

        tracing::event!(tracing::Level::INFO, "Sending request...");
        let response = self.client.post("/v1/complete", request).await?;
        tracing::event!(tracing::Level::INFO, "response: {:?}", response);

        tracing::event!(tracing::Level::INFO, "Getting response body...");
        let body = response.text().await?;
        tracing::event!(tracing::Level::INFO, "body: {:?}", body);

        tracing::event!(tracing::Level::INFO, "Deserializing response body...");
        let response: Response = serde_json::from_str(&body)?;
        tracing::event!(tracing::Level::INFO, "response: {:?}", response);

        if let Some(session) = &self.session {
            tracing::event!(tracing::Level::INFO, "Getting session file: {:?}", session);
            let session_file = get_session_file(session)?;

            tracing::event!(tracing::Level::INFO, "Updating session prompt");
            self.prompt.push_str(&response.completion);

            tracing::event!(tracing::Level::INFO, "Serializing session file");
            serialize_sessions_file(&session_file, self)?;
        }

        Ok(response)
    }
}

pub fn get_session_file(session: &str) -> Result<String> {
    tracing::event!(tracing::Level::INFO, "Getting home directory...");
    let home_directory = get_home_directory();
    tracing::event!(tracing::Level::INFO, "home directory: {}", home_directory);

    // Create the HOME directory if it doesn't exist.
    if !directory_exists(&home_directory) {
        tracing::event!(
            tracing::Level::INFO,
            "Creating home directory: {}",
            home_directory
        );
        create_dir_all(&home_directory)?;
    }

    let sessions_file = format!("{}/{}.yaml", home_directory, session);

    tracing::event!(
        tracing::Level::INFO,
        "Getting sessions file: {}",
        sessions_file
    );
    if !file_exists(&sessions_file) {
        tracing::event!(
            tracing::Level::INFO,
            "Creating sessions file: {}",
            sessions_file
        );
        File::create(&sessions_file)?;

        let mut complete_api = Api::new(Default::default())?;
        complete_api.session = Some(session.to_string());

        tracing::event!(
            tracing::Level::INFO,
            "Serializing sessions file: {}",
            sessions_file
        );
        serialize_sessions_file(&sessions_file, &complete_api)?;
    }

    Ok(sessions_file)
}

/// Deserialize the sessions file
pub fn deserialize_sessions_file(sessions_file: &str) -> Result<Api> {
    let file = File::open(sessions_file)?;
    let reader = BufReader::new(file);
    let complete_api = serde_yaml::from_reader(reader)?;

    Ok(complete_api)
}

/// Serialize the sessions file
pub fn serialize_sessions_file(session_file: &str, complete_api: &Api) -> Result<()> {
    let file = File::create(session_file)?;
    let writer = BufWriter::new(file);

    serde_yaml::to_writer(writer, complete_api)?;

    Ok(())
}

/// Token language of a prompt.
/// TODO: Make this better!
fn token_length(prompt: String) -> usize {
    let words = prompt.split_whitespace().rev().collect::<Vec<&str>>();

    // Estimate the total tokens by multiplying words by 4/3
    words.len() * 4 / 3
}

/// Trims the size of the prompt to match the max value.
fn trim_prompt(prompt: String, max: u32) -> Result<String> {
    let prompt = prepare_prompt(prompt);
    let tokens = token_length(prompt.clone());

    if tokens as u32 <= max {
        return Ok(prompt);
    }

    let mut words = prompt.split_whitespace().rev().collect::<Vec<&str>>();

    // Because we need to add back "\n\nHuman:" back to the prompt.
    let diff = words.len() - (max + 3) as usize;

    // Take the last `diff` words, and reverse the order of those words.
    words.truncate(diff);
    words.reverse();

    // Join the selected words back together into a single string.
    Ok(prepare_prompt(words.join(" ")))
}

/// Prepare prompt for completion
fn prepare_prompt(prompt: String) -> String {
    let mut prompt = "\n\nHuman: ".to_string() + &prompt + "\n\nAssistant:";

    prompt = prompt.replace("\n\n\n", "\n\n");
    prompt = prompt.replace("Human: Human:", "Human:");
    prompt = prompt.replace("\n\nHuman:\n\nHuman: ", "\n\nHuman: ");
    prompt = prompt.replace("\n\nHuman: \nHuman: ", "\n\nHuman: ");
    prompt = prompt.replace("\n\nHuman: \n\nHuman: ", "\n\nHuman: ");
    prompt = prompt.replace("\n\nHuman:\n\nAssistant:", "\n\nAssistant:");
    prompt = prompt.replace("\n\nHuman: \n\nAssistant:", "\n\nAssistant:");
    prompt = prompt.replace("\n\nAssistant:\n\nAssistant:", "\n\nAssistant:");
    prompt = prompt.replace("\n\nAssistant: \n\nAssistant: ", "\n\nAssistant:");

    prompt
}
