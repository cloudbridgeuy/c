use clap::{Parser, ValueEnum};
use color_eyre::eyre::Result;
use openai::client::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

use crate::session::{Message, Role, Session, Vendor};

/// Stores a message.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionMessage {
    pub role: Role,
    pub content: String,
}

impl CompletionMessage {
    /// Creates a new message.
    pub fn new(content: String, role: Role) -> Self {
        Self { content, role }
    }
}

impl From<Message> for CompletionMessage {
    fn from(message: Message) -> Self {
        CompletionMessage {
            role: if message.role == Role::Human {
                Role::User
            } else {
                message.role
            },
            content: message.content,
        }
    }
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

    pub fn as_u32(&self) -> u32 {
        match self {
            Model::GPT4 => 8000,
            Model::GPT432K => 32000,
            Model::GPT35Turbo => 4000,
            Model::GPT35Turbo16K => 16000,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RequestOptions {
    model: String,
    messages: Vec<CompletionMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logit_bias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Default, Clone, Parser, Debug, Serialize, Deserialize)]
pub struct CommandOptions {
    /// The content of the message to be sent to the chatbot. You can also populate this value
    /// from stdin. If you pass a value here and pipe data from stdin, both will be sent to the
    /// API, stdin taking precedence.
    prompt: Option<String>,
    /// ID of the model to use. See the following link: https://platform.openai.com/docs/models/overview
    #[clap(short, long, value_enum)]
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
    #[clap(long, value_parser = crate::utils::parse_temperature)]
    temperature: Option<f32>,
    /// An alternative sampling with temperature, called nucleus sampling, where the model
    /// considers the results of the tokens with `top_p` probability mass. So, 0.1 means only
    /// the tokens comprising the top 10% probability mass are considered. It's generally
    /// recommended to alter this or `temperature` but not both.
    #[clap(long, value_parser = crate::utils::parse_top_p)]
    top_p: Option<f32>,
    /// How many completions to generate for each prompt.
    #[arg(long)]
    n: Option<u32>,
    /// Up to 4 sequences where the API will stop generating further tokens. The returned text
    /// will not contain the stop sequence.
    #[arg(long)]
    stop: Option<String>,
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
    #[arg(long)]
    logit_bias: Option<String>,
    /// A user identifier representing your end-user, which can help OpenAI to monitor and
    /// detect abuse.
    #[arg(long)]
    user: Option<String>,
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
    /// Max history size to add to the user prompt.
    #[clap(long)]
    max_history: Option<u32>,
    /// Whether to save the prompt and response to the history file.
    #[clap(long)]
    nosave: bool,
    /// Number of messages to keep in the history. Pinned messages are not counted.
    #[clap(long)]
    history_size: Option<usize>,
}

impl From<CommandOptions> for RequestOptions {
    fn from(options: CommandOptions) -> Self {
        Self {
            model: options
                .model
                .as_ref()
                .unwrap_or(&Model::default())
                .as_str()
                .to_string(),
            messages: Vec::new(),
            max_tokens: Some(options.max_tokens.unwrap_or(1000)),
            stop: options.stop,
            stream: options.stream,
            temperature: options.temperature,
            top_p: options.top_p,
            n: options.n,
            presence_penalty: options.presence_penalty,
            frequency_penalty: options.frequency_penalty,
            logit_bias: options.logit_bias,
            user: options.user,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl From<RequestOptions> for SessionOptions {
    fn from(options: RequestOptions) -> Self {
        Self {
            model: Some(options.model),
            max_tokens: options.max_tokens,
            stop: options.stop,
            temperature: options.temperature,
            top_p: options.top_p,
            n: options.n,
            presence_penalty: options.presence_penalty,
            frequency_penalty: options.frequency_penalty,
            logit_bias: options.logit_bias,
            user: options.user,
        }
    }
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
    pub message: CompletionMessage,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Runs the `openai` command.
pub async fn run(mut options: CommandOptions) -> Result<()> {
    // Start the spinner animation
    let mut spinner = spinner::Spinner::new();

    // Finish parsing the options. Clap takes care of everything except reading the user prompt
    // from `stdin`.
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

    // Get the RequestBody options from the command options.
    let request_options: RequestOptions = options.clone().into();
    let session_options: SessionOptions = request_options.clone().into();

    // Create a new session
    // If the user provided a session name then we need to check it exist.
    let session: Session<SessionOptions> = if let Some(session) = options.session.take() {
        tracing::event!(tracing::Level::INFO, "Checking if session exists...");
        if Session::<RequestOptions>::exists(&session) {
            tracing::event!(tracing::Level::INFO, "Session exists, loading...");
            let session: Session<SessionOptions> = Session::load(&session)?;
            session
        } else {
            tracing::event!(tracing::Level::INFO, "Session does not exist, creating...");
            let session: Session<SessionOptions> = Session::new(
                session,
                Vendor::OpenAI,
                session_options,
                options.model.unwrap_or(Model::default()).as_u32(),
            );
            session
        }
    } else {
        tracing::event!(tracing::Level::INFO, "Creating anonymous session...");
        let session: Session<SessionOptions> = Session::anonymous(
            Vendor::OpenAI,
            session_options,
            options.model.unwrap_or(Model::default()).as_u32(),
        );
        session
    };

    tracing::event!(tracing::Level::INFO, "Merging command options...");
    // Create a new named or anonymous session.
    let mut session = merge_options(session, options)?;

    // Add the new prompt message to the session messages, if one was provided.
    if let Some(prompt) = prompt {
        let message = Message::new(prompt, Role::User, session.meta.pin);
        session.history.push(message);
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
            .history
            .push(Message::new(acc, Role::Assistant, session.meta.pin));
    } else {
        let response = complete(&session).await?;

        // Stop the spinner.
        spinner.stop();

        // Print the response output.
        print_output(&session.meta.format, &response)?;

        // Save the response to the session
        session.history.push(Message::new(
            response.choices.get(0).unwrap().message.content.clone(),
            Role::Assistant,
            session.meta.pin,
        ));
    }

    // Save the session to a file.
    if session.meta.save {
        if session.meta.history_size.is_some() && session.meta.history_size.unwrap() > 0 {
            session.history =
                crate::utils::filter_history(&session.history, session.meta.history_size.unwrap());
        }

        session.save()?;
    }

    Ok(())
}

/// Completes the command by streaming the response.
async fn complete_stream(
    session: &Session<SessionOptions>,
) -> Result<impl Stream<Item = Result<Chunk>>> {
    let body = create_body(session)?;
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
async fn complete(session: &Session<SessionOptions>) -> Result<Response> {
    let body = create_body(session)?;

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
pub fn print_output(format: &crate::Output, response: &Response) -> Result<()> {
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

/// Merges an options object into the session options.
pub fn merge_options(
    mut session: Session<SessionOptions>,
    options: CommandOptions,
) -> Result<Session<SessionOptions>> {
    if options.model.is_some() {
        session.options.model = Some(options.model.unwrap_or_default().as_str().to_string());
        session.max_supported_tokens = options.model.unwrap().as_u32();
    }

    if options.max_tokens.is_some() {
        session.options.max_tokens = options.max_tokens;
    }

    if options.max_supported_tokens.is_some() {
        session.max_supported_tokens = options.max_supported_tokens.unwrap();
    }

    if options.max_history.is_some() {
        session.max_history = options.max_history;
    }

    if options.temperature.is_some() {
        session.options.temperature = options.temperature;
    }

    if options.top_p.is_some() {
        session.options.top_p = options.top_p;
    }

    if options.stop.is_some() {
        session.options.stop = options.stop;
    }

    if options.presence_penalty.is_some() {
        session.options.presence_penalty = options.presence_penalty;
    }

    if options.frequency_penalty.is_some() {
        session.options.frequency_penalty = options.frequency_penalty;
    }

    if options.logit_bias.is_some() {
        session.options.logit_bias = options.logit_bias;
    }

    if options.user.is_some() {
        session.options.user = options.user;
    }

    if options.format.is_some() {
        session.meta.format = options.format.unwrap();
    }

    if options.system.is_some() {
        if let Some(m) = session.history.first_mut() {
            if m.role == Role::System {
                m.content = options.system.unwrap();
            }
        } else {
            session.history.insert(
                0,
                Message {
                    content: options.system.unwrap(),
                    role: Role::System,
                    pin: true,
                },
            );
        }
    }

    if options.history_size.is_some() {
        session.meta.history_size = options.history_size;
    }

    session.meta.save = !options.nosave;
    session.meta.key = options.openai_api_key;
    session.meta.stream = options.stream;
    session.meta.silent = options.silent;
    session.meta.pin = options.pin;

    Ok(session)
}

/// Returns a valid list of messages for the completion to work.
pub fn complete_messages(
    messages: Vec<Message>,
    max_supported_tokens: u32,
    max_tokens_to_sample: u32,
) -> Result<Vec<CompletionMessage>> {
    let max = max_supported_tokens - max_tokens_to_sample;

    tracing::event!(tracing::Level::INFO, "max: {:?}", max);

    tracing::event!(
        tracing::Level::INFO,
        "max_supported_tokens: {:?}",
        max_supported_tokens
    );
    tracing::event!(
        tracing::Level::INFO,
        "max_tokens_to_sample: {:?}",
        max_tokens_to_sample
    );
    let messages = trim_messages(messages, max)?;

    Ok(messages.into_iter().map(CompletionMessage::from).collect())
}

/// Trim messages until the total number of tokens inside is less than the maximum.
fn trim_messages(mut messages: Vec<Message>, max: u32) -> Result<Vec<Message>> {
    let total_tokens: usize = messages.iter().map(|m| m.content.len() * 4 / 3).sum();
    tracing::event!(tracing::Level::INFO, "total_tokens: {:?}", total_tokens);

    if total_tokens as u32 <= max {
        return Ok(messages);
    }

    let window = messages
        .windows(messages.len() - 1)
        .next()
        .unwrap_or_default();

    if let Some((index, _)) = window
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

impl From<&Session<SessionOptions>> for RequestOptions {
    fn from(session: &Session<SessionOptions>) -> RequestOptions {
        Self {
            model: session
                .options
                .model
                .clone()
                .unwrap_or_default()
                .as_str()
                .to_string(),
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
            messages: Vec::new(),
        }
    }
}

/// Creates a serialized request body from the session
fn create_body(session: &Session<SessionOptions>) -> Result<String> {
    tracing::event!(tracing::Level::INFO, "Serializing body...");
    let mut request_options: RequestOptions = session.into();

    request_options.messages = complete_messages(
        session.history.clone(),
        session.max_history.unwrap_or(session.max_supported_tokens),
        if session.max_history.is_some() {
            0
        } else {
            request_options.max_tokens.unwrap_or(1000)
        },
    )?;

    match serde_json::to_string(&request_options) {
        Ok(body) => Ok(body),
        Err(e) => {
            tracing::event!(tracing::Level::ERROR, "Error serializing request body.");
            color_eyre::eyre::bail!("error: {e}")
        }
    }
}
