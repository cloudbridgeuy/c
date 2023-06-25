use std::error::Error;

use async_trait::async_trait;
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

use ::anthropic::complete::Model;
use openai::error::OpenAi as OpenAiError;

pub mod anthropic;
pub mod chats;
pub mod commands;
pub mod completions;
pub mod edits;
pub mod tokenizer;
pub mod utils;

#[derive(Debug)]
pub enum CommandError {
    /// Struct can't be serialized/deserialized
    SerializationError { body: String },

    /// OpenAi API Error
    OpenAiError { body: String },

    /// Tokenizer Error
    Tokenizer { body: String },

    /// Io Error
    IoError { source: std::io::Error },

    /// Anthropic Error
    AnthropicError { body: String },
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Error for CommandError {}

impl From<OpenAiError> for CommandError {
    fn from(e: OpenAiError) -> Self {
        Self::OpenAiError {
            body: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerializationError {
            body: e.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for CommandError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::SerializationError {
            body: e.to_string(),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError { source: e }
    }
}

pub trait CommandResult {
    type ResultError: Error
        + From<serde_json::Error>
        + From<serde_yaml::Error>
        + From<std::io::Error>;

    fn print_yaml<W: std::io::Write>(&self, writer: W) -> Result<(), Self::ResultError>
    where
        Self: Serialize,
    {
        serde_yaml::to_writer(writer, &self).map_err(Self::ResultError::from)
    }

    fn print_json<W: std::io::Write>(&self, writer: W) -> Result<(), Self::ResultError>
    where
        Self: Serialize,
    {
        serde_json::to_writer(writer, &self).map_err(Self::ResultError::from)
    }

    /// Returns the raw results of the command.
    fn print_raw<W: std::io::Write>(&self, writer: W) -> Result<(), Self::ResultError>;
}

#[async_trait]
pub trait CommandHandle<R: CommandResult> {
    type CallError: Error;

    /// Runs the command handler
    async fn call(&self) -> Result<R, Self::CallError>;
}

#[derive(Debug, Parser)]
#[command(name = "v2")]
#[command(about = "Interact with OpenAI's ChatGPT through the terminal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// OpenAI API Key to use. Will default to the environment variable `OPENAI_API_KEY` if not set.
    #[arg(long, env = "OPENAI_API_KEY")]
    pub openai_api_key: Option<String>,
    /// Anthropic API Key to use. Will default to the environment variable `OPENAI_API_KEY` if not set.
    #[arg(long, env = "ANTHROPIC_API_KEY")]
    pub anthropic_api_key: Option<String>,
    /// Command output format
    #[clap(short, long, value_enum, default_value_t = Output::Raw)]
    pub output: Output,
    /// Silent mode
    #[clap(short, long, action, default_value_t = false)]
    pub silent: bool,
    /// Wether to incrementally stream the response using SSE.
    #[clap(long)]
    pub stream: bool,
}

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum Output {
    /// Plain text
    Raw,
    /// JSON
    Json,
    /// YAML
    Yaml,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Tokenize
    Tokenizer {
        #[command(subcommand)]
        command: TokenizerCommands,
    },
    /// Completions API commands
    Completions {
        #[command(subcommand)]
        command: CompletionsCommands,
    },
    /// Chats API commands
    Chats {
        #[command(subcommand)]
        command: ChatsCommands,
    },
    /// Edits API commands
    Edits {
        #[command(subcommand)]
        command: EditsCommands,
    },
    /// Anthropic API commands
    Anthropic {
        #[command(subcommand)]
        command: AnthropicCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum TokenizerCommands {
    /// Encodes a prompt
    Encode { prompt: String },
    /// Decodes a prompt
    Decode { encoded: Vec<u32> },
}

#[derive(ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum ClaudeModelOption {
    ClaudeV1,
    ClaudeV1_100k,
    ClaudeInstantV1,
    ClaudeInstantV1_100k,
}

impl From<ClaudeModelOption> for Model {
    fn from(other: ClaudeModelOption) -> Model {
        match other {
            ClaudeModelOption::ClaudeV1 => Model::ClaudeV1,
            ClaudeModelOption::ClaudeV1_100k => Model::ClaudeV1_100k,
            ClaudeModelOption::ClaudeInstantV1 => Model::ClaudeInstantV1,
            ClaudeModelOption::ClaudeInstantV1_100k => Model::ClaudeInstantV1_100k,
        }
    }
}

impl From<Model> for ClaudeModelOption {
    fn from(other: Model) -> ClaudeModelOption {
        match other {
            Model::ClaudeV1 => ClaudeModelOption::ClaudeV1,
            Model::ClaudeV1_100k => ClaudeModelOption::ClaudeV1_100k,
            Model::ClaudeInstantV1 => ClaudeModelOption::ClaudeInstantV1,
            Model::ClaudeInstantV1_100k => ClaudeModelOption::ClaudeInstantV1_100k,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum AnthropicCommands {
    /// Create a new complete session
    Create {
        /// The prompt you want Claude to complete.
        prompt: String,
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
    },
}

#[derive(Debug, Subcommand)]
pub enum ChatsCommands {
    /// Create a new chat session
    Create {
        /// ID of the model to use. Use the `modesl list` command to see all your available models
        /// or see the following link: https://platform.openai.com/docs/models/overview
        #[clap(long)]
        model: Option<String>,
        /// Chat session name. Will be used to store previous session interactions.
        #[arg(long)]
        session: Option<String>,
        /// The system message helps set the behavior of the assistant.
        #[arg(long)]
        system: Option<String>,
        /// The content of the message to be sent to the chatbot. You can also populate this value
        /// from stdin. If you pass a value here and pipe data from stdin, both will be sent to the
        /// API, stdin taking precedence.
        prompt: Option<String>,
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
        #[arg(long)]
        temperature: Option<f32>,
        /// An alternative sampling with temperature, called nucleus sampling, where the model
        /// considers the results of the tokens with `top_p` probability mass. So, 0.1 means only
        /// the tokens comprising the top 10% probability mass are considered. It's generally
        /// recommended to alter this or `temperature` but not both.
        #[arg(long)]
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
        #[arg(long, default_value = "750")]
        min_available_tokens: Option<u32>,
        /// The maximum number of tokens supporte by the model.
        #[arg(long, default_value = "4096")]
        max_supported_tokens: Option<u32>,
        /// A list of functions the model may generate JSON inputs for, provided as JSON.
        #[arg(long)]
        functions: Option<String>,
        /// Controls how the model responds to function calls. "none" means the model does not call
        /// a function, and responds to the end-user. "auto" means the model can pick between an
        /// end-user or calling a function. Specifying a particular function via `{"name":
        /// "my_function" }` forces the model to call that function. "none" is the default when no
        /// functions are present. "auto" is the default if functions are present.
        #[arg(long)]
        function_call: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum EditsCommands {
    /// Create a new chat session
    Create {
        /// ID of the model to use. Use the `modesl list` command to see all your available models
        /// or see the following link: https://platform.openai.com/docs/models/overview
        #[arg(long, default_value = "code-davinci-edit-001")]
        model: String,
        /// The input text to use as a starting point.
        #[arg(long)]
        input: Option<String>,
        /// The instruction that tells the model how to edit the prompt.
        #[arg(long)]
        instruction: String,
        /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the
        /// output more random, while lower valies like 0.2 will make it more focused and
        /// deterministic. It's generally recommended to alter this or `top_p` but not both.
        #[arg(long)]
        temperature: Option<f32>,
        /// An alternative sampling with temperature, called nucleus sampling, where the model
        /// considers the results of the tokens with `top_p` probability mass. So, 0.1 means only
        /// the tokens comprising the top 10% probability mass are considered. It's generally
        /// recommended to alter this or `temperature` but not both.
        #[arg(long)]
        top_p: Option<f32>,
        /// How many completions to generate for each prompt.
        #[arg(long)]
        n: Option<u32>,
    },
}

#[derive(Debug, Subcommand, Clone)]
pub enum CompletionsCommands {
    /// Create a new chat session
    Create {
        /// ID of the model to use. Use the `modesl list` command to see all your available models
        /// or see the following link: https://platform.openai.com/docs/models/overview
        #[arg(long, default_value = "text-davinci-003")]
        model: String,
        /// The prompt(s) to generate completions for, encoded as a string, array of strings, array
        /// of tokens, or array of token arrays.
        #[arg(long)]
        prompt: Vec<String>,
        /// The suffix that comes after a completion of inserted text.
        #[arg(long)]
        suffix: Option<String>,
        /// The maximum number of tokens to generate in the completion.
        #[arg(long)]
        max_tokens: Option<u32>,
        /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the
        /// output more random, while lower valies like 0.2 will make it more focused and
        /// deterministic. It's generally recommended to alter this or `top_p` but not both.
        #[arg(long)]
        temperature: Option<f32>,
        /// An alternative sampling with temperature, called nucleus sampling, where the model
        /// considers the results of the tokens with `top_p` probability mass. So, 0.1 means only
        /// the tokens comprising the top 10% probability mass are considered. It's generally
        /// recommended to alter this or `temperature` but not both.
        #[arg(long)]
        top_p: Option<f32>,
        /// How many completions to generate for each prompt.
        #[arg(long)]
        n: Option<u32>,
        /// Include the probabilities on the `logprobs` most likely tokens, as well the chosen
        /// tokens. For example, if `logprobs` is 5, the API will return a list of the 5 most
        /// likely tokens. The API will always return the `logprob` of the sampled token, so there
        /// may be up to `logprobs+1` elements in the response. The maximum value for `logprobs` is
        /// 5.
        #[arg(long)]
        logprobs: Option<f32>,
        /// Echo back the prompt in addition to the completion.
        #[arg(long)]
        echo: Option<bool>,
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
        /// Generates `best_of` completions server-side and returns the `best` (the one with the
        /// highest log probability per token). Results cannot be streamed.
        #[arg(long)]
        best_of: Option<u32>,
        // /// Modify the likelihood of specified tokens appearing in the completion.
        // #[arg(long)]
        // logit_bias: Option<HashMap<String, f32>>,
        /// A use identifier representing your end-user, which can help OpenAI to monitor and
        /// detect abuse.
        #[arg(long)]
        user: Option<String>,
    },
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid key-value pair: {}", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
