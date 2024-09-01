use es_stream::openai;
use futures::stream::TryStreamExt;
use std::io::Write;

use crate::prelude::*;

const DEFAULT_URL: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o";

pub async fn run(args: Args) -> Result<()> {
    let key = match args.globals.api_key {
        Some(key) => key,
        None => {
            let environment_variable = match args.globals.api_env {
                Some(env) => env,
                None => "OPENAI_API_KEY".to_string(),
            };
            std::env::var(environment_variable)?
        }
    };
    let url = match args.globals.api_base_url {
        Some(url) => url,
        None => DEFAULT_URL.to_string(),
    };

    let auth = openai::Auth::new(key);

    let client = openai::Client::new(auth, url);

    let messages = vec![openai::Message {
        role: openai::Role::User,
        content: args.globals.prompt.into_inner(),
    }];

    let body = openai::MessageBody::new(
        args.globals
            .model
            .unwrap_or(DEFAULT_MODEL.to_string())
            .as_ref(),
        messages,
    );

    // let mut stream = client.message_stream(&body)?;
    let mut stream = client.delta(&body)?;

    while let Ok(Some(text)) = stream.try_next().await {
        print!("{text}");
        std::io::stdout().flush()?;
    }

    Ok(())
}
