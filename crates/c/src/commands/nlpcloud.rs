use std::time::Duration;

use clap::{Parser, ValueEnum};
use color_eyre::eyre::Result;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};

use crate::session::{Message, Role, Session, Vendor};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Request {
    input: String,
    context: Option<String>,
    history: Vec<NLPMessage>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NLPMessage {
    input: String,
    response: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Response {
    response: String,
    history: Vec<NLPMessage>,
}

#[derive(ValueEnum, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
enum Model {
    #[default]
    #[serde(rename = "finetuned-gpt-neox-20b")]
    FinetunedGptNeox20b,
    #[serde(rename = "fast-gpt-j")]
    FastGptJ,
    #[serde(rename = "dolphin")]
    Dolphin,
    #[serde(rename = "chatdolphin")]
    ChatDolphin,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::FastGptJ => "fast-gpt-j",
            Model::FinetunedGptNeox20b => "finetuned-gpt-neox-20b",
            Model::Dolphin => "dolphin",
            Model::ChatDolphin => "chatdolphin",
        }
    }

    pub fn as_u32(&self) -> u32 {
        2048
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SessionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<Model>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
}

#[derive(Default, Clone, Parser, Debug, Serialize, Deserialize)]
pub struct CommandOptions {
    /// The prompt you want Claude to complete.
    prompt: Option<String>,
    /// Chat session name. Will be used to store previous session interactions.
    #[arg(long)]
    session: Option<String>,
    /// A context for the conversation that gives potential details about the mood, facts, etc.
    #[clap(long)]
    context: Option<String>,
    /// NLP model to use.
    #[clap(short, long, value_enum)]
    model: Option<Model>,
    /// NLP Cloud Key to use. Will default to the environment variable `NLPCLOUD_API_KEY` if not set.
    #[arg(long, env = "NLPCLOUD_API_KEY")]
    #[serde(skip)]
    nlpcloud_api_key: String,
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
    /// Add the history messages in reverse order
    #[clap(long)]
    reverse: bool,
}

/// Runs the nlpcloud chat command
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

    let session_options = SessionOptions {
        ..Default::default()
    };

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
                Vendor::NLPCloud,
                session_options,
                options.model.unwrap_or(Model::default()).as_u32(),
            );
            session
        }
    } else {
        tracing::event!(tracing::Level::INFO, "Creating anonymous session...");
        let session: Session<SessionOptions> = Session::anonymous(
            Vendor::NLPCloud,
            session_options,
            options.model.unwrap_or(Model::default()).as_u32(),
        );
        session
    };

    tracing::event!(tracing::Level::INFO, "Mergin command options...");
    // Create a new named or anonymous session.
    let mut session = merge_options(session, options)?;

    let prompt = prompt.unwrap_or_default();

    let response = complete(&session, prompt.clone()).await?;

    // Stop the spinner.
    spinner.stop();

    // Print the response output.
    print_output(&session.meta.format, &response)?;

    // Save the input and the response to the session.
    session
        .history
        .push(Message::new(prompt, Role::User, session.meta.pin));
    session.history.push(Message::new(
        response.response.trim().to_string(),
        Role::Assistant,
        session.meta.pin,
    ));

    // Save the session to a file.
    if session.meta.save {
        session.save()?;
    }

    Ok(())
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

    if options.context.is_some() {
        session.options.context = options.context;
    }

    if options.max_history.is_some() {
        session.max_history = options.max_history;
    }

    if options.format.is_some() {
        session.meta.format = options.format.unwrap();
    }

    session.meta.save = !options.nosave;
    session.meta.key = options.nlpcloud_api_key;
    session.meta.silent = options.silent;
    session.meta.pin = options.pin;
    session.meta.reverse = options.reverse;

    Ok(session)
}

/// Prints the Response output according to the user options.
pub fn print_output(format: &crate::Output, response: &Response) -> Result<()> {
    match format {
        crate::Output::Raw => {
            println!("{}", response.response);
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

/// Completes the command without streaming the response.
async fn complete(session: &Session<SessionOptions>, input: String) -> Result<Response> {
    let body = create_body(session, input)?;
    tracing::event!(tracing::Level::INFO, "body: {:?}", body);

    let reqwest = ReqwestClient::builder()
        .timeout(Duration::from_secs(300))
        .build()?;
    tracing::event!(tracing::Level::INFO, "Created HTTP client...");

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(format!("Token {}", session.meta.key).as_str())?,
    );
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);
    tracing::event!(tracing::Level::INFO, "Created HTTP headers...");

    let url = format!(
        "https://api.nlpcloud.io/v1/gpu/{}/chatbot",
        session.options.model.unwrap_or_default().as_str()
    );
    tracing::event!(tracing::Level::INFO, "POST {:?}", url);

    let res = reqwest
        .post(url)
        .headers(headers)
        .body(body.clone())
        .send()
        .await?;
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

/// Creates a serialized request body from the session
fn create_body(session: &Session<SessionOptions>, input: String) -> Result<String> {
    tracing::event!(tracing::Level::INFO, "Serializing body...");

    match serde_json::to_string(&Request {
        input,
        context: session.options.context.clone(),
        history: complete_history(session.history.clone(), session.meta.reverse)?,
    }) {
        Ok(body) => Ok(body),
        Err(e) => {
            tracing::event!(tracing::Level::ERROR, "Error serializing request body.");
            color_eyre::eyre::bail!("error: {e}")
        }
    }
}

/// Returns a valid list of messages for the completion to work.
pub fn complete_history(mut messages: Vec<Message>, reverse: bool) -> Result<Vec<NLPMessage>> {
    if messages.len() < 2 {
        return Ok(vec![]);
    }

    if reverse {
        messages.reverse();
    }

    let mut messages = trim_messages(messages)?;

    if reverse {
        messages.reverse();
    }

    Ok(messages
        .windows(2)
        .map(|messages| {
            tracing::event!(tracing::Level::INFO, "messages: {:?}", messages);
            let human = messages[0].clone();
            let ai = messages[1].clone();

            NLPMessage {
                input: human.content,
                response: ai.content,
            }
        })
        .collect())
}

/// Trim messages until the total number of tokens inside is less than the maximum.
fn trim_messages(mut messages: Vec<Message>) -> Result<Vec<Message>> {
    let max = 2048;

    tracing::event!(tracing::Level::INFO, "max: {:?}", max);

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
        trim_messages(messages)
    } else {
        Err(color_eyre::eyre::format_err!(
            "Could not trim messages to fit the maximum number of tokens."
        ))
    }
}
