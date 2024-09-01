use anyhow::Result;
use es_stream::google::{Auth, Client, Content, MessageBody, Part, Role};
use futures::stream::TryStreamExt;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let key = std::env::var("GOOGLE_API_KEY")?;

    let auth = Auth::new(key);
    let client = Client::new(auth, "https://generativelanguage.googleapis.com/v1beta");

    let messages = vec![Content {
        parts: vec![Part {
            text: "What is the capital of the United States?".to_string(),
        }],
        role: Role::User,
    }];

    let body = MessageBody::new("gemini-1.5-flash", messages);

    // let mut stream = client.message_stream(&body)?;
    let mut stream = client.delta(&body)?;

    while let Ok(Some(text)) = stream.try_next().await {
        print!("{text}");
        std::io::stdout().flush()?;
    }

    Ok(())
}
