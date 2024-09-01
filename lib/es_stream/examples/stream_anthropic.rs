use anyhow::Result;
use es_stream::anthropic::{Auth, Client, Message, MessageBody, Role};
use futures::stream::TryStreamExt;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let key = std::env::var("ANTHROPIC_API_KEY")?;

    let auth = Auth::new(key, None);
    let client = Client::new(auth, "https://api.anthropic.com/v1");

    let messages = vec![Message {
        role: Role::User,
        content: "What is the capital of the United States?".to_string(),
    }];

    let body = MessageBody::new("claude-3-opus-20240229", messages, 300);

    // let mut stream = client.message_stream(&body)?;
    let mut stream = client.delta(&body)?;

    while let Ok(Some(text)) = stream.try_next().await {
        print!("{text}");
        std::io::stdout().flush()?;
    }

    Ok(())
}
