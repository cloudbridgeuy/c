use clap::{Parser, ValueEnum};
use clap_stdin::MaybeStdin;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::prelude::*;

#[derive(Debug, clap::Args)]
pub struct Globals {
    /// The user message prompt
    pub prompt: MaybeStdin<String>,

    /// The API provider to use.
    #[clap(short, long, default_value = "anthropic", env = "E_API", value_enum)]
    pub api: String,

    /// The LLM Model to use
    #[clap(short, long, env = "E_MODEL")]
    pub model: Option<String>,

    /// The maximum amount of tokens to return.
    #[clap(long, env = "E_MAX_TOKENS", default_value = "4096")]
    pub max_tokens: i32,

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
}

#[derive(ValueEnum, Debug, Default, Clone, Copy, Serialize, Deserialize)]
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
