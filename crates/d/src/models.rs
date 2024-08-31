use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum Model {
    #[serde(rename = "gpt-4-1106-preview")]
    GPT41106Preview,
    #[serde(rename = "gpt-4o-mini")]
    GPT4OMini,
    #[default]
    #[serde(rename = "gpt-4o")]
    GPT4O,
    #[serde(rename = "gpt-4")]
    GPT4,
    #[serde(rename = "gpt-4-32k")]
    GPT432K,
    #[serde(rename = "gpt-3.5-turbo")]
    GPT35Turbo,
    #[serde(rename = "gpt-3.5-turbo-16k")]
    GPT35Turbo16K,
    #[serde(rename = "gpt-3.5-turbo-1106")]
    GPT35Turbo1106,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::GPT41106Preview => "gpt-4-1106-preview",
            Model::GPT4OMini => "gpt-4o-mini",
            Model::GPT4O => "gpt-4o",
            Model::GPT4 => "gpt-4",
            Model::GPT432K => "gpt-4-32k",
            Model::GPT35Turbo => "gpt-3.5-turbo",
            Model::GPT35Turbo16K => "gpt-3.5-turbo-16k",
            Model::GPT35Turbo1106 => "gpt-3.5-turbo-1106",
        }
    }
}
