use std::{ops::RangeInclusive, time::Duration};

use clap::{Parser, ValueEnum};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

use crate::session::{Message, Role, Session, Vendor};

#[derive(ValueEnum, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum Model {
    #[default]
    ChatBison,
    CodechatBison,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ChatBison => "chat-bison",
            Self::CodechatBison => "codechat-bison",
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            Self::ChatBison => 8000,
            Self::CodechatBison => 6000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Author {
    #[default]
    Bot,
    User,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct VertexMessage {
    content: String,
    author: Author,
}

impl From<Message> for VertexMessage {
    fn from(message: Message) -> Self {
        VertexMessage {
            author: if message.role == Role::Human || message.role == Role::User {
                Author::User
            } else {
                Author::Bot
            },
            content: message.content,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Instance {
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
    messages: Vec<VertexMessage>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RequestOptions {
    instances: Vec<Instance>,
    parameters: Parameters,
}

impl From<CommandOptions> for RequestOptions {
    fn from(options: CommandOptions) -> Self {
        Self {
            instances: vec![Instance {
                context: None,
                messages: Vec::new(),
            }],
            parameters: Parameters {
                temperature: options.temperature,
                max_output_tokens: options.max_output_tokens,
                top_p: options.top_p,
                top_k: options.top_k,
            },
        }
    }
}

impl From<Session<SessionOptions>> for RequestOptions {
    fn from(session: Session<SessionOptions>) -> Self {
        Self {
            instances: session
                .history
                .iter()
                .map(|message| Instance {
                    context: None,
                    messages: vec![VertexMessage {
                        content: message.content.clone(),
                        author: match message.role {
                            Role::Human => Author::User,
                            Role::User => Author::User,
                            Role::System => Author::Bot,
                            Role::Assistant => Author::Bot,
                        },
                    }],
                })
                .collect(),
            parameters: Parameters {
                temperature: session.options.temperature,
                max_output_tokens: session.options.max_output_tokens,
                top_p: session.options.top_p,
                top_k: session.options.top_k,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SessionOptions {
    endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<Model>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

impl From<CommandOptions> for SessionOptions {
    fn from(options: CommandOptions) -> Self {
        Self {
            endpoint: format!("https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/us-central1/publishers/google/models/{}:predict", options.gcp_region, options.gcp_project, options.model.unwrap().as_str()),
            context: None,
            model: None,
            max_output_tokens: options.max_output_tokens,
            temperature: options.temperature,
            top_k: options.top_k,
            top_p: options.top_p,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Candidate {
    author: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    start_index: u32,
    end_index: u32,
    url: String,
    title: String,
    license: String,
    publication_date: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CitationMeta {
    citations: Vec<Citation>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SafetyAttributes {
    categories: Vec<String>,
    blocked: bool,
    scores: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Prediction {
    candidates: Vec<Candidate>,
    citation_metadata: Vec<CitationMeta>,
    safety_attributes: Vec<SafetyAttributes>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenCount {
    total_tokens: u32,
    total_billable_characters: u32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    output_token_count: TokenCount,
    input_token_count: TokenCount,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    token_metadata: TokenMetadata,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Response {
    predictions: Vec<Prediction>,
    metadata: Metadata,
}

#[derive(Default, Clone, Parser, Debug, Serialize, Deserialize)]
pub struct CommandOptions {
    /// The prompt you want Claude to complete.
    prompt: Option<String>,
    /// The context you want to provide.
    context: Option<String>,
    /// Chat session name. Will be used to store previous session interactions.
    #[arg(long)]
    session: Option<String>,
    /// The maximum number of tokens supported by the model.
    #[arg(long)]
    max_supported_tokens: Option<u32>,
    /// Controls the chat model
    #[clap(short, long, value_enum, default_value = "chat-bison")]
    model: Option<Model>,
    /// A maximum number of tokens to generate before stopping.
    #[arg(long, default_value = "1000")]
    max_output_tokens: Option<u32>,
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
    #[arg(long, env = "C_GCP_KEY")]
    #[serde(skip)]
    #[clap(long, default_value = "INVALID KEY")]
    gcp_key: String,
    #[arg(long, env = "C_GCP_REGION")]
    #[serde(skip)]
    #[clap(long, default_value = "us-central1")]
    gcp_region: String,
    #[arg(long, env = "C_GCP_PROJECT")]
    #[serde(skip)]
    #[clap(long, default_value = "root")]
    gcp_project: String,
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

/// Runs the `vertex api` command.
pub async fn run(mut options: CommandOptions) -> Result<()> {
    // Start the spinner animation
    let mut spinner = spinner::Spinner::new();

    // Finish parsing the options. Clap takes care of everything except reading the
    // user prompt from `stdin`.
    tracing::event!(tracing::Level::INFO, "Parsing prompt...");
    let prompt: Option<String> = if let Some(prompt) = options.prompt.clone() {
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

    // Get the RequestBody options from the command options.
    let session_options: SessionOptions = options.clone().into();

    // Create a new session.
    // If the user provided a session name then we need to check if it exists.
    let session: Session<SessionOptions> = if let Some(session) = options.session.take() {
        tracing::event!(tracing::Level::INFO, "Checking if session exists...");
        if Session::<SessionOptions>::exists(&session) {
            tracing::event!(tracing::Level::INFO, "Session exists, loading...");
            let session: Session<SessionOptions> = Session::load(&session)?;
            session
        } else {
            tracing::event!(tracing::Level::INFO, "Session does not exist, creating...");
            let session: Session<SessionOptions> = Session::new(
                session,
                Vendor::Google,
                session_options,
                options.model.unwrap_or(Model::default()).as_u32(),
            );
            session
        }
    } else {
        tracing::event!(tracing::Level::INFO, "Creating anonymous session...");
        let session: Session<SessionOptions> = Session::anonymous(
            Vendor::Google,
            session_options,
            options.model.unwrap_or(Model::default()).as_u32(),
        );
        session
    };

    tracing::event!(tracing::Level::INFO, "Mergin command options...");
    // Create a new named or anonymous session.
    let mut session = merge_options(session, options)?;

    // Add the new prompt message to the session messages if one was provided.
    if let Some(prompt) = prompt {
        let message = Message::new(prompt, Role::Human, session.meta.pin);
        session.history.push(message);
    }

    let response = complete(&session).await?;

    // Print the response output.
    print_output(&session.meta.format, &response)?;

    // Save the response to the session.
    session.history.push(Message::new(
        response
            .predictions
            .first()
            .unwrap()
            .candidates
            .first()
            .unwrap()
            .content
            .to_string(),
        Role::Assistant,
        session.meta.pin,
    ));

    // Save the session to a file.
    session.save()?;

    // Stop the spinner.
    spinner.stop();

    Ok(())
}

/// Returns a valid list of messages for the completion to work.
pub fn complete_messages(
    messages: Vec<Message>,
    max_supported_tokens: u32,
    max_tokens_to_sample: u32,
) -> Result<Vec<VertexMessage>> {
    let max = max_supported_tokens - max_tokens_to_sample;
    let messages = trim_messages(messages, max)?;

    Ok(messages.into_iter().map(VertexMessage::from).collect())
}

/// Trim messages until the total number of tokens inside is less than the maximum.
fn trim_messages(mut messages: Vec<Message>, max: u32) -> Result<Vec<Message>> {
    let total_tokens: usize = messages.iter().map(|m| m.content.len() * 4 / 3).sum();

    if total_tokens as u32 <= max {
        return Ok(messages);
    }

    if let Some((index, _)) = messages
        .iter()
        .enumerate()
        .find(|(_, m)| m.role != Role::System && !m.pin)
    {
        messages.remove(index);
        trim_messages(messages, max)
    } else {
        Err(color_eyre::eyre::format_err!(
            "Could not trim messages to fit the maximum number of tokens."
        ))
    }
}

/// Merges an options object into the session options.
pub fn merge_options(
    mut session: Session<SessionOptions>,
    options: CommandOptions,
) -> Result<Session<SessionOptions>> {
    if options.model.is_some() {
        session.options.model = options.model;
        session.max_supported_tokens = options.model.unwrap().as_u32();
    }

    if options.max_supported_tokens.is_some() {
        session.max_supported_tokens = options.max_supported_tokens.unwrap();
    }

    if options.temperature.is_some() {
        session.options.temperature = options.temperature;
    }

    if options.top_k.is_some() {
        session.options.top_k = options.top_k;
    }

    if options.top_p.is_some() {
        session.options.top_p = options.top_p;
    }

    if options.format.is_some() {
        session.meta.format = options.format.unwrap();
    }

    session.meta.key = options.gcp_key;
    session.meta.stream = false;
    session.meta.silent = options.silent;
    session.meta.pin = options.pin;

    Ok(session)
}

/// Completes the command without streaming the response.
async fn complete(session: &Session<SessionOptions>) -> Result<Response> {
    tracing::event!(tracing::Level::INFO, "Serializing body...");
    let mut body: RequestOptions = std::convert::Into::<RequestOptions>::into(session.clone());

    let max_output_tokens = session.options.max_output_tokens.unwrap_or(1000);
    body.instances.first_mut().unwrap().messages = complete_messages(
        session.history.clone(),
        session.max_supported_tokens,
        max_output_tokens,
    )?;
    tracing::event!(tracing::Level::INFO, "body: {:?}", body);

    tracing::event!(tracing::Level::INFO, "Creating client...");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    tracing::event!(tracing::Level::INFO, "Creating API client headers...");
    let mut headers = reqwest::header::HeaderMap::new();

    let authorization =
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", session.meta.key.clone()))?;
    let content_type = reqwest::header::HeaderValue::from_static("application/json");

    headers.insert(reqwest::header::AUTHORIZATION, authorization);
    headers.insert(reqwest::header::CONTENT_TYPE, content_type);

    let json = serde_json::to_string(&body)?;

    tracing::event!(
        tracing::Level::INFO,
        "Sending request to {}",
        &session.options.endpoint
    );

    let res = client
        .post(&session.options.endpoint)
        .headers(headers)
        .body(json.clone())
        .send()
        .await?;

    tracing::event!(tracing::Level::INFO, "res: {:?}", res);

    let text = res.text().await?;
    tracing::event!(tracing::Level::INFO, "text: {:?}", text);

    let response: Response = serde_json::from_str(&text).map_err(|e| {
        tracing::event!(tracing::Level::ERROR, "Error parsing response text.");
        tracing::event!(tracing::Level::ERROR, "json: {json}");
        tracing::event!(tracing::Level::ERROR, "text: {text}");
        color_eyre::eyre::format_err!("error: {e}")
    })?;
    tracing::event!(tracing::Level::INFO, "response: {:?}", response);

    Ok(response)
}

/// Prints the Response output according to the user options.
fn print_output(format: &crate::Output, response: &Response) -> Result<()> {
    match format {
        crate::Output::Raw => {
            println!(
                "{}",
                response
                    .predictions
                    .first()
                    .unwrap()
                    .candidates
                    .first()
                    .unwrap()
                    .content
            );
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
