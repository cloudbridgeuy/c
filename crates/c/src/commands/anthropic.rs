use std::env;
use std::fs;
use std::ops::RangeInclusive;
use std::path;

use clap::{Parser, ValueEnum};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

use anthropic::client::Client;

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
    pub model: ClaudeModelOption,
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
    /// The system prompt is an optional initial prompt that you could indclude with every
    /// message. This is similar to how `system` prompts work with OpenAI Chat GPT models.
    /// It's recommended that you use the `\n\nHuman:` and `\n\nAssistant:` stops tokens to
    /// create the system prompt.
    #[arg(long)]
    system: Option<String>,
    /// Chat session name. Will be used to store previous session interactions.
    #[arg(long)]
    session: Option<String>,
    /// The maximum number of tokens supported by the model.
    #[arg(long, default_value = "4096")]
    max_supported_tokens: Option<u32>,
    /// Controls which version of Claude answers your request. Two model families are exposed
    /// Claude and Claude Instant.
    #[clap(short, long, value_enum)]
    model: Option<ClaudeModelOption>,
    /// A maximum number of tokens to generate before stopping.
    #[arg(long, default_value = "750")]
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
pub async fn run(options: Options) -> Result<()> {
    if options.session.is_some() {
        run_with_session(options).await?;
    } else {
        complete(&options).await?;
    }

    Ok(())
}

/// Runs the `anthropic` command with a session.
pub async fn run_with_session(options: Options) -> Result<()> {
    let mut session = Session::load(options.session.as_ref().unwrap().to_string())?;
    session.merge_options(options)?;

    let prompt = complete(&session.options).await?;

    tracing::event!(tracing::Level::INFO, "Storing completion...");
    session.options.prompt = Some(prompt);

    tracing::event!(
        tracing::Level::INFO,
        "New prompt: {}",
        session.options.prompt.as_ref().unwrap()
    );
    session.save()?;

    Ok(())
}

/// Run the anthropic completion
pub async fn complete(options: &Options) -> Result<String> {
    let mut prompt = options.prompt.clone().unwrap_or_default();

    let mut max =
        options.max_supported_tokens.unwrap_or(4096) - options.max_tokens_to_sample.unwrap_or(1000);

    if options.system.is_some() {
        tracing::event!(tracing::Level::INFO, "system: {:?}", options.system);
        prompt = prepare_prompt(format!("{}\n{}", options.system.as_ref().unwrap(), prompt));
        max -= token_length(options.system.as_ref().unwrap().to_string()) as u32;
    }

    tracing::event!(tracing::Level::INFO, "original prompt: {}", prompt);
    prompt = trim_prompt(prompt, max)?;
    tracing::event!(tracing::Level::INFO, "trimmed prompt: {}", prompt);

    let completion = if options.stream {
        complete_stream(prompt, options).await?
    } else {
        complete_no_stream(prompt, options).await?
    };

    Ok(prepare_prompt(options.prompt.clone().unwrap_or_default()) + &completion)
}

/// Stream the result
pub async fn complete_stream(_prompt: String, _options: &Options) -> Result<String> {
    Ok("something".to_string())
}

/// Don't stream the result.
pub async fn complete_no_stream(prompt: String, options: &Options) -> Result<String> {
    let spinner_arc = spinner::Spinner::new_with_checky_messages(5000);
    let client = Client::new(options.anthropic_api_key.clone())?;
    let body = serde_json::to_string(&CompleteRequestBody {
        model: options.model.unwrap_or(ClaudeModelOption::ClaudeV1),
        prompt,
        max_tokens_to_sample: options.max_tokens_to_sample,
        stop_sequences: options.stop_sequences.clone(),
        stream: options.stream,
        temperature: options.temperature,
        top_k: options.top_k,
        top_p: options.top_p,
    })?;
    tracing::event!(tracing::Level::INFO, "body: {:?}", body);

    let res = client.post("/v1/complete", body).await?;
    tracing::event!(tracing::Level::INFO, "res: {:?}", res);

    let text = res.text().await?;
    tracing::event!(tracing::Level::INFO, "text: {:?}", text);

    let response: Response = serde_json::from_str(&text)?;
    tracing::event!(tracing::Level::INFO, "response: {:?}", response);

    let mut spinner = spinner_arc.lock().await;
    spinner.stop();

    print_output(options, &response)?;

    Ok(response.completion)
}

/// Prints the Response output according to the user options.
fn print_output(options: &Options, response: &Response) -> Result<()> {
    if options.stream {
        return Ok(());
    }

    match options.format {
        Some(crate::Output::Raw) => {
            println!("{}", response.completion);
        }
        Some(crate::Output::Json) => {
            let json = serde_json::to_string_pretty(&response)?;
            println!("{}", json);
        }
        Some(crate::Output::Yaml) => {
            let json = serde_yaml::to_string(&response)?;
            println!("{}", json);
        }
        None => {
            color_eyre::eyre::bail!("No output format specified");
        }
    }

    Ok(())
}

/// Stores the session metadata.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Meta {
    path: String,
}

/// Represents a chat session.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Session {
    id: String,
    options: Options,
    #[serde(skip)]
    meta: Meta,
}

impl Session {
    /// Tries to load a session from the filesystem.
    pub fn load(id: String) -> Result<Self> {
        let home = env::var("C_ROOT").unwrap_or(env::var("HOME")?);
        let path = format!("{home}/.c/sessions/{id}.yaml");

        let session = if fs::metadata(&path).is_ok() {
            let mut session: Session = serde_yaml::from_str(&fs::read_to_string(&path)?)?;
            session.meta = Meta { path };
            session
        } else {
            if !directory_exists(&home) {
                fs::create_dir_all(&home)?;
            }

            let session = Self {
                id,
                meta: Meta { path },
                ..Default::default()
            };
            fs::write(&session.meta.path, serde_yaml::to_string(&session)?)?;

            session
        };

        Ok(session)
    }

    /// Merges an options object into the session options.
    pub fn merge_options(&mut self, options: Options) -> Result<()> {
        self.merge_prompt(options.prompt)?;

        if options.system.is_some() {
            self.options.system = options.system;
        }

        if options.max_tokens_to_sample.is_some() {
            self.options.max_tokens_to_sample = options.max_tokens_to_sample;
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
            self.options.format = options.format;
        }

        self.options.anthropic_api_key = options.anthropic_api_key;
        self.options.stream = options.stream;
        self.options.silent = options.silent;

        Ok(())
    }

    /// Merges the prompt into the session prompt.
    pub fn merge_prompt(&mut self, prompt: Option<String>) -> Result<()> {
        if let Some(prompt) = prompt {
            if !prompt.is_empty() {
                let patch: String = if prompt == "-" {
                    tracing::event!(tracing::Level::INFO, "Reading prompt from stdin...");
                    crate::utils::read_from_stdin()?
                } else {
                    prompt
                };

                let patch = format!("\n\nHuman: {}", patch);

                if self.options.prompt.is_none() {
                    self.options.prompt = Some(patch);
                } else {
                    self.options.prompt.as_mut().unwrap().push_str(&patch);
                }
            }
        };

        Ok(())
    }

    /// Saves the session to the filesystem.
    pub fn save(&self) -> Result<()> {
        tracing::event!(
            tracing::Level::INFO,
            "saving session to {:?}",
            self.meta.path
        );
        fs::write(&self.meta.path, serde_yaml::to_string(&self)?)?;
        Ok(())
    }
}

/// Chacks if a directory exists.
pub fn directory_exists(dir_name: &str) -> bool {
    let p = path::Path::new(dir_name);
    p.exists() && p.is_dir()
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
