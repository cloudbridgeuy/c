# b

## Snippets

```rust
#[derive(Debug, Parser)]
#[command(name = "v2")]
#[command(about = "Interact with OpenAI's ChatGPT through the terminal")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Name of the chat session.
    #[arg(short, long)]
    session: Option<String>,

    /// URL endpoint of the OpenAI ChatGPT API.
    #[arg(short, long, default_value_t=String::from("https://api.openai.com/v1/chat/completions"))]
    url: String,

    /// ChatGPT model to use.
    #[arg(short, long, default_value_t=String::from("gpt-3.5-turbo"))]
    model: String,

    /// Temperature value of ChatGPT response.
    #[arg(long, default_value_t=0.0, value_parser = in_range)]
    temperature: f32,

    /// Top-p value of ChatGPT response.
    #[arg(long, default_value_t=0.8, value_parser = in_range)]
    top_p: f32,

    /// Presence penalty value of ChatGPT response.
    #[arg(long, default_value_t=0.0, value_parser = in_range)]
    presence_penalty: f32,

    /// Frequencey penalty value of ChatGPT response.
    #[arg(long, default_value_t=0.0, value_parser = in_range)]
    frequency_penalty: f32,

    /// Prompt that should be send to ChatGPT.
    prompt: Vec<String>,
}

fn in_range(s: &str) -> Result<f32, String> {
    let num: f32 = s.parse().map_err(|_| "Not a number".to_string())?;
    if &num < &0.0 {
        Err(String::from("Temperature must be positive"))
    } else if &num > &1.0 {
        Err(String::from("Temperature must be less than 1"))
    } else {
        Ok(num)
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Whisper to OpenAI
    Whisper,
    /// Create new resources
    New {
        #[command(subcommand)]
        command: NewCommand,
    },
}

#[derive(Debug, Subcommand, Clone)]
enum NewCommand {
    /// Create a new chat session
    Chat,
}
```
