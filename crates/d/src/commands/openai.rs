use std::io::{Read, Write};

use clap::{Parser, ValueEnum};
use color_eyre::eyre::{bail, Result};
use crossterm::{cursor, execute};
use openai::chat::{
    ChatCompletion, ChatCompletionDelta, ChatCompletionMessage, ChatCompletionMessageRole,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

#[derive(ValueEnum, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
enum Model {
    #[default]
    #[serde(rename = "gpt-4-1106-preview")]
    GPT41106Preview,
    #[serde(rename = "gpt-4")]
    GPT4,
    #[serde(rename = "gpt-4-32k")]
    GPT432K,
    #[serde(rename = "gpt-3.5-turbo")]
    GPT35Turbo,
    #[serde(rename = "gpt-3.5-turbo-16k")]
    GPT35Turbo16K,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::GPT41106Preview => "gpt-4-1106-preview",
            Model::GPT4 => "gpt-4",
            Model::GPT432K => "gpt-4-32k",
            Model::GPT35Turbo => "gpt-3.5-turbo",
            Model::GPT35Turbo16K => "gpt-3.5-turbo-16k",
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            Model::GPT41106Preview => 128000,
            Model::GPT4 => 8000,
            Model::GPT432K => 32000,
            Model::GPT35Turbo => 4000,
            Model::GPT35Turbo16K => 16000,
        }
    }
}

#[derive(Default, Clone, Parser, Debug, Serialize, Deserialize)]
pub struct CommandOptions {
    /// The content of the message to be sent to the chatbot. You can also populate this value
    /// from stdin. If you pass a value here and pipe data from stdin, both will be sent to the
    /// API, stdin taking precedence.
    prompt: Option<String>,
    /// ID of the model to use. See the following link: https://platform.openai.com/docs/models/overview
    #[clap(short, long, value_enum)]
    model: Option<Model>,
    /// Chat session name. Will be used to store previous session interactions.
    #[arg(long)]
    session: Option<String>,
    /// The system message helps set the behavior of the assistant.
    #[arg(long)]
    system: Option<String>,
}

/// Reads from `stdin`
fn read_stdin() -> Result<String> {
    let mut input = String::new();

    // Read the entire contents of stdin into the string
    match std::io::stdin().read_to_string(&mut input) {
        Ok(_) => Ok(input),
        Err(e) => {
            bail!("[read_stdin] Error: {e}");
        }
    }
}

/// Runs the `openai` command
pub async fn run(options: CommandOptions) -> Result<()> {
    let mut messages = Vec::new();

    if options.system.is_some() {
        messages.push(ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            content: options.system,
            name: None,
            function_call: None,
        });
    } else {
        messages.push(ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            content: Some("You're an AI that with detail and using markdown to format your answers, with proper code fences when you need to write code.".to_string()),
            name: None,
            function_call: None,
        });
    }

    match options.prompt {
        Some(prompt) => {
            if !atty::is(atty::Stream::Stdin) {
                messages.push(ChatCompletionMessage {
                    role: ChatCompletionMessageRole::User,
                    content: Some(read_stdin()?),
                    name: None,
                    function_call: None,
                });
            }

            if !prompt.is_empty() {
                messages.push(ChatCompletionMessage {
                    role: ChatCompletionMessageRole::User,
                    content: Some(prompt),
                    name: None,
                    function_call: None,
                });
            }
        }
        None => {
            if atty::is(atty::Stream::Stdin) {
                print!("User: ");

                std::io::stdout().flush()?;

                let mut user_message_content = String::new();

                std::io::stdin().read_line(&mut user_message_content)?;

                messages.push(ChatCompletionMessage {
                    role: ChatCompletionMessageRole::User,
                    content: Some(user_message_content.to_string()),
                    name: None,
                    function_call: None,
                });
            }
        }
    }

    fetch(messages).await
}

/// Prepare `openai` request
async fn fetch(mut messages: Vec<ChatCompletionMessage>) -> Result<()> {
    let chat_stream = ChatCompletionDelta::builder("gpt-3.5-turbo", messages.clone())
        .create_stream()
        .await?;

    let chat_completion: ChatCompletion = listen_for_tokens(chat_stream).await?;
    let returned_message = chat_completion
        .choices
        .first()
        .expect("A response choice was expected")
        .message
        .clone();

    messages.push(returned_message);

    Ok(())
}

/// Handle streaming output
async fn listen_for_tokens(
    mut chat_stream: Receiver<ChatCompletionDelta>,
) -> Result<ChatCompletion> {
    let mut merged: Option<ChatCompletionDelta> = None;
    let mut previous_output = String::new();
    let mut accumulated_content_bytes = Vec::new();

    execute!(std::io::stdout(), cursor::Hide)?;
    while let Some(delta) = chat_stream.recv().await {
        let choice = &delta.choices[0];

        if let Some(content) = &choice.delta.content {
            accumulated_content_bytes.extend_from_slice(content.as_bytes());

            let output = crate::printer::CustomPrinter::new()
                .input_from_bytes(&accumulated_content_bytes)
                .print()?;

            let unprinted_lines = output
                .lines()
                .skip(if previous_output.lines().count() == 0 {
                    0
                } else {
                    previous_output.lines().count() - 1
                })
                .collect::<Vec<_>>()
                .join("\n");

            execute!(std::io::stdout(), cursor::MoveToColumn(0))?;
            print!("{unprinted_lines}");
            std::io::stdout().flush()?;

            // Update the previous output
            previous_output = output;
        }

        if choice.finish_reason.is_some() {
            // The message being streamed has been fully received.
            println!();
        }

        // Merge completion into accrued.
        match merged.as_mut() {
            Some(c) => {
                c.merge(delta).unwrap();
            }
            None => merged = Some(delta),
        };
    }

    execute!(std::io::stdout(), cursor::Show)?;
    std::io::stdout().flush()?;

    Ok(merged.unwrap().into())
}
