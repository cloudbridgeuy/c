use std::ops::RangeInclusive;

use anthropic::client::Client;
use clap::{Parser, ValueEnum};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

use super::anthropic::message::{Message, Role};
use super::anthropic::session::Session;

mod message;
mod session;

#[derive(ValueEnum, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum ClaudeModelOption {
    #[default]
    ClaudeV1,
    ClaudeV1_100k,
    ClaudeInstantV1,
    ClaudeInstantV1_100k,
}

impl ClaudeModelOption {
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
pub struct CompleteRequestBody {
    pub model: String,
    pub prompt: String,
    pub max_tokens_to_sample: Option<u32>,
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
    #[arg(long, default_value = "4096")]
    max_supported_tokens: Option<u32>,
    /// Controls which version of Claude answers your request. Two model families are exposed
    /// Claude and Claude Instant.
    #[clap(short, long, value_enum, default_value = "claude-v1")]
    model: Option<ClaudeModelOption>,
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
    let spinner_arc = spinner::Spinner::new_with_checky_messages(5000);

    // Finish parsing the options. Clap takes care of everything except support for readin the
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

    dbg!(&session);

    tracing::event!(tracing::Level::INFO, "Creating client...");
    // Call the completion endpoint with the current session.
    let client = Client::new(session.meta.key.clone())?;
    let body = serde_json::to_string(&CompleteRequestBody {
        model: session.options.model.to_string(),
        max_tokens_to_sample: session.options.max_tokens_to_sample,
        stop_sequences: session.options.stop_sequences.clone(),
        temperature: session.options.temperature,
        top_k: session.options.top_k,
        top_p: session.options.top_p,
        stream: false,
        prompt: session.complete_prompt()?,
    })?;
    tracing::event!(tracing::Level::INFO, "body: {:?}", body);

    let res = client.post("/v1/complete", body).await?;
    tracing::event!(tracing::Level::INFO, "res: {:?}", res);

    let text = res.text().await?;
    tracing::event!(tracing::Level::INFO, "text: {:?}", text);

    let response: Response = serde_json::from_str(&text)?;
    tracing::event!(tracing::Level::INFO, "response: {:?}", response);

    // Print the response output.
    print_output(&session.meta.format, &response)?;

    // Save the response to the session.
    session.messages.push(Message::new(
        response.completion,
        Role::Assistant,
        session.meta.pin,
    ));

    // Save the session to a file.
    session.save()?;

    // Stop the spinner.
    let mut spinner = spinner_arc.lock().await;
    spinner.stop();

    Ok(())
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
