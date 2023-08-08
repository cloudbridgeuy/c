use std::env;
use std::fs;
use std::ops::RangeInclusive;
use std::path;

use clap::{Parser, ValueEnum};
use color_eyre::eyre::Result;
use gpt_tokenizer::Default as DefaultTokenizer;
use openai::client::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionOptions {
    pub model: String,
    /// The maximum number of tokens to generate in the completion.
    pub max_tokens: Option<u32>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the
    /// output more random, while lower valies like 0.2 will make it more focused and
    /// deterministic. It's generally recommended to alter this or `top_p` but not both.
    pub temperature: Option<f32>,
    /// An alternative sampling with temperature, called nucleus sampling, where the model
    /// considers the results of the tokens with `top_p` probability mass. So, 0.1 means only
    /// the tokens comprising the top 10% probability mass are considered. It's generally
    /// recommended to alter this or `temperature` but not both.
    pub top_p: Option<f32>,
    /// How many completions to generate for each prompt.
    pub n: Option<u32>,
    /// Up to 4 sequences where the API will stop generating further tokens. The returned text
    /// will not contain the stop sequence.
    pub stop: Option<Vec<String>>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they
    /// appear in the text so far, increasing the model's likelihood to talk about new topics.
    pub presence_penalty: Option<f32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their
    /// existing frequency in the text so far, decreasing the model's likelihood to repeat the
    /// same line verbatim.
    pub frequency_penalty: Option<f32>,
    /// Modify the likelihood of specified tokens appearing in the completion.
    pub logit_bias: Option<Vec<(u32, f32)>>,
    /// A user identifier representing your end-user, which can help OpenAI to monitor and
    /// detect abuse.
    pub user: Option<String>,
    /// The minimum available tokens left to the Model to construct the completion message.
    pub min_available_tokens: Option<u32>,
    /// The maximum number of tokens supporte by the model.
    pub max_supported_tokens: Option<u32>,
    /// A list of functions the model may generate JSON inputs for, provided as JSON.
    pub functions: Option<String>,
    /// Controls how the model responds to function calls. "none" means the model does not call
    /// a function, and responds to the end-user. "auto" means the model can pick between an
    /// end-user or calling a function. Specifying a particular function via `{"name":
    /// "my_function" }` forces the model to call that function. "none" is the default when no
    /// functions are present. "auto" is the default if functions are present.
    pub function_call: Option<String>,
}

/// Important data that depends on the command invocation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    id: String,
    pub messages: Vec<HistoryMessage>,
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

        if options.max_tokens.is_some() {
            self.options.max_tokens = options.max_tokens;
        }

        if options.max_supported_tokens.is_some() {
            self.options.max_supported_tokens = options.max_supported_tokens;
        }

        if options.temperature.is_some() {
            self.options.temperature = options.temperature;
        }

        if options.top_p.is_some() {
            self.options.top_p = options.top_p;
        }

        if options.stop.is_some() {
            self.options.stop = options.stop;
        }

        if options.presence_penalty.is_some() {
            self.options.presence_penalty = options.presence_penalty;
        }

        if options.frequency_penalty.is_some() {
            self.options.frequency_penalty = options.frequency_penalty;
        }

        if options.logit_bias.is_some() {
            self.options.logit_bias = options.logit_bias;
        }

        if options.user.is_some() {
            self.options.user = options.user;
        }

        if options.min_available_tokens.is_some() {
            self.options.min_available_tokens = options.min_available_tokens;
        }

        if options.format.is_some() {
            self.meta.format = options.format.unwrap();
        }

        if options.system.is_some() {
            if let Some(m) = self.messages.first_mut() {
                if m.role == Role::System {
                    m.content = options.system.unwrap();
                }
            } else {
                self.messages.insert(
                    0,
                    HistoryMessage {
                        content: options.system.unwrap(),
                        role: Role::System,
                        pin: true,
                    },
                );
            }
        }

        self.meta.key = options.openai_api_key;
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

    /// Returns a valid list of messages for the completion to work.
    pub fn complete_messages(&self) -> Result<Vec<Message>> {
        let tokenizer = DefaultTokenizer::new();
        let min = std::cmp::max(
            self.options.min_available_tokens.unwrap_or(1000),
            self.options.max_tokens.unwrap_or(0),
        );
        let max = self.options.max_supported_tokens.unwrap_or(4096) - min;
        let messages = trim_messages(self.messages.clone(), max, &tokenizer)?;

        Ok(messages)
    }
}

