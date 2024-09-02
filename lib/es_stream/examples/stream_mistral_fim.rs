use anyhow::Result;
use es_stream::mistral_fim::{Auth, Client, MessageBody};
use futures::stream::TryStreamExt;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let key = std::env::var("MISTRAL_API_KEY")?;

    let auth = Auth::new(key);
    let client = Client::new(auth, "https://api.mistral.ai/v1");

    let prompt = "def coin_problem_solved_with_dp".to_string();
    let suffix = Some("return result".to_string());

    let body = MessageBody::new("codestral-2405", prompt, suffix);

    // let mut stream = client.message_stream(&body)?;
    let mut stream = client.delta(&body)?;

    while let Ok(Some(text)) = stream.try_next().await {
        print!("{text}");
        std::io::stdout().flush()?;
    }

    Ok(())
}
