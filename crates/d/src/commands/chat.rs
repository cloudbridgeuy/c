use std::io::{Read, Write};

use clap::Parser;
use color_eyre::eyre::{bail, Result};
use crossterm::{cursor, execute};
use openai::chat::{ChatCompletion, ChatCompletionDelta, ChatCompletionMessageRole};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

use crate::models::Model;
use crate::sessions::Session;

#[derive(Default, Clone, Parser, Debug, Serialize, Deserialize)]
pub struct Options {
    /// The content of the message to be sent to the chatbot. You can also populate this value
    /// from stdin. If you pass a value here and pipe data from stdin, both will be sent to the
    /// API, stdin taking precedence.
    prompt: Option<String>,
    /// ID of the model to use.
    #[clap(short, long, value_enum)]
    model: Option<Model>,
    /// Chat session name. Will be used to store previous session interactions.
    #[arg(long)]
    session: Option<String>,
    /// DB collection where the new messages should be stored.
    #[arg(long)]
    collection: Option<String>,
    /// The system message helps set the behavior of the assistant.
    #[arg(long)]
    system: Option<String>,
    /// The temperature value to use for the session.
    #[arg(long)]
    temperature: Option<f32>,
    /// The top_p value to use for the session
    #[arg(long)]
    top_p: Option<f32>,
    /// The max_tokens value to use for the session
    #[arg(long)]
    max_tokens: Option<u64>,
    /// Don't perform the request and instead print the session to stdout.
    #[arg(long)]
    dry_run: bool,
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
pub async fn run(mut options: Options) -> Result<()> {
    let mut session = match options.session {
        Some(session) => Session::load(session)?,
        None => Session::new(),
    };

    if let Some(collection) = options.collection.take() {
        session.collection = Some(collection);
    }

    if let Some(model) = options.model.take() {
        session.model = model;
    }

    if let Some(temperature) = options.temperature.take() {
        session.set_temperature(temperature)?;
    }

    if let Some(top_p) = options.top_p.take() {
        session.set_top_p(top_p)?;
    }

    if let Some(max_tokens) = options.max_tokens.take() {
        session.set_max_tokens(max_tokens)?;
    }

    if let Some(system) = options.system.take() {
        session.system(system);
    }

    if !atty::is(atty::Stream::Stdin) {
        session.push(read_stdin()?, ChatCompletionMessageRole::User)
    }

    match options.prompt {
        Some(prompt) => {
            if !prompt.is_empty() {
                session.push(prompt, ChatCompletionMessageRole::User)
            }
        }
        None => {
            if atty::is(atty::Stream::Stdin) {
                print!("User: ");

                std::io::stdout().flush()?;

                let mut user_message_content = String::new();

                std::io::stdin().read_line(&mut user_message_content)?;

                session.push(
                    user_message_content.to_string(),
                    ChatCompletionMessageRole::User,
                )
            }
        }
    }

    if options.dry_run {
        println!("{}", serde_yaml::to_string(&session)?);
        return Ok(());
    }

    let messages = session.completion_messages();
    let chat_stream = ChatCompletionDelta::builder(
        options.model.unwrap_or(Model::GPT41106Preview).as_str(),
        messages,
    )
    .temperature(session.get_temperature())
    .top_p(session.get_top_p())
    .max_tokens(session.get_max_tokens())
    .create_stream()
    .await?;

    let chat_completion: ChatCompletion = listen_for_tokens(chat_stream).await?;
    let returned_message = chat_completion
        .choices
        .first()
        .expect("A response choice was expected")
        .message
        .clone();

    session.push(
        returned_message.content.unwrap(),
        ChatCompletionMessageRole::Assistant,
    );

    session.save().await?;

    Ok(())
}

/// Handle streaming output
async fn listen_for_tokens(
    mut chat_stream: Receiver<ChatCompletionDelta>,
) -> Result<ChatCompletion> {
    let mut merged: Option<ChatCompletionDelta> = None;
    let mut previous_output = String::new();
    let mut accumulated_content_bytes = Vec::new();
    let mut sp: Option<spinners::Spinner> = None;

    if atty::is(atty::Stream::Stdout) {
        sp = Some(spinners::Spinner::new(
            spinners::Spinners::OrangeBluePulse,
            "Loading...".into(),
        ));
    }

    while let Some(delta) = chat_stream.recv().await {
        let choice = &delta.choices[0];

        if let Some(content) = &choice.delta.content {
            if atty::is(atty::Stream::Stdout) && sp.is_some() {
                sp.take().unwrap().stop();
                std::io::stdout().flush()?;
            }

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

    std::io::stdout().flush()?;

    Ok(merged.unwrap().into())
}
