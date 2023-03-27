use openai::error::ClientError;
use openai::{Authenticated, Client};

fn get_client() -> Client<Authenticated> {
    match Client::new()
        .set_api_key("sk-xvQv2vf8yZ1K2pQ3q9OZqmXj3Xe5R4vZuK")
        .set_model("gpt-4")
        .set_n(1)
        .with_temperature(1.0)
        .and_then(|client| client.with_top_p(0.8))
        .and_then(|client| client.with_frequency_penalty(0.0))
        .and_then(|client| client.with_presence_penalty(0.0))
    {
        Ok(client) => client,
        Err(err) => {
            match err {
                ClientError::InvalidTopP { top_p: _ } => {
                    println!("custom error message. err: {}", err);
                    std::process::exit(3);
                }
                _ => {
                    println!("{}", err);
                    std::process::exit(1);
                }
            };
        }
    }
}

fn main() {
    let client = get_client();
    client.completion("Hello", "It's me");
}
