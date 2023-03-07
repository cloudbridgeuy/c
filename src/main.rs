pub mod gpt;
pub mod util;

const PREVIOUS_PROMPT: &str = "/tmp/a_previous_prompt";

fn main() {
    let mut args: Vec<_> = std::env::args().collect();
    args.remove(0);

    if args.len() == 0 {
        println!("no prompt provided");
        std::process::exit(1);
    }

    let mut lang = args[0].clone();
    let prompt = args.join(" ");
    let api_key = std::env::var("OPENAI_API_KEY").expect("Please set the GPT_API_KEY not set environment variable");

    let client = gpt::GPTClient::new(api_key);
    let mut response = client.prompt(prompt).expect("Could not make request to API");

    response.push_str("\n");
    if let Some(r) = response.strip_suffix("\n\n") {
        response = String::from(r);
    }

    #[cfg(feature = "clipboard")]
    {
        util::copy_to_clipboard(&response);
    }

    util::write_to_file(PREVIOUS_PROMPT, &response).unwrap();

    util::pretty_print(&response, &mut lang);
}