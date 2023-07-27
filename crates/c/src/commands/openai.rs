use clap::Parser;
use color_eyre::eyre::Result;

#[derive(Parser, Debug)]
pub struct Options {
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
    /// OpenAI API Key to use. Will default to the environment variable `OPENAI_API_KEY` if not set.
    #[arg(long, env = "OPENAI_API_KEY")]
    openai_api_key: Option<String>,
    /// Silent mode
    #[clap(short, long, action, default_value_t = false)]
    silent: bool,
    /// Wether to incrementally stream the response using SSE.
    #[clap(long)]
    stream: bool,
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid key-value pair: {}", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

pub async fn run(options: Options) -> Result<()> {
    println!("{:?}", options);

    Ok(())
}
