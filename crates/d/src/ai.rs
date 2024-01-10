use std::io::Write;

use color_eyre::eyre::Result;
use crossterm::{cursor, execute, terminal};
use openai::chat::{
    ChatCompletion, ChatCompletionDelta, ChatCompletionMessage, ChatCompletionMessageRole,
};
use tokio::sync::mpsc::Receiver;

struct CustomPrinter<'a> {
    inputs: Vec<bat::input::Input<'a>>,
    config: bat::config::Config<'a>,
    assets: bat::assets::HighlightingAssets,
    term_width: Option<usize>,
}

impl<'a> CustomPrinter<'a> {
    pub fn new() -> Self {
        let config = bat::config::Config {
            colored_output: true,
            true_color: true,
            language: Some(LANGUAGE),
            theme: THEME.to_string(),
            use_italic_text: true,
            wrapping_mode: bat::WrappingMode::Character,
            ..Default::default()
        };

        CustomPrinter {
            inputs: vec![],
            config,
            assets: bat::assets::HighlightingAssets::from_binary(),
            term_width: None,
        }
    }

    /// Add a byte string as an input
    pub fn input_from_bytes(&mut self, content: &'a [u8]) -> &mut Self {
        self.input_from_reader(content)
    }

    /// Add a custom reader as an input
    pub fn input_from_reader<R: std::io::Read + 'a>(&mut self, reader: R) -> &mut Self {
        self.inputs
            .push(bat::input::Input::from_reader(Box::new(reader)));
        self
    }

    pub fn print(&mut self) -> Result<String> {
        self.config.term_width = self
            .term_width
            .unwrap_or_else(|| terminal::size().unwrap().0 as usize);

        // Collect the inputs to print
        let inputs = std::mem::take(&mut self.inputs);

        // Create the output string
        let mut output = String::new();

        // Run the cotroller
        let controller = bat::controller::Controller::new(&self.config, &self.assets);
        controller.run(inputs, Some(&mut output))?;

        Ok(output)
    }
}

// Markdown language constant string
const THEME: &str = "ansi";
const LANGUAGE: &str = "markdown";

pub async fn run() -> color_eyre::eyre::Result<()> {
    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some("You're an AI that with detail and using markdown to format your answers, with proper code fences when you need to write code.".to_string()),
        name: None,
        function_call: None,
    }];

    loop {
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
    }
}

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

            let output = CustomPrinter::new()
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
