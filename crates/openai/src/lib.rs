use reqwest::Client as ReqwestClient;

pub mod error;

#[derive(Debug)]
pub struct Client {
    _client: ReqwestClient,
    api_key: Option<String>,
    endpoint: String,
    model: String,
    max_tokens: Option<u32>,
    temperature: f32,
    top_p: f32,
    n: u32,
    echo: bool,
    presence_penalty: f32,
    frequency_penalty: f32,
}

const OPEN_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const MODEL: &str = "gpt-4";
const TEMPERATURE: f32 = 0.0;
const TOP_P: f32 = 0.8;
const N: u32 = 1;
const PRESENCE_PENALTY: f32 = 0.0;
const FREQUENCY_PENALTY: f32 = 0.0;

impl Client {
    /// Creates a new [`OpenAiClient`].
    pub fn new() -> Client {
        Client {
            _client: ReqwestClient::new(),
            model: MODEL.to_string(),
            api_key: None,
            endpoint: String::from(OPEN_API_URL),
            max_tokens: None,
            temperature: TEMPERATURE,
            top_p: TOP_P,
            n: N,
            echo: false,
            presence_penalty: PRESENCE_PENALTY,
            frequency_penalty: FREQUENCY_PENALTY,
        }
    }

    /// Configures the OpenAI API endpoint.
    pub fn set_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }

    /// Configures the OpenAI API key.
    pub fn set_api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Configures the ID of the OpenAI model to use. You can use the `completions.list` method to
    /// get a list of all available models.
    pub fn set_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// Configures the maximum tokens to generate the completion.
    ///
    /// The token count on your prompt plus `max_tokens` cannot exceed the model's context length.
    /// Most models have a context length of 2048, 4096, or 8192 tokens. GPT-4 also has a model
    /// that supports around 25K tokens.
    pub fn set_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Configures how many completions to generate for each prompt.
    pub fn set_n(mut self, n: u32) -> Self {
        self.n = n;
        self
    }

    /// Configures the sampling temperature to use, between 0 and 2. Higher values like 0.8 will
    /// make the output more determnistic.
    ///
    /// It's recommended to alter this or the `top_p` parameter, but not both.
    pub fn with_temperature(mut self, temperature: f32) -> Result<Self, error::OpenAiError> {
        if (0.0..=2.0).contains(&temperature) {
            self.temperature = temperature;
            Ok(self)
        } else {
            Err(error::OpenAiError::InvalidTemperature { temperature })
        }
    }

    /// Configures the top_p parameter to use, which is an alternative to sampling with temperature,
    /// called nucleus sampling., where the model considers the results of the tokens with `top_p`
    /// probability mass. So, 0.1 means only the tokens comprising the top 10% probability mass are
    /// considered.
    ///
    /// It's recommended to alter this or the `temperature` parameter, but not both.
    pub fn with_top_p(mut self, top_p: f32) -> Result<Self, error::OpenAiError> {
        if (0.0..=1.0).contains(&top_p) {
            self.top_p = top_p;
            Ok(self)
        } else {
            Err(error::OpenAiError::InvalidTopP { top_p })
        }
    }

    /// Echo back the prompt in addition to the completion.
    pub fn with_echo(mut self, echo: bool) -> Self {
        self.echo = echo;
        self
    }

    /// Configures the presence penalty to use, which is a number betweeen -2.0 and 2.0 where
    /// positive values penalize new tokens based on whether they appear in the text so far,
    /// increasing the model's likelihood to talk about new topics.
    pub fn with_presence_penalty(
        mut self,
        presence_penalty: f32,
    ) -> Result<Self, error::OpenAiError> {
        if (-2.0..=2.0).contains(&presence_penalty) {
            self.presence_penalty = presence_penalty;
            Ok(self)
        } else {
            Err(error::OpenAiError::InvalidPresencePenalty { presence_penalty })
        }
    }

    /// Configures the frequency penalty, which is a number between -2.0 and 2.0 where positive
    /// values penalize new tokens based on their existing frequency in the text so far, decreasing
    /// the model's likelihood to repeat the same line verbatim.
    pub fn with_frequency_penalty(
        mut self,
        frequency_penalty: f32,
    ) -> Result<Self, error::OpenAiError> {
        if (-2.0..=2.0).contains(&frequency_penalty) {
            self.frequency_penalty = frequency_penalty;
            Ok(self)
        } else {
            Err(error::OpenAiError::InvalidFrequencyPenalty { frequency_penalty })
        }
    }

    /// Creates a completion for the provided prompt, suffix, and parameters.
    pub fn completion(&self, prompt: &str, suffix: &str) -> String {
        format!("{}{}", prompt, suffix)
    }
}
