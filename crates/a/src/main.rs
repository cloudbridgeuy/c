use log::{debug, error, info};

const WHISPER_TRIGGER: &str = "whisper";

fn main() {
    env_logger::init();

    let mut args: Vec<_> = std::env::args().collect();

    let tuple = match a::gather_args(&mut args) {
        Ok(args) => args,
        Err(e) => {
            error!("error parsing arguments: {}", e);
            std::process::exit(1);
        }
    };

    debug!("tuple: {:?}", tuple);
    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            error!("Please set the GPT_API_KEY not set environment variable");
            std::process::exit(1);
        }
    };

    let mut client = a::gpt::GPTClient::new(api_key.to_string());

    let (prompt, lang) = if tuple.1 == crate::WHISPER_TRIGGER {
        let text = match a::record::whisper(api_key) {
            Ok(text) => text,
            Err(e) => {
                error!("error recording whisper: {}", e);
                std::process::exit(1);
            }
        };
        (
            text.clone(),
            text.split_whitespace().next().unwrap_or("text").to_string(),
        )
    } else {
        tuple
    };

    let mut response = match client.prompt(prompt) {
        Ok(response) => response,
        Err(e) => {
            error!("prompt error: {}", e);
            std::process::exit(2);
        }
    };
    debug!("response: {:#?}", response);

    response.push('\n');
    if let Some(r) = response.strip_suffix("\n\n") {
        response = String::from(r);
    }

    #[cfg(feature = "clipboard")]
    {
        a::util::copy_to_clipboard(&response);
        info!("copy to clipboard");
    }

    info!("pretty print to stdout");
    a::util::pretty_print(&a::util::remove_code_lines(&response), &lang);
}
