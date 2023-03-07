use log::{info, debug, error};

pub mod gpt;
pub mod util;

const LAST_REQUEST_FILE: &str = "last_request.json";
const CONFIG_DIRECTORY_PATH: &str = "/tmp/a";

fn main() {
    env_logger::init();

    let mut args: Vec<_> = std::env::args().collect();
    args.remove(0);

    if args.len() == 0 {
        error!("no prompt provided");
        std::process::exit(1);
    }

    let mut lang = args[0].clone();
    let prompt = args.join(" ");
    debug!("prompt: {}", prompt);
    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            error!("Please set the GPT_API_KEY not set environment variable");
            std::process::exit(1);
        }
    };

    let mut client = gpt::GPTClient::new(api_key);
    debug!("client: {:#?}", client);
    let mut response = match client.prompt(prompt) {
        Ok(response) => response,
        Err(e) => {
            error!("prompt error: {}", e);
            std::process::exit(2);
        }
    };
    debug!("response: {:#?}", response);

    response.push_str("\n");
    if let Some(r) = response.strip_suffix("\n\n") {
        response = String::from(r);
    }

    #[cfg(feature = "clipboard")]
    {
        util::copy_to_clipboard(&response);
        info!("copy to clipboard");
    }

    info!("pretty print to stdout");
    util::pretty_print(&util::remove_code_lines(&response), &mut lang);
}
