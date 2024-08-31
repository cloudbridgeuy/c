use es_stream::anthropic;
use futures::stream::TryStreamExt;
use std::io::Write;

use crate::prelude::*;

pub async fn run(args: Args) -> Result<()> {
    let key = match args.globals.api_key {
        Some(key) => key,
        None => {
            let environment_variable = match args.globals.api_env {
                Some(env) => env,
                None => "ANTHROPIC_API_KEY".to_string(),
            };
            std::env::var(environment_variable)?
        }
    };
    let url = match args.globals.api_base_url {
        Some(url) => url,
        None => "https://api.anthropic.com/v1/".to_string(),
    };

    let auth = anthropic::Auth::new(key, args.globals.api_version);

    let client = anthropic::Client::new(auth, url);

    let messages = vec![anthropic::Message {
        role: anthropic::Role::User,
        content: args.globals.prompt.into_inner(),
    }];

    let body = anthropic::MessageBody::new(
        args.globals
            .model
            .unwrap_or("claude-3-5-sonnet-20240620".to_string())
            .as_ref(),
        messages,
        args.globals.max_tokens,
    );

    // let mut stream = client.message_stream(&body)?;
    let mut stream = client.delta(&body)?;

    while let Ok(Some(text)) = stream.try_next().await {
        print!("{text}");
        std::io::stdout().flush()?;
    }

    Ok(())
}
