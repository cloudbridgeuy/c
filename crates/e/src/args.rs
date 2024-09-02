use clap::{Parser, ValueEnum};
use clap_stdin::MaybeStdin;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

use crate::prelude::*;

#[derive(Debug, clap::Args)]
pub struct Globals {
    /// Hidden prompt to support prompting from stdin and as an argument
    #[clap(default_value = "-", hide = true)]
    pub stdin: MaybeStdin<String>,

    /// The user message prompt
    #[clap(default_value = "", hide = true)]
    pub prompt: MaybeStdin<String>,

    /// The API provider to use.
    #[clap(short, long, value_enum)]
    pub api: Option<String>,

    /// The LLM Model to use
    #[clap(short, long)]
    pub model: Option<String>,

    /// The maximum amount of tokens to return.
    #[clap(long)]
    pub max_tokens: Option<u32>,

    /// The minimum amount of tokens to return.
    #[clap(long)]
    pub min_tokens: Option<u32>,

    /// The environment variable to use to get the access token for the api.
    #[clap(long)]
    pub api_env: Option<String>,

    /// The api version to use.
    #[clap(long)]
    pub api_version: Option<String>,

    /// The api key to use (will override the value of the environment variable.)
    #[clap(long)]
    pub api_key: Option<String>,

    /// The api base url.
    #[clap(long)]
    pub api_base_url: Option<String>,

    /// Don't run the spinner
    #[clap(long)]
    pub quiet: Option<bool>,

    /// Add a system message to the request.
    #[clap(long)]
    pub system: Option<String>,

    /// Temperature value.
    #[clap(long)]
    pub temperature: Option<f32>,

    /// Top-P value.
    #[clap(long)]
    pub top_p: Option<f32>,

    /// Top-K value.
    #[clap(long)]
    pub top_k: Option<u32>,

    /// Config file
    #[clap(long, default_value = "~/.config/e.toml")]
    pub config_file: String,

    /// Preset configuration
    #[clap(short, long)]
    pub preset: Option<String>,

    /// Additional variables in JSON format
    #[clap(long, default_value="{}", value_parser = parse_json)]
    pub vars: Option<Value>,

    /// Suffix prompt
    #[clap(long)]
    pub suffix: Option<String>,

    /// Language to use for syntax highlight
    #[clap(long, default_value = "markdown")]
    pub language: String,

    /// Prompt template to use
    #[clap(short, long)]
    pub template: Option<String>,

    /// Prints the rendered template instead of calling the LLM.
    #[clap(long, default_value = "false")]
    pub print_template: bool,
}

/// Custom parser function for JSON values
fn parse_json(s: &str) -> std::result::Result<Value, serde_json::Error> {
    serde_json::from_str(s)
}

#[derive(ValueEnum, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Api {
    OpenAi,
    #[default]
    Anthropic,
    Google,
    Mistral,
    MistralFim,
}

// From string to API enum
impl FromStr for Api {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "OpenAi" => Ok(Api::OpenAi),
            "openai" => Ok(Api::OpenAi),
            "Anthropic" => Ok(Api::Anthropic),
            "anthropic" => Ok(Api::Anthropic),
            "google" => Ok(Api::Google),
            "Google" => Ok(Api::Google),
            "gemini" => Ok(Api::Google),
            "Gemini" => Ok(Api::Google),
            "mistral" => Ok(Api::Mistral),
            "Mistral" => Ok(Api::Mistral),
            "mistral-fim" => Ok(Api::MistralFim),
            "mistral_fim" => Ok(Api::MistralFim),
            "Mistral-FIM" => Ok(Api::MistralFim),
            "Mistral-Fim" => Ok(Api::MistralFim),
            "MistralFim" => Ok(Api::MistralFim),
            "Mistral_FIM" => Ok(Api::MistralFim),
            "Mistral_Fim" => Ok(Api::MistralFim),
            "MistralFIM" => Ok(Api::MistralFim),
            _ => Err(Error::InvalidAPI),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "e", version = "0.1.0")]
#[command(about = "Interact with LLMs through the terminal")]
#[command(
    long_about = "This Rust-based CLI enables users to interact with various Large Language Models
(LLMs) directly from the terminal. Through this tool, you can send prompts to different
APIs, such as OpenAI, Anthropic, Google, Mistral, and Mistral FIM, and receive and handle
responses from these models.

The tool offers extensive configuration options, allowing you
to specify parameters like model type, maximum and minimum tokens, temperature, top-p
sampling, system messages, and more. You can set these options via command line arguments
or environment variables. Additionally, it supports preset configurations and prompt
templates, enabling more advanced and customizable usage scenarios.

The CLI can format and
highlight the model's responses using syntax highlighting, making it easier to read the
output in the terminal. It also includes functionality to handle streaming responses
efficiently, ensuring a smooth user experience when interacting with the LLMs."
)]
pub struct Args {
    #[clap(flatten)]
    pub globals: Globals,
}
