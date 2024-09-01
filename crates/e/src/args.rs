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
    #[clap(short, long, env = "E_API", value_enum)]
    pub api: Option<String>,

    /// The LLM Model to use
    #[clap(short, long, env = "E_MODEL")]
    pub model: Option<String>,

    /// The maximum amount of tokens to return.
    #[clap(long, env = "E_MAX_TOKENS")]
    pub max_tokens: Option<u32>,

    /// The environment variable to use to get the access token for the api.
    #[clap(long, env = "E_API_ENV")]
    pub api_env: Option<String>,

    /// The api version to use.
    #[clap(long, env = "E_API_VERSION")]
    pub api_version: Option<String>,

    /// The api key to use (will override the value of the environment variable.)
    #[clap(long, env = "E_API_KEY")]
    pub api_key: Option<String>,

    /// The api base url.
    #[clap(long, env = "E_API_BASE_URL")]
    pub api_base_url: Option<String>,

    /// Don't run the spinner
    #[clap(long, env = "E_QUIET")]
    pub quiet: Option<bool>,

    /// Add a system message to the request.
    #[clap(long, env = "E_SYSTEM")]
    pub system: Option<String>,

    /// Temperature value.
    #[clap(long, env = "E_TEMPERATURE")]
    pub temperature: Option<f32>,

    /// Top-P value.
    #[clap(long, env = "E_TOP_P")]
    pub top_p: Option<f32>,

    /// Top-K value.
    #[clap(long, env = "E_TOP_K")]
    pub top_k: Option<u32>,

    /// Config file
    #[clap(long, env = "E_CONFIG_FILE", default_value = "~/.config/e.toml")]
    pub config_file: String,

    /// Preset configuration
    #[clap(long, env = "E_PRESET")]
    pub preset: Option<String>,

    /// Templates to use.
    #[clap(short, long, env = "E_TEMPLATES")]
    pub templates: Option<Vec<String>>,

    /// Additional variables in JSON format
    #[clap(long, env = "E_VARS", value_parser = parse_json)]
    pub vars: Option<Value>,
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
            _ => Err(Error::InvalidAPI),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "e")]
#[command(about = "Interact with LLMs through the terminal")]
pub struct Args {
    #[clap(flatten)]
    pub globals: Globals,
}
