use clap::Parser;
use color_eyre::eyre::Result;
use reqwest_eventsource::EventSource;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

use crate::session::{Message, Role, Session, Vendor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub response: String,
    pub done: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RequestOptions {
    pub model: String,
    pub prompt: String,
}

impl From<CommandOptions> for RequestOptions {
    fn from(options: CommandOptions) -> Self {
        Self {
            model: options.model.unwrap_or_default(),
            prompt: options.prompt.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SessionOptions {
    model: String,
    url: String,
}

impl From<RequestOptions> for SessionOptions {
    fn from(options: RequestOptions) -> Self {
        Self {
            model: options.model,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Response {
    pub model: String,
    pub response: Option<String>,
    pub done: bool,
}

#[derive(Default, Clone, Parser, Debug, Serialize, Deserialize)]
pub struct CommandOptions {
    /// The prompt you want Claude to complete.
    prompt: Option<String>,
    /// Chat session name. Will be used to store previous session interactions.
    #[arg(long)]
    session: Option<String>,
    /// The maximum number of tokens supported by the model.
    #[arg(long)]
    max_supported_tokens: Option<u32>,
    /// The endpoint to use
    #[clap(short, long, default_value = "http://localhost:11434/api/generate")]
    url: Option<String>,
    /// Controls which version of Claude answers your request. Two model families are exposed
    /// Claude and Claude Instant.
    #[clap(short, long, value_enum)]
    model: Option<String>,
    /// Silent mode
    #[clap(short, long, action, default_value_t = false)]
    silent: bool,
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

/// Runs the `anthropic` command.
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
    let request_options: RequestOptions = options.clone().into();
    let session_options: SessionOptions = request_options.clone().into();

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
            let session: Session<SessionOptions> =
                Session::new(session, Vendor::Anthropic, session_options, 16000);
            session
        }
    } else {
        tracing::event!(tracing::Level::INFO, "Creating anonymous session...");
        let session: Session<SessionOptions> =
            Session::anonymous(Vendor::Anthropic, session_options, 16000);
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

    let mut acc: String = Default::default();
    let chunks = complete_stream(&session).await?;

    tokio::pin!(chunks);

    while let Some(chunk) = chunks.next().await {
        if chunk.is_err() {
            color_eyre::eyre::bail!("Error streaming response: {:?}", chunk);
        }

        let chunk = chunk.unwrap();
        tracing::event!(tracing::Level::DEBUG, "Received chunk... {:?}", chunk);

        // TODO: I can't make the stream print to `stdout` without some artifacts to appear
        // on the console due to it.
        // Stop the spinner when the stream starts.
        spinner.stop();

        // Print the response output.
        print!("{}", chunk.response);

        acc.push_str(&chunk.response);
    }
    // Add a new line at the end to make sure the prompt is on a new line.
    println!();

    // Save the response to the session.
    session.history.push(Message::new(
        acc.trim().to_string(),
        Role::Assistant,
        session.meta.pin,
    ));

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

/// Returns a valid completion prompt from the list of messages.
pub fn complete_prompt_history(
    mut messages: Vec<Message>,
    max_history: u32,
    max_tokens_to_sample: u32,
) -> Result<String> {
    let max = max_history - max_tokens_to_sample;

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

        let window = messages
            .windows(messages.len() - 1)
            .next()
            .unwrap_or_default();

        if let Some((index, _)) = window.iter().enumerate().find(|(_, m)| !m.pin) {
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

/// Merges an options object into the session options.
pub fn merge_options(
    mut session: Session<SessionOptions>,
    options: CommandOptions,
) -> Result<Session<SessionOptions>> {
    if options.model.is_some() {
        session.options.model = options.model.unwrap();
        session.max_supported_tokens = 8000;
    }

    if options.max_supported_tokens.is_some() {
        session.max_supported_tokens = options.max_supported_tokens.unwrap();
    }

    if options.max_history.is_some() {
        session.max_history = options.max_history;
    }

    if options.format.is_some() {
        session.meta.format = options.format.unwrap();
    }

    if options.history_size.is_some() {
        session.meta.history_size = options.history_size;
    }

    session.meta.save = !options.nosave;
    session.meta.silent = options.silent;
    session.meta.pin = options.pin;

    Ok(session)
}

/// Completes the command by streaming the response.
async fn complete_stream(
    session: &Session<SessionOptions>,
) -> Result<impl Stream<Item = Result<Chunk>>> {
    let body = create_body(session)?;
    tracing::event!(tracing::Level::INFO, "body: {:?}", body);

    tracing::event!(tracing::Level::INFO, "Creating client...");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    tracing::event!(tracing::Level::INFO, "Creating API client headers...");
    let mut headers = reqwest::header::HeaderMap::new();
    let content_type = reqwest::header::HeaderValue::from_static("application/json");
    headers.insert(reqwest::header::CONTENT_TYPE, content_type);

    tracing::event!(tracing::Level::INFO, "POST {}", &session.options.url);
    let builder = client
        .post(session.options.url.as_str())
        .headers(headers)
        .body(body);

    tracing::event!(tracing::Level::INFO, "Creating the Reqwest EventSource...");
    let mut event_source = EventSource::new(builder)?;

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
                        tracing::event!(tracing::Level::DEBUG, "Open SSE stream...");
                    }
                    reqwest_eventsource::Event::Message(message) => {
                        tracing::event!(tracing::Level::DEBUG, "message: {:?}", message);

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

                        if chunk.done == Some(true) {
                            tracing::event!(
                                tracing::Level::INFO,
                                "Stopping stream due to stop_reason",
                            );
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

impl From<&Session<SessionOptions>> for RequestOptions {
    fn from(session: &Session<SessionOptions>) -> RequestOptions {
        Self {
            model: session.options.model.clone(),
            prompt: "".to_string(),
        }
    }
}

/// Creates a serialized request body from the session
fn create_body(session: &Session<SessionOptions>) -> Result<String> {
    tracing::event!(tracing::Level::INFO, "Serializing body...");
    let mut request_options: RequestOptions = session.into();

    request_options.prompt = complete_prompt_history(
        session.history.clone(),
        session.max_history.unwrap_or(session.max_supported_tokens),
        if session.max_history.is_some() {
            0
        } else {
            4000
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

/// Join messages
fn join_messages(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|m| format!("\n\n{:?}: {}", m.role, m.content))
        .collect::<Vec<String>>()
        .join("")
}

/// Token language of a prompt.
/// TODO: Make this better!
fn token_length(prompt: &str) -> usize {
    let words = prompt.split_whitespace().rev().collect::<Vec<&str>>();

    // Estimate the total tokens by multiplying words by 4/3
    words.len() * 4 / 3
}
