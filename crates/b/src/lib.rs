use std::error::Error;

use clap::{Parser, Subcommand, ValueEnum};
use openai::error::OpenAi as OpenAiError;
use serde::Serialize;

pub mod chats;
pub mod commands;
pub mod completions;

pub trait CommandResult {
    fn print_yaml(&self) -> Result<(), OpenAiError>
    where
        Self: Serialize,
    {
        match serde_yaml::to_writer(std::io::stdout(), &self) {
            Ok(_) => Ok(()),
            Err(e) => Err(OpenAiError::SerializationError {
                body: e.to_string(),
            }),
        }
    }

    fn print_json(&self) -> Result<(), OpenAiError>
    where
        Self: Serialize,
    {
        match serde_json::to_writer(std::io::stdout(), &self) {
            Ok(_) => Ok(()),
            Err(e) => Err(OpenAiError::SerializationError {
                body: e.to_string(),
            }),
        }
    }

    /// Returns the raw results of the command.
    fn print_raw(&self) -> Result<(), OpenAiError>;
}

pub trait CommandHandle<R: CommandResult> {
    type CallError: Error;

    /// Runs the command handler
    fn call(&self) -> Result<R, Self::CallError>;
}

#[derive(Debug, Parser)]
#[command(name = "v2")]
#[command(about = "Interact with OpenAI's ChatGPT through the terminal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// OpenAI API Key to use. Will default to the environment variable `OPENAI_API_KEY` if not
    /// set.
    #[arg(long, env = "OPENAI_API_KEY")]
    pub api_key: Option<String>,
    #[clap(short, long, value_enum, default_value_t = Output::Raw)]
    pub output: Output,
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
    /// Completions API commands
    Completions {
        #[command(subcommand)]
        command: CompletionsCommands,
    },
    /// Chat API commands
    Chats {
        #[command(subcommand)]
        command: ChatsCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ChatsCommands {
    /// Create a new chat session
    Create {
        /// ID of the model to use. Use the `modesl list` command to see all your available models
        /// or see the following link: https://platform.openai.com/docs/models/overview
        #[arg(long, default_value = "gpt-3.5-turbo")]
        model: String,
        /// The prompt(s) to generate completions for, encoded as a string, array of strings, array
        /// of tokens, or array of token arrays.
        #[arg(long)]
        prompt: Vec<String>,
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
        /// Whether to stream back partial progress. If set, tokens will be sent as data-only
        /// server-sent-events (SSE) as they become available, with the stream terminated by a
        /// `data: [DONE]` message.
        #[arg(long)]
        stream: Option<bool>,
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
        // /// Modify the likelihood of specified tokens appearing in the completion.
        // #[arg(long)]
        // logit_bias: Option<HashMap<String, f32>>,
        /// A use identifier representing your end-user, which can help OpenAI to monitor and
        /// detect abuse.
        #[arg(long)]
        user: Option<String>,
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
        /// Whether to stream back partial progress. If set, tokens will be sent as data-only
        /// server-sent-events (SSE) as they become available, with the stream terminated by a
        /// `data: [DONE]` message.
        #[arg(long)]
        stream: Option<bool>,
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
