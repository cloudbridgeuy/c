use anthropic::complete::Model as AnthropicModel;
use clap::{Parser, ValueEnum};
use color_eyre::eyre::Result;

#[derive(ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum ClaudeModelOption {
    ClaudeV1,
    ClaudeV1_100k,
    ClaudeInstantV1,
    ClaudeInstantV1_100k,
}

impl From<ClaudeModelOption> for AnthropicModel {
    fn from(other: ClaudeModelOption) -> AnthropicModel {
        match other {
            ClaudeModelOption::ClaudeV1 => AnthropicModel::ClaudeV1,
            ClaudeModelOption::ClaudeV1_100k => AnthropicModel::ClaudeV1_100k,
            ClaudeModelOption::ClaudeInstantV1 => AnthropicModel::ClaudeInstantV1,
            ClaudeModelOption::ClaudeInstantV1_100k => AnthropicModel::ClaudeInstantV1_100k,
        }
    }
}

impl From<AnthropicModel> for ClaudeModelOption {
    fn from(other: AnthropicModel) -> ClaudeModelOption {
        match other {
            AnthropicModel::ClaudeV1 => ClaudeModelOption::ClaudeV1,
            AnthropicModel::ClaudeV1_100k => ClaudeModelOption::ClaudeV1_100k,
            AnthropicModel::ClaudeInstantV1 => ClaudeModelOption::ClaudeInstantV1,
            AnthropicModel::ClaudeInstantV1_100k => ClaudeModelOption::ClaudeInstantV1_100k,
        }
    }
}

#[derive(Parser, Debug)]
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
    #[clap(long)]
    temperature: Option<f32>,
    /// Only sample fromt the top `K` options of each subsequent token. Used to remove "long
    /// tail" low probability responses. Defaults to -1, which disables it.
    #[clap(long)]
    top_k: Option<f32>,
    /// Does nucleus sampleing, in which we compute the cumulative distribution over all the
    /// options for each subsequent token in decreasing probability order and cut it off once
    /// it reaches a particular probability specified by the top_p. Defaults to -1, which
    /// disables it. Not that you should either alter *temperature* or *top_p* but not both.
    #[clap(long)]
    top_p: Option<f32>,
    /// Anthropic API Key to use. Will default to the environment variable `OPENAI_API_KEY` if not set.
    #[arg(long, env = "ANTHROPIC_API_KEY")]
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

pub async fn run(options: Options) -> Result<()> {
    let mut api = create_api(&options)?;
    api = update_api(api, &options)?;

    let spinner_arc = spinner::Spinner::new_with_checky_messages(5000);
    let response = api.create().await?;
    let mut spinner = spinner_arc.lock().await;
    spinner.stop();

    print_output(&options, &response)?;

    Ok(())
}

/// Updates the api according to the user options.
fn update_api(
    mut api: anthropic::complete::Api,
    options: &Options,
) -> Result<anthropic::complete::Api> {
    if let Some(options_model) = options.model.as_ref() {
        let model: anthropic::complete::Model = anthropic::complete::Model::from(*options_model);

        if api.model as u32 != model as u32 {
            tracing::event!(tracing::Level::INFO, "Setting model {:?}", model);
            api.model = model;
        }
    }

    if let Some(prompt) = options.prompt.as_ref() {
        if !prompt.is_empty() {
            api.prompt.push_str("\n\nHuman: ");

            if prompt == "-" {
                tracing::event!(tracing::Level::INFO, "Reading prompt from stdin...");
                let stdin = crate::utils::read_from_stdin()?;
                api.prompt.push_str(&stdin);
            } else {
                tracing::event!(tracing::Level::INFO, "Reading user prompt...");
                api.prompt.push_str(prompt);
            }
        } else {
            tracing::event!(tracing::Level::INFO, "Prompt is empty.");
        }
    }

    if options.system.is_some() {
        api.system = options.system.clone();
    }

    if options.max_tokens_to_sample.is_some() {
        api.max_tokens_to_sample = options.max_tokens_to_sample;
    }

    if options.max_supported_tokens.is_some() {
        api.max_supported_tokens = options.max_supported_tokens;
    }

    if options.temperature.is_some() {
        api.set_temperature(options.temperature.unwrap())
            .map_err(|e| {
                color_eyre::eyre::eyre!(
                    "Failed to set temperature to {} with error: {}",
                    options.temperature.unwrap(),
                    e
                )
            })?;
    }

    if options.top_k.is_some() {
        api.set_top_k(options.top_k.unwrap()).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to set top_k to {} with error: {}",
                options.top_k.unwrap(),
                e
            )
        })?;
    }

    if options.top_p.is_some() {
        api.set_top_p(options.top_p.unwrap()).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to set top_p to {} with error: {}",
                options.top_p.unwrap(),
                e
            )
        })?;
    }

    if options.stream {
        api.stream = Some(options.stream);
    }

    if let Some(stop_sequences) = options.stop_sequences.as_ref() {
        api.stop_sequences = Some(stop_sequences.clone());
    }

    Ok(api)
}

/// Creates an anthropic API Client from the session and API key.
fn create_api(options: &Options) -> Result<anthropic::complete::Api> {
    if let Some(session) = &options.session {
        tracing::event!(
            tracing::Level::INFO,
            "Creating a new API client with session... {}",
            session
        );
        Ok(anthropic::complete::Api::new_with_session(
            options.anthropic_api_key.to_string(),
            session.to_string(),
        )
        .or_else(|_| {
            color_eyre::eyre::bail!(
                "Failed to create a new API client. Please check your API key and try again."
            )
        })?)
    } else {
        tracing::event!(tracing::Level::INFO, "Creating a new API client...",);
        Ok(
            anthropic::complete::Api::new(options.anthropic_api_key.to_string()).or_else(|_| {
                color_eyre::eyre::bail!(
                    "Failed to create a new API client. Please check your API key and try again."
                )
            })?,
        )
    }
}

/// Prints the Response output according to the user options.
fn print_output(options: &Options, response: &anthropic::complete::Response) -> Result<()> {
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
