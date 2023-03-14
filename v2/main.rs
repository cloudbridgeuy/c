use clap::{Parser, Subcommand};

/// A simple program to greet a person.
#[derive(Debug, Parser)]
#[command(name="v2")]
#[command(about="Interact with OpenAI's ChatGPT through the terminal")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long)]
    session: Option<String>,

    #[arg(short, long, default_value_t=String::from("https://api.openai.com/v1/chat/completions"))]
    url: String,

    #[arg(short, long, default_value_t=String::from("gpt-3.5-turbo"))]
    model: String,

    #[arg(long, default_value_t=0.0, value_parser = in_range)]
    temperature: f32,

    #[arg(long, default_value_t=0.8, value_parser = in_range)]
    top_p: f32,

    #[arg(long, default_value_t=0.0, value_parser = in_range)]
    presence_penalty: f32,

    #[arg(long, default_value_t=0.0, value_parser = in_range)]
    frequency_penalty: f32,

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
    }
}

#[derive(Debug, Subcommand)]
enum NewCommand {
    /// Create a new chat session
    Chat,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Whisper) => {
            println!("Whisper to OpenAI");
        }
        Some(Commands::New { command }) => {
            handle_new_command(&command);
        }
        None => {
            handle_chat_session(&cli);
        }
    }
}

fn handle_new_command(command: &NewCommand) {
    match command {
        NewCommand::Chat => {
            println!("Create a new chat session");
        }
    }
}

fn handle_chat_session(cli: &Cli) {
    let _url = &cli.url;
    let _model = &cli.model;
    let _temperature = &cli.temperature;
    let _top_p = &cli.top_p;
    let _presence_penalty = &cli.presence_penalty;
    let _frequency_penalty = &cli.frequency_penalty;
    let _prompt = &cli.prompt;

    println!("{:#?}", &cli);
}
