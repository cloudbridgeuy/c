use log::error;
use std::error::Error;
use std::io;

pub mod gpt;
pub mod record;
pub mod util;

/// Max tokens that will be used for the prompt. Thise leaves
/// 1096 tokens for ChatGPT response.
const MAX_TOKENS: u32 = 3000;
const LAST_REQUEST_FILE: &str = "last_request.json";
const CONFIG_DIRECTORY_PATH: &str = "/tmp/a";

/// Gathers all arguments provided to the binary. If no arguments are provided then stdin
/// is used. The first argument will always be considered the programming language.
///
/// # Errors
///
/// This function will return an error if .
pub fn gather_args(args: &mut Vec<String>) -> Result<(String, String), Box<dyn Error>> {
    let lang;
    let mut prompt = String::new();

    if args.is_empty() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No arguments provided",
        )));
    }

    args.remove(0);
    if args.is_empty() {
        if let Err(e) = io::stdin().read_line(&mut prompt) {
            error!("Could not read from stdin: {}", e);
            return Err(Box::new(e));
        }

        let words: Vec<String> = prompt.split_whitespace().map(|s| s.to_string()).collect();
        if words.len() < 1 {
            error!("Less than one word found");
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Less than one word found",
            )));
        }

        if words[0] != "a" {
            lang = words[0].to_string();
        } else if words.len() >= 2 {
            lang = words[1].to_string();
        } else {
            error!("No language specified");
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No language specified",
            )));
        }
    } else {
        lang = args[0].clone();
        prompt = args.join(" ");
    }

    Ok((prompt, lang))
}
