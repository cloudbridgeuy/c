use es_stream::anthropic;

use crate::prelude::*;

const DEFAULT_URL: &str = "https://api.anthropic.com/v1";
const DEFAULT_MODEL: &str = "claude-3-5-sonnet-20240620";

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
        None => DEFAULT_URL.to_string(),
    };

    let auth = anthropic::Auth::new(key, args.globals.api_version);

    let client = anthropic::Client::new(auth, url);

    let messages = vec![anthropic::Message {
        role: anthropic::Role::User,
        content: args.globals.prompt.into_inner(),
    }];

    let mut body = anthropic::MessageBody::new(
        args.globals
            .model
            .unwrap_or(DEFAULT_MODEL.to_string())
            .as_ref(),
        messages,
        args.globals.max_tokens,
    );

    body.system = args.globals.system;

    let stream = client.delta(&body)?;

    handle_stream(stream, args.globals.quiet).await
}
