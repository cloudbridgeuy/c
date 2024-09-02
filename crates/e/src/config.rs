use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Default, Deserialize)]
pub struct Preset {
    pub name: String,

    // Api
    pub api: crate::args::Api,
    pub env: Option<String>,
    pub key: Option<String>,
    pub base_url: Option<String>,

    // Model
    pub model: Option<String>,

    // Model Configuration
    pub system: Option<String>,
    pub max_tokens: Option<u32>,
    pub version: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
pub enum Role {
    Assistant,
    Model,
    #[default]
    User,
    Human,
    System,
}

#[derive(Debug, Default, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: Option<String>,
    pub template: String,
    pub default_vars: Option<Value>,
    pub system: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    // Api
    pub api: Option<crate::args::Api>,
    pub base_url: Option<String>,
    pub env: Option<String>,
    pub key: Option<String>,

    // Presets
    pub presets: Option<Vec<Preset>>,

    // Templates
    pub templates: Option<Vec<Template>>,

    // Global
    pub quiet: Option<bool>,

    // Model
    pub model: Option<String>,

    // Model Configuration
    pub system: Option<String>,
    pub max_tokens: Option<u32>,
    pub version: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
}
