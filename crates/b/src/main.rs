use clap::{Parser, Subcommand, ValueEnum};
use log;
use serde_either::SingleOrVec;

use openai::completions::CompletionsApi;

/// A simple program to greet a person.
#[derive(Debug, Parser)]
#[command(name = "v2")]
#[command(about = "Interact with OpenAI's ChatGPT through the terminal")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// OpenAI API Key to use. Will default to the environment variable `OPENAI_API_KEY` if not
    /// set.
    #[arg(long)]
    api_key: Option<String>,
    #[clap(short, long, value_enum, default_value_t = Output::Raw)]
    output: Output,
}

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
enum Output {
    /// Plain text
    Raw,
    /// JSON
    Json,
    /// YAML
    Yaml,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Whisper to OpenAI
    Completions {
        #[command(subcommand)]
        command: CompletionsCommands,
    },
}

#[derive(Debug, Subcommand, Clone)]
enum CompletionsCommands {
    /// Create a new chat session
    Create {
        /// ID of the model to use. Use the `modesl list` command to see all your available models
        /// or see the following link: https://platform.openai.com/docs/models/overview
        #[arg(long, default_value = "text-davinci-003")]
        model: String,
        /// The prompt(s) to generate completions for, encoded as a string, array of strings, array
        /// of tokens, or array of token arrays.
        #[arg(long)]
        prompt: Option<Vec<String>>,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Completions { ref command }) => handle_completions(&cli, &command),
        None => {
            println!("{:#?}", cli);
            Ok(())
        }
    }
}

fn get_api_key() -> String {
    match std::env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            log::error!("Please set the GPT_API_KEY not set environment variable");
            std::process::exit(1);
        }
    }
}

fn handle_completions(
    cli: &Cli,
    command: &CompletionsCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        CompletionsCommands::Create {
            model,
            prompt,
            suffix,
            max_tokens,
            temperature,
            top_p,
            n,
            stream,
            logprobs,
            echo,
            stop,
            presence_penalty,
            frequency_penalty,
            best_of,
            user,
        } => {
            let mut api = CompletionsApi::new(get_api_key());

            if let Some(prompt) = prompt {
                api.prompt = Some(SingleOrVec::Vec(prompt.to_owned()));
            }

            api.model = model.to_owned();
            api.max_tokens = *max_tokens;
            api.n = *n;
            api.user = user.to_owned();

            if let Some(echo) = echo {
                api.set_echo(*echo)?;
            }
            if let Some(stream) = stream {
                api.set_stream(*stream)?;
            }
            if let Some(suffix) = suffix {
                api.set_suffix(suffix.to_string())?;
            }
            if let Some(best_of) = best_of {
                api.set_best_of(*best_of)?;
            }
            if let Some(stop) = stop {
                api.set_stop(SingleOrVec::Vec(stop.to_owned()))?;
            }
            if let Some(logprobs) = logprobs {
                api.set_logprobs(*logprobs)?;
            }
            if let Some(temperature) = temperature {
                api.set_temperature(*temperature)?;
            }
            if let Some(top_p) = top_p {
                api.set_top_p(*top_p)?;
            }
            if let Some(presence_penalty) = presence_penalty {
                api.set_presence_penalty(*presence_penalty)?;
            }
            if let Some(frequency_penalty) = frequency_penalty {
                api.set_frequency_penalty(*frequency_penalty)?;
            }

            let completions = api.create()?;

            match cli.output {
                Output::Json => {
                    serde_json::to_writer(std::io::stdout(), &completions)?;
                }
                Output::Yaml => {
                    serde_yaml::to_writer(std::io::stdout(), &completions)?;
                }
                Output::Raw => {
                    println!("{:#?}", completions);
                }
            }

            Ok(())
        }
    }
}