/// Trim messages until the total number of tokens inside is less than the maximum.
fn trim_messages(
    mut messages: Vec<HistoryMessage>,
    max: u32,
    tokenizer: &DefaultTokenizer,
) -> Result<Vec<Message>> {
    let total_tokens: usize = messages
        .iter()
        .map(|m| tokenizer.encode(&m.content).len())
        .sum();

    if total_tokens as u32 <= max {
        let messages: Vec<Message> = messages.into_iter().map(Message::from).collect();

        return Ok(messages);
    }

    if let Some((index, _)) = messages
        .iter()
        .enumerate()
        .find(|(_, m)| m.role != Role::System && !m.pin)
    {
        messages.remove(index);
        trim_messages(messages, max, tokenizer)
    } else {
        Err(color_eyre::eyre::format_err!(
            "Could not trim messages to fit the maximum number of tokens."
        ))
    }
}

/// Chacks if a directory exists.
pub fn directory_exists(dir_name: &str) -> bool {
    let p = path::Path::new(dir_name);
    p.exists() && p.is_dir()
}

#[derive(ValueEnum, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
enum Model {
    #[default]
    #[serde(rename = "gpt-4")]
    GPT4,
    #[serde(rename = "gpt-4-32k")]
    GPT432K,
    #[serde(rename = "gpt-3.5-turbo")]
    GPT35Turbo,
    #[serde(rename = "gpt-3.5-turbo-16k")]
    GPT35Turbo16K,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::GPT4 => "gpt-4",
            Model::GPT432K => "gpt-4-32k",
            Model::GPT35Turbo => "gpt-3.5-turbo",
            Model::GPT35Turbo16K => "gpt-3.5-turbo-16k",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CompleteRequestBody {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<Vec<(u32, f32)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Response {
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
    pub message: Message,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Default, Clone, Parser, Debug, Serialize, Deserialize)]
pub struct Options {
    /// The content of the message to be sent to the chatbot. You can also populate this value
    /// from stdin. If you pass a value here and pipe data from stdin, both will be sent to the
    /// API, stdin taking precedence.
    prompt: Option<String>,
    /// ID of the model to use. See the following link: https://platform.openai.com/docs/models/overview
    #[clap(short, long, value_enum, default_value = "gpt4")]
    model: Option<Model>,
    /// Chat session name. Will be used to store previous session interactions.
    #[arg(long)]
    session: Option<String>,
    /// The system message helps set the behavior of the assistant.
    #[arg(long)]
    system: Option<String>,
    /// The system prompt to use for the chat. It's always sent as the first message of any
    /// chat request.
    // #[arg(long)]
    // system: Option<String>,
    /// The maximum number of tokens to generate in the completion.
    #[arg(long)]
    max_tokens: Option<u32>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the
    /// output more random, while lower valies like 0.2 will make it more focused and
    /// deterministic. It's generally recommended to alter this or `top_p` but not both.
    #[clap(long, value_parser = parse_temperature)]
    temperature: Option<f32>,
    /// An alternative sampling with temperature, called nucleus sampling, where the model
    /// considers the results of the tokens with `top_p` probability mass. So, 0.1 means only
    /// the tokens comprising the top 10% probability mass are considered. It's generally
    /// recommended to alter this or `temperature` but not both.
    #[clap(long, value_parser = parse_top_p)]
    top_p: Option<f32>,
    /// How many completions to generate for each prompt.
    #[arg(long)]
    n: Option<u32>,
    /// Up to 4 sequences where the API will stop generating further tokens. The returned text
    /// will not contain the stop sequence.
    #[arg(long)]
    stop: Option<Vec<String>>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they
    /// appear in the text so far, increasing the model's likelihood to talk about new topics.
    #[arg(long)]
    presence_penalty: Option<f32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their
    /// existing frequency in the text so far, decreasing the model's likelihood to repeat the
    /// same line verbatim.
    #[arg(long)]
    frequency_penalty: Option<f32>,
    /// Modify the likelihood of specified tokens appearing in the completion.
    #[arg(long, value_parser = parse_key_val::<u32, f32>)]
    logit_bias: Option<Vec<(u32, f32)>>,
    /// A user identifier representing your end-user, which can help OpenAI to monitor and
    /// detect abuse.
    #[arg(long)]
    user: Option<String>,
    /// The minimum available tokens left to the Model to construct the completion message.
    #[arg(long)]
    min_available_tokens: Option<u32>,
    /// The maximum number of tokens supporte by the model.
    #[arg(long)]
    max_supported_tokens: Option<u32>,
    /// OpenAI API Key to use. Will default to the environment variable `OPENAI_API_KEY` if not set.
    #[arg(long, env = "OPENAI_API_KEY")]
    #[serde(skip)]
    openai_api_key: String,
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
const TEMPERATURE_RANGE: RangeInclusive<f32> = 0.0..=2.0;
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

/// Parse a single key-value pair
fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid key-value pair: {}", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

/// Runs the `openai` command.
pub async fn run(mut options: Options) -> Result<()> {
    // Start the spinner animation
    let mut spinner = spinner::Spinner::new();

    // Finish parsing the options. Clap takes care of everything except reading the user prompt
    // from `stdin`.
    tracing::event!(tracing::Level::INFO, "Parsing options");
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

    // Add the new prompt message to the session messages, if one was provided.
    if let Some(prompt) = prompt {
        let message = HistoryMessage::new(prompt, Role::User, session.meta.pin);
        session.messages.push(message);
    }

    // Call the completion endpoint with the current session.
    if session.meta.stream {
        let mut acc: String = String::new();
        let chunks = complete_stream(&session).await?;

        tokio::pin!(chunks);

        while let Some(chunk) = chunks.next().await {
            if chunk.is_err() {
                color_eyre::eyre::bail!("Error streaming response: {:?}", chunk);
            }

            let chunk = chunk.unwrap();

            if let Some(choice) = &chunk.choices.get(0) {
                if let Some(delta) = &choice.delta {
                    if let Some(content) = &delta.content {
                        acc.push_str(content);
                        spinner.print(content);
                    }
                }
            }
        }

        // Add a new line at the end to make sure the prompt is on a new line.
        println!();

        // Save the response to the session.
        session
            .messages
            .push(HistoryMessage::new(acc, Role::Assistant, session.meta.pin));
    } else {
        let response = complete(&session).await?;

        // Print the response output.
        print_output(&session.meta.format, &response)?;

        // Save the response to the session
        session.messages.push(HistoryMessage::new(
            response.choices.get(0).unwrap().message.content.clone(),
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
        max_tokens: session.options.max_tokens,
        stop: session.options.stop.clone(),
        temperature: session.options.temperature,
        top_p: session.options.top_p,
        n: session.options.n,
        logit_bias: session.options.logit_bias.clone(),
        stream: session.meta.stream,
        frequency_penalty: session.options.frequency_penalty,
        presence_penalty: session.options.presence_penalty,
        user: session.options.user.clone(),
        messages: session.complete_messages()?,
    })?;
    tracing::event!(tracing::Level::INFO, "body: {:?}", body);

    tracing::event!(tracing::Level::INFO, "Creating client...");
    let client = Client::new(session.meta.key.clone())?;

    let mut event_source = client.post_stream("/v1/chat/completions", body).await?;

    let (tx, rx) = mpsc::channel(100);

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
                        tracing::event!(tracing::Level::DEBUG, "message: {:?}", message);

                        if message.data == "[DONE]" {
                            break;
                        }

                        let chunk: Chunk = match serde_json::from_str(&message.data) {
                            Ok(chunk) => chunk,
                            Err(e) => {
                                tracing::event!(tracing::Level::ERROR, "e: {e}");
                                if tx
                                    .send(Err(color_eyre::eyre::format_err!(
                                        "Error parsing response: {e}"
                                    )))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                return;
                            }
                        };

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
        max_tokens: session.options.max_tokens,
        stop: session.options.stop.clone(),
        temperature: session.options.temperature,
        top_p: session.options.top_p,
        n: session.options.n,
        logit_bias: session.options.logit_bias.clone(),
        stream: session.meta.stream,
        frequency_penalty: session.options.frequency_penalty,
        presence_penalty: session.options.presence_penalty,
        user: session.options.user.clone(),
        messages: session.complete_messages()?,
    })?;
    tracing::event!(tracing::Level::INFO, "body: {:?}", body);

    tracing::event!(tracing::Level::INFO, "Creating client...");
    let client = Client::new(session.meta.key.clone())?;

    let res = client.post("/v1/chat/completions", body.clone()).await?;
    tracing::event!(tracing::Level::INFO, "res: {:?}", res);

    let text = res.text().await?;
    tracing::event!(tracing::Level::INFO, "text: {:?}", text);

    let response: Response = serde_json::from_str(&text).map_err(|e| {
        tracing::event!(tracing::Level::ERROR, "Error parsing response text.");
        tracing::event!(tracing::Level::ERROR, "body: {body}");
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
            println!("{}", response.choices[0].message.content);
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
