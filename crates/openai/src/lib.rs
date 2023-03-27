use reqwest::Client as ReqwestClient;

pub mod error;

#[derive(Debug)]
pub struct Completion;
#[derive(Debug)]
pub struct Chat;

#[derive(Debug)]
pub struct OpenAi {
    _client: ReqwestClient,
    api_key: String,
    endpoint: String,
}

#[derive(Debug)]
enum OpenAiApi {
    CompletionApi(CompletionConfig),
    ChatApi(ChatConfig),
}

#[derive(Debug)]
struct CompletionConfig {
    model: String,
    echo: bool,
    prompt: Option<String>,
    suffix: Option<String>,
}

#[derive(Debug)]
struct ChatConfig {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug)]
pub struct Client<'a, State = Completion> {
    openai: &'a OpenAi,
    temperature: f32,
    top_p: f32,
    n: u32,
    presence_penalty: f32,
    frequency_penalty: f32,
    max_tokens: Option<u32>,
    user: Option<String>,
    api: OpenAiApi,
    state: std::marker::PhantomData<State>,
}

const OPEN_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const COMPLETION_MODEL: &str = "text-davinci-003";
const CHAT_MODEL: &str = "chat-gpt3.5-turbo";
const TEMPERATURE: f32 = 0.0;
const TOP_P: f32 = 0.8;
const N: u32 = 1;
const PRESENCE_PENALTY: f32 = 0.0;
const FREQUENCY_PENALTY: f32 = 0.0;

impl<'a> OpenAi {
    pub fn new(api_key: String) -> Self {
        OpenAi {
            _client: ReqwestClient::new(),
            api_key: api_key.to_string(),
            endpoint: OPEN_API_URL.to_string(),
        }
    }

    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = endpoint;
        self
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = api_key;
        self
    }

    pub fn completions(&'a self) -> Client<Completion> {
        Client {
            openai: self,
            max_tokens: None,
            temperature: TEMPERATURE,
            top_p: TOP_P,
            n: N,
            user: None,
            presence_penalty: PRESENCE_PENALTY,
            frequency_penalty: FREQUENCY_PENALTY,
            api: OpenAiApi::CompletionApi(CompletionConfig {
                model: COMPLETION_MODEL.to_string(),
                echo: false,
                prompt: None,
                suffix: None,
            }),
            state: std::marker::PhantomData::<Completion>,
        }
    }

    pub fn chat(&self) -> Client<Chat> {
        Client {
            openai: self,
            max_tokens: None,
            temperature: TEMPERATURE,
            top_p: TOP_P,
            n: N,
            user: None,
            presence_penalty: PRESENCE_PENALTY,
            frequency_penalty: FREQUENCY_PENALTY,
            api: OpenAiApi::ChatApi(ChatConfig {
                model: CHAT_MODEL.to_string(),
                messages: Vec::new(),
            }),
            state: std::marker::PhantomData::<Chat>,
        }
    }
}

impl Client<'_, Completion> {
    /// Echo back the prompt in addition to the completion.
    pub fn with_echo(mut self, echo: bool) -> Self {
        match self.api {
            OpenAiApi::CompletionApi(config) => {
                let CompletionConfig {
                    echo: _,
                    model,
                    prompt,
                    suffix,
                } = config;
                self.api = OpenAiApi::CompletionApi(CompletionConfig {
                    echo,
                    model,
                    prompt,
                    suffix,
                })
            }
            _ => unreachable!(),
        }
        self
    }

    /// Create a chat completion
    pub fn create(&self, prompt: &str) {
        println!("Self: {:#?}", self);
        println!("Prompt: {}", prompt);
    }
}

impl Client<'_, Chat> {
    /// Create a chat completion
    pub fn create(&self, prompt: &str) {
        println!("Self: {:#?}", self);
        println!("Prompt: {}", prompt);
    }
}

impl<'a, State> Client<'a, State> {
    /// Configures the ID of the OpenAI model to use.
    /// You can use the `completions.list` method to get a list of all available models.
    pub fn with_model(mut self, model: &str) -> Self {
        match self.api {
            OpenAiApi::CompletionApi(config) => {
                let CompletionConfig {
                    echo,
                    model: _,
                    prompt,
                    suffix,
                } = config;
                self.api = OpenAiApi::CompletionApi(CompletionConfig {
                    echo,
                    model: model.to_string(),
                    prompt,
                    suffix,
                })
            }
            OpenAiApi::ChatApi(config) => {
                let ChatConfig { model: _, messages } = config;
                self.api = OpenAiApi::ChatApi(ChatConfig {
                    model: model.to_string(),
                    messages,
                })
            }
        }
        self
    }

    /// Configures the maximum tokens to generate the completion.
    ///
    /// The token count on your prompt plus `max_tokens` cannot exceed the model's context length.
    /// Most models have a context length of 2048, 4096, or 8192 tokens. GPT-4 also has a model
    /// that supports around 25K tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Configures how many completions to generate for each prompt.
    pub fn with_n(mut self, n: u32) -> Self {
        self.n = n;
        self
    }

    /// Configures the sampling temperature to use, between 0 and 2. Higher values like 0.8 will
    /// make the output more determnistic.
    ///
    /// It's recommended to alter this or the `top_p` parameter, but not both.
    pub fn with_temperature(mut self, temperature: f32) -> Result<Self, error::ClientError> {
        if (0.0..=2.0).contains(&temperature) {
            self.temperature = temperature;
            Ok(self)
        } else {
            Err(error::ClientError::InvalidTemperature { temperature })
        }
    }

    /// Configures the top_p parameter to use, which is an alternative to sampling with temperature,
    /// called nucleus sampling., where the model considers the results of the tokens with `top_p`
    /// probability mass. So, 0.1 means only the tokens comprising the top 10% probability mass are
    /// considered.
    ///
    /// It's recommended to alter this or the `temperature` parameter, but not both.
    pub fn with_top_p(mut self, top_p: f32) -> Result<Self, error::ClientError> {
        if (0.0..=1.0).contains(&top_p) {
            self.top_p = top_p;
            Ok(self)
        } else {
            Err(error::ClientError::InvalidTopP { top_p })
        }
    }

    /// Configures the presence penalty to use, which is a number betweeen -2.0 and 2.0 where
    /// positive values penalize new tokens based on whether they appear in the text so far,
    /// increasing the model's likelihood to talk about new topics.
    pub fn with_presence_penalty(
        mut self,
        presence_penalty: f32,
    ) -> Result<Self, error::ClientError> {
        if (-2.0..=2.0).contains(&presence_penalty) {
            self.presence_penalty = presence_penalty;
            Ok(self)
        } else {
            Err(error::ClientError::InvalidPresencePenalty { presence_penalty })
        }
    }

    /// Configures the frequency penalty, which is a number between -2.0 and 2.0 where positive
    /// values penalize new tokens based on their existing frequency in the text so far, decreasing
    /// the model's likelihood to repeat the same line verbatim.
    pub fn with_frequency_penalty(
        mut self,
        frequency_penalty: f32,
    ) -> Result<Self, error::ClientError> {
        if (-2.0..=2.0).contains(&frequency_penalty) {
            self.frequency_penalty = frequency_penalty;
            Ok(self)
        } else {
            Err(error::ClientError::InvalidFrequencyPenalty { frequency_penalty })
        }
    }
}
