use std::env;
use std::fs;
use std::ops::RangeInclusive;
use std::path;

use anthropic::client::Client;
use clap::{Parser, ValueEnum};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use ulid::Ulid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub completion: String,
    pub stop_reason: Option<String>,
    pub model: String,
    pub stop: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    #[default]
    /// The user is a human
    Human,
    /// The user is a bot
    Assistant,
}

#[derive(ValueEnum, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum Model {
    #[default]
    Claude2,
    ClaudeV1,
    ClaudeV1_100k,
    ClaudeInstantV1,
    ClaudeInstantV1_100k,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude2 => "claude-2",
            Self::ClaudeV1 => "claude-v1",
            Self::ClaudeV1_100k => "claude-v1-100k",
            Self::ClaudeInstantV1 => "claude-instant-v1",
            Self::ClaudeInstantV1_100k => "claude-instant-v1-100k",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CompleteRequestBody {
    pub model: String,
    pub prompt: String,
    pub max_tokens_to_sample: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Response {
    pub completion: String,
    pub stop_reason: String,
}

#[derive(Default, Clone, Parser, Debug, Serialize, Deserialize)]
pub struct Options {
    /// The prompt you want Claude to complete.
    prompt: Option<String>,
    /// Chat session name. Will be used to store previous session interactions.
    #[arg(long)]
    session: Option<String>,
    /// The maximum number of tokens supported by the model.
    #[arg(long)]
    max_supported_tokens: Option<u32>,
    /// Controls which version of Claude answers your request. Two model families are exposed
    /// Claude and Claude Instant.
    #[clap(short, long, value_enum, default_value = "claude-v1")]
    model: Option<Model>,
    /// A maximum number of tokens to generate before stopping.
    #[arg(long, default_value = "1000")]
    max_tokens_to_sample: Option<u32>,
    /// Claude models stop on `\n\nHuman:`, and may include additional built-in stops sequences
    /// in the future. By providing the `stop_sequences` parameter, you may include additional
    /// strings that will cause the model to stop generation.
    #[clap(long)]
    stop_sequences: Option<Vec<String>>,
    /// Amount of randomness injected into the response. Ranges from 0 to 1. Use temp closer to
    /// 0 for analytical/multiple choice, and temp closer to 1 for creative and generative
    /// tasks.
    #[clap(long, value_parser = parse_temperature)]
    temperature: Option<f32>,
    /// Only sample fromt the top `K` options of each subsequent token. Used to remove "long
    /// tail" low probability responses. Defaults to -1, which disables it.
    #[clap(long, value_parser = parse_top_k)]
    top_k: Option<f32>,
    /// Does nucleus sampleing, in which we compute the cumulative distribution over all the
    /// options for each subsequent token in decreasing probability order and cut it off once
    /// it reaches a particular probability specified by the top_p. Defaults to -1, which
    /// disables it. Not that you should either alter *temperature* or *top_p* but not both.
    #[clap(long, value_parser = parse_top_p)]
    top_p: Option<f32>,
    /// Anthropic API Key to use. Will default to the environment variable `ANTHROPIC_API_KEY` if not set.
    #[arg(long, env = "ANTHROPIC_API_KEY")]
    #[serde(skip)]
    anthropic_api_key: String,
    /// Silent mode
    #[clap(short, long, action, default_value_t = false)]
    silent: bool,
    /// Wether to incrementally stream the response using SSE.
    #[clap(long)]
    stream: bool,
    /// Wether to pin this message to the message history.
    #[clap(long)]
    pin: bool,
    /// Response output format
    #[clap(short, long, default_value = "raw")]
    format: Option<crate::Output>,
}

/// The range of values for the `temperature` option which goes from 0 to 1.
const TEMPERATURE_RANGE: RangeInclusive<f32> = 0.0..=1.0;
/// The range of values for the `top_k` option which goes from 0 to Infinity.
const TOP_K_RANGE: RangeInclusive<f32> = 0.0..=f32::INFINITY;
/// The range of values for the `top_p` option which goes from 0 to 1.
const TOP_P_RANGE: RangeInclusive<f32> = 0.0..=1.0;

/// Parses the temperature value.
fn parse_temperature(s: &str) -> std::result::Result<f32, String> {
    let value = s.parse::<f32>().map_err(|_| {
        format!(
            "`{s}` must be a number between {} and {}",
            TEMPERATURE_RANGE.start(),
            TEMPERATURE_RANGE.end()
        )
    })?;
    if !TEMPERATURE_RANGE.contains(&value) {
        return Err(format!(
            "`{s}` must be a number between {} and {}",
            TEMPERATURE_RANGE.start(),
            TEMPERATURE_RANGE.end()
        ));
    }
    Ok(value)
}

/// Parses the top_k value.
fn parse_top_k(s: &str) -> std::result::Result<f32, String> {
    let value = s.parse::<f32>().map_err(|_| {
        format!(
            "`{s}` must be a number between {} and {}",
            TOP_K_RANGE.start(),
            TOP_K_RANGE.end()
        )
    })?;
    if !TOP_K_RANGE.contains(&value) {
        return Err(format!(
            "`{s}` must be a number between {} and {}",
            TOP_K_RANGE.start(),
            TOP_K_RANGE.end()
        ));
    }
    Ok(value)
}

/// Parses the top_p value.
fn parse_top_p(s: &str) -> std::result::Result<f32, String> {
    let value = s.parse::<f32>().map_err(|_| {
        format!(
            "`{s}` must be a number between {} and {}",
            TOP_P_RANGE.start(),
            TOP_P_RANGE.end()
        )
    })?;
    if !TOP_P_RANGE.contains(&value) {
        return Err(format!(
            "`{s}` must be a number between {} and {}",
            TOP_P_RANGE.start(),
            TOP_P_RANGE.end()
        ));
    }
    Ok(value)
}

/// Runs the `anthropic` command.
pub async fn run(mut options: Options) -> Result<()> {
    // Start the spinner animation
    let mut spinner = spinner::Spinner::new();

    // Finish parsing the options. Clap takes care of everything except reading the
    // user prompt from `stdin`.
    tracing::event!(tracing::Level::INFO, "Parsing prompt...");
    let prompt: Option<String> = if let Some(prompt) = options.prompt.take() {
        Some(if prompt == "-" {
            tracing::event!(tracing::Level::INFO, "Reading prompt from stdin...");
            let stdin = crate::utils::read_from_stdin()?.trim().to_string();
            stdin
        } else {
            prompt
        })
    } else {
        None
    };

    tracing::event!(tracing::Level::INFO, "Creating session...");
    // Create a new named or anonymous session.
    let mut session = Session::from(options)?;

    // Add the new prompt message to the session messages if one was provided.
    if let Some(prompt) = prompt {
        let message = Message::new(prompt, Role::Human, session.meta.pin);
        session.messages.push(message);
    }

    // Call the completion endpoint with the current session.
    if session.meta.stream {
        let mut acc: String = Default::default();
        let chunks = complete_stream(&session).await?;

        tokio::pin!(chunks);

        while let Some(chunk) = chunks.next().await {
            if chunk.is_err() {
                color_eyre::eyre::bail!("Error streaming response: {:?}", chunk);
            }

            let chunk = chunk.unwrap();
            tracing::event!(tracing::Level::DEBUG, "Received chunk... {:?}", chunk);

            spinner.print(&chunk.completion);

            acc.push_str(&chunk.completion);
        }
        // Add a new line at the end to make sure the prompt is on a new line.
        println!();

        // Save the response to the session.
        session.messages.push(Message::new(
            acc.trim().to_string(),
            Role::Assistant,
            session.meta.pin,
        ));
    } else {
        let response = complete(&session).await?;

        // Print the response output.
        print_output(&session.meta.format, &response)?;

        // Save the response to the session.
        session.messages.push(Message::new(
            response.completion.trim().to_string(),
            Role::Assistant,
            session.meta.pin,
        ));
    }

    // Save the session to a file.
    session.save()?;

    // Stop the spinner.
    spinner.stop();

    Ok(())
}

/// Completes the command by streaming the response.
async fn complete_stream(session: &Session) -> Result<impl Stream<Item = Result<Chunk>>> {
    tracing::event!(tracing::Level::INFO, "Serializing body...");
    let body = serde_json::to_string(&CompleteRequestBody {
        model: session.options.model.to_string(),
        max_tokens_to_sample: session.options.max_tokens_to_sample,
        stop_sequences: session.options.stop_sequences.clone(),
        temperature: session.options.temperature,
        top_k: session.options.top_k,
        top_p: session.options.top_p,
        stream: session.meta.stream,
        prompt: session.complete_prompt()?,
    })?;
    tracing::event!(tracing::Level::INFO, "body: {:?}", body);

    tracing::event!(tracing::Level::INFO, "Creating client...");
    let client = Client::new(session.meta.key.clone())?;

    let mut event_source = client.post_stream("/v1/complete", body).await?;

    let (tx, rx) = mpsc::channel(100);
    tracing::event!(tracing::Level::INFO, "Streaming output...");
    tokio::spawn(async move {
        while let Some(ev) = event_source.next().await {
            match ev {
                Err(e) => {
                    tracing::event!(tracing::Level::ERROR, "e: {e}");
                    if tx
                        .send(Err(color_eyre::eyre::format_err!(
                            "Error streaming response: {e}"
                        )))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                Ok(event) => match event {
                    reqwest_eventsource::Event::Open { .. } => {
                        tracing::event!(tracing::Level::INFO, "Open SSE stream...");
                    }
                    reqwest_eventsource::Event::Message(message) => {
                        tracing::event!(tracing::Level::INFO, "message: {:?}", message);

                        if message.data == "[DONE]" {
                            break;
                        }

                        if message.event != "completion" {
                            continue;
                        }

                        let chunk: Chunk = match serde_json::from_str(&message.data) {
                            Ok(chunk) => chunk,
                            Err(e) => {
                                tracing::event!(tracing::Level::ERROR, "e: {e}");
                                if tx
                                    .send(Err(color_eyre::eyre::format_err!(
                                        "Error parsing event: {e}"
                                    )))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                return;
                            }
                        };

                        if chunk.stop_reason.is_some() {
                            let stop_reason = chunk.stop_reason.clone().unwrap();

                            tracing::event!(
                                tracing::Level::INFO,
                                "Stopping stream due to stop_reason: {stop_reason}",
                            );

                            if stop_reason == "stop_sequence" {
                                tracing::event!(
                                    tracing::Level::INFO,
                                    "Found stop sequence: {}",
                                    &chunk.stop.unwrap()
                                );
                            }

                            break;
                        }

                        tracing::event!(tracing::Level::DEBUG, "chunk: {:?}", chunk);
                        if tx.send(Ok(chunk)).await.is_err() {
                            return;
                        }
                    }
                },
            }
        }
    });

    Ok(ReceiverStream::from(rx))
}

/// Completes the command without streaming the response.
async fn complete(session: &Session) -> Result<Response> {
    tracing::event!(tracing::Level::INFO, "Serializing body...");
    let body = serde_json::to_string(&CompleteRequestBody {
        model: session.options.model.to_string(),
        max_tokens_to_sample: session.options.max_tokens_to_sample,
        stop_sequences: session.options.stop_sequences.clone(),
        temperature: session.options.temperature,
        top_k: session.options.top_k,
        top_p: session.options.top_p,
        stream: session.meta.stream,
        prompt: session.complete_prompt()?,
    })?;
    tracing::event!(tracing::Level::INFO, "body: {:?}", body);

    tracing::event!(tracing::Level::INFO, "Creating client...");
    let client = Client::new(session.meta.key.clone())?;

    let res = client.post("/v1/complete", body).await?;
    tracing::event!(tracing::Level::INFO, "res: {:?}", res);

    let text = res.text().await?;
    tracing::event!(tracing::Level::INFO, "text: {:?}", text);

    let response: Response = serde_json::from_str(&text)?;
    tracing::event!(tracing::Level::INFO, "response: {:?}", response);

    Ok(response)
}

/// Prints the Response output according to the user options.
fn print_output(format: &crate::Output, response: &Response) -> Result<()> {
    match format {
        crate::Output::Raw => {
            println!("{}", response.completion);
        }
        crate::Output::Json => {
            let json = serde_json::to_string_pretty(&response)?;
            println!("{}", json);
        }
        crate::Output::Yaml => {
            let json = serde_yaml::to_string(&response)?;
            println!("{}", json);
        }
    }

    Ok(())
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CompletionOptions {
    /// The maximum number of tokens supported by the model.
    pub max_supported_tokens: Option<u32>,
    /// Controls which version of Claude answers your request. Two model families are exposed
    /// Claude and Claude Instant.
    pub model: String,
    /// A maximum number of tokens to generate before stopping.
    pub max_tokens_to_sample: u32,
    /// Claude models stop on `\n\nHuman:`, and may include additional built-in stops sequences
    /// in the future. By providing the `stop_sequences` parameter, you may include additional
    /// strings that will cause the model to stop generation.
    pub stop_sequences: Option<Vec<String>>,
    /// Amount of randomness injected into the response. Ranges from 0 to 1. Use temp closer to
    /// 0 for analytical/multiple choice, and temp closer to 1 for creative and generative
    /// tasks.
    pub temperature: Option<f32>,
    /// Only sample fromt the top `K` options of each subsequent token. Used to remove "long
    /// tail" low probability responses. Defaults to -1, which disables it.
    pub top_k: Option<f32>,
    /// Does nucleus sampleing, in which we compute the cumulative distribution over all the
    /// options for each subsequent token in decreasing probability order and cut it off once
    /// it reaches a particular probability specified by the top_p. Defaults to -1, which
    /// disables it. Not that you should either alter *temperature* or *top_p* but not both.
    pub top_p: Option<f32>,
}

/// Important data that depends on the command invocation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Meta {
    path: String,
    pub silent: bool,
    pub stream: bool,
    pub pin: bool,
    pub key: String,
    pub format: crate::Output,
}

impl Meta {
    pub fn new(path: String) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }
}

/// Represents a chat session.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Session {
    id: String,
    pub messages: Vec<Message>,
    pub options: CompletionOptions,
    #[serde(skip)]
    pub meta: Meta,
}

