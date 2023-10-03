use clap::{Parser, Subcommand, ValueEnum};

use serde::{Deserialize, Serialize};

pub mod commands;
pub mod session;
pub mod utils;

#[derive(Debug, Parser)]
#[command(name = "v2")]
#[command(about = "Interact with OpenAI's ChatGPT through the terminal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Anthropic Chat AI API
    #[clap(alias = "a")]
    Anthropic(commands::anthropic::CommandOptions),
    /// OpenAi Chat AI API
    #[clap(name = "openai", alias = "o")]
    OpenAi(commands::openai::CommandOptions),
    /// Google Vertex AI Chat Code API
    #[clap(name = "vertex", alias = "v")]
    Vertex(commands::vertex::CommandOptions),
    /// NLPCloud AI Chat Bot API
    #[clap(name = "nlpcloud", alias = "n")]
    NLPCloud(commands::nlpcloud::CommandOptions),
    /// Ollama AI Chat Bot API
    #[clap(name = "ollama", alias = "l")]
    Ollama(commands::ollama::CommandOptions),
}

#[derive(Default, ValueEnum, Debug, Clone, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
pub enum Output {
    #[default]
    /// Plain text
    Raw,
    /// JSON
    Json,
    /// YAML
    Yaml,
}
