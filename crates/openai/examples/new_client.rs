use openai;
use openai::error::ClientError;

fn main() {
    let openai = openai::OpenAi::new(String::from("sk-xvQv2vf8yZ1K2pQ3q9OZqmXj3Xe5R4vZuK"));

    // Load the completion client and handle specific errors by matching against the ClientError enum.
    let completion = match openai
        .completions()
        .with_model("text-davinci-003")
        .with_echo(false)
        .with_temperature(0.0)
        .and_then(|client| client.with_top_p(0.8))
    {
        Ok(client) => client,
        Err(err) => match err {
            ClientError::InvalidTopP { top_p: _ } => {
                println!("custom error message. err: {}", err);
                std::process::exit(3);
            }
            _ => {
                println!("{}", err);
                std::process::exit(1);
            }
        },
    };

    // Load the chat client and handle specific errors by using the expect method.
    let chat = openai
        .chat()
        .with_model("gpt-4")
        .with_temperature(0.0)
        .and_then(|c| c.with_top_p(0.8))
        .and_then(|c| c.with_frequency_penalty(0.0))
        .expect("Error setting the OpenAi Chat Client");

    // Call the create method on each client.
    completion.create("Completion");
    chat.create("Chat")
}