impl Session {
    /// Creates a new session
    fn new(id: String, path: String) -> Self {
        Self {
            id,
            meta: Meta::new(path),
            ..Default::default()
        }
    }

    /// Creates a new session from the anthropic command options.
    pub fn from(options: Options) -> Result<Self> {
        let mut session = if let Some(id) = options.session.clone() {
            Self::load(id)?
        } else {
            let id = Ulid::new().to_string();
            let home = env::var("C_ROOT").unwrap_or(env::var("HOME")?);
            let path = format!("{home}/.c/sessions/anonymous/{id}.yaml");
            Self::new(id, path)
        };

        session.merge_options(options)?;

        Ok(session)
    }

    /// Tries to load a session from the filesystem.
    pub fn load(id: String) -> Result<Self> {
        let home = env::var("C_ROOT").unwrap_or(env::var("HOME")?);
        let path = format!("{home}/.c/sessions/{id}.yaml");

        let meta = Meta {
            path: path.clone(),
            ..Default::default()
        };

        let session = if fs::metadata(&path).is_ok() {
            let mut session: Session = serde_yaml::from_str(&fs::read_to_string(&path)?)?;
            session.meta = meta;
            session
        } else {
            Self::new(id, path)
        };

        Ok(session)
    }

    /// Merges an options object into the session options.
    pub fn merge_options(&mut self, options: Options) -> Result<()> {
        if options.model.is_some() {
            self.options.model = options.model.unwrap().as_str().to_string();
        }

        if options.max_tokens_to_sample.is_some() {
            self.options.max_tokens_to_sample = options.max_tokens_to_sample.unwrap_or(1000);
        }

        if options.max_supported_tokens.is_some() {
            self.options.max_supported_tokens = options.max_supported_tokens;
        }

        if options.temperature.is_some() {
            self.options.temperature = options.temperature;
        }

        if options.top_k.is_some() {
            self.options.top_k = options.top_k;
        }

        if options.top_p.is_some() {
            self.options.top_p = options.top_p;
        }

        if options.stop_sequences.is_some() {
            self.options.stop_sequences = options.stop_sequences;
        }

        if options.format.is_some() {
            self.meta.format = options.format.unwrap();
        }

        self.meta.key = options.anthropic_api_key;
        self.meta.stream = options.stream;
        self.meta.silent = options.silent;
        self.meta.pin = options.pin;

        Ok(())
    }

