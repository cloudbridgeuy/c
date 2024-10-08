use anyhow::Result;
use es_stream::mistral::{Auth, Client, Message, MessageBody, Role};
use futures::stream::TryStreamExt;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let key = std::env::var("MISTRAL_API_KEY")?;

    let auth = Auth::new(key);
    let client = Client::new(auth, "https://api.mistral.ai/v1");

    let messages = vec![Message {
        role: Role::User,
        content: "What is the capital of the United States?".to_string(),
    }];

    let body = MessageBody::new("mistral-small-latest", messages);

    // let mut stream = client.message_stream(&body)?;
    let mut stream = client.delta(&body)?;

    while let Ok(Some(text)) = stream.try_next().await {
        print!("{text}");
        std::io::stdout().flush()?;
    }

    Ok(())
}
