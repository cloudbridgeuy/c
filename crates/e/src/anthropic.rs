use es_stream::anthropic;

use crate::prelude::*;

const DEFAULT_URL: &str = "https://api.anthropic.com/v1";
const DEFAULT_MODEL: &str = "claude-3-5-sonnet-20240620";
const DEFAULT_ENV: &str = "ANTHROPIC_API_KEY";

pub async fn run(prompt: String, args: Args) -> Result<()> {
    let key = match args.globals.api_key {
        Some(key) => key,
        None => {
            let environment_variable = match args.globals.api_env {
                Some(env) => env,
                None => DEFAULT_ENV.to_string(),
            };
            std::env::var(environment_variable)?
        }
    };
    log::info!("key: {}", key);

    let url = match args.globals.api_base_url {
        Some(url) => url,
        None => DEFAULT_URL.to_string(),
    };
    log::info!("url: {}", url);

    let auth = anthropic::Auth::new(key, args.globals.api_version);

    log::info!("auth: {:#?}", auth);

    let client = anthropic::Client::new(auth, url);

    log::info!("client: {:#?}", client);

    let messages = vec![anthropic::Message {
        role: anthropic::Role::User,
        content: prompt,
    }];

    let mut body = anthropic::MessageBody::new(
        args.globals
            .model
            .unwrap_or(DEFAULT_MODEL.to_string())
            .as_ref(),
        messages,
        args.globals.max_tokens.unwrap_or(4096),
    );

    body.system = args.globals.system;
    body.temperature = args.globals.temperature;
    body.top_p = args.globals.top_p;
    body.top_k = args.globals.top_k;

    log::info!("body: {:#?}", body);

    let stream = client.delta(&body)?;

    handle_stream(stream, args.globals.quiet.unwrap_or(false)).await
}