    /// Saves the session to the filesystem.
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

    /// Returns a valid completion prompt from the list of messages.
    pub fn complete_prompt(&self) -> Result<String> {
        let max =
            self.options.max_supported_tokens.unwrap_or(4096) - self.options.max_tokens_to_sample;
        let mut messages = self.messages.clone();
        messages.push(Message::new("".to_string(), Role::Assistant, false));

        tracing::event!(
            tracing::Level::INFO,
            "Creating a complete prompt that is less than {max} tokens long"
        );

        let prompt = loop {
            let prompt = join_messages(&messages);
            let tokens = token_length(&prompt) as u32;

            if tokens <= max {
                tracing::event!(
                    tracing::Level::INFO,
                    "Tokens ({tokens}) is less than max ({max}). Returning prompt",
                );
                break prompt;
            }

            if let Some((index, _)) = messages.iter().enumerate().find(|(_, m)| !m.pin) {
                tracing::event!(
                    tracing::Level::INFO,
                    "Tokens ({tokens}) is greater than max ({max}). Trying again with fewer messages",
                );
                messages.remove(index);
            } else {
                Err(color_eyre::eyre::format_err!(
                    "The prompt is larger than {max} and there are no messages to remove"
                ))?;
            }
        };

        Ok(prompt)
    }
}

/// Join messages
fn join_messages(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|m| format!("\n\n{:?}: {}", m.role, m.content))
        .collect::<Vec<String>>()
        .join("")
}

/// Chacks if a directory exists.
pub fn directory_exists(dir_name: &str) -> bool {
    let p = path::Path::new(dir_name);
    p.exists() && p.is_dir()
}

/// Token language of a prompt.
/// TODO: Make this better!
fn token_length(prompt: &str) -> usize {
    let words = prompt.split_whitespace().rev().collect::<Vec<&str>>();

    // Estimate the total tokens by multiplying words by 4/3
    words.len() * 4 / 3
}
