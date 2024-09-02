use es_stream::mistral_fim;

use crate::prelude::*;

const DEFAULT_URL: &str = "https://api.mistral.ai/v1";
const DEFAULT_MODEL: &str = "codestral-2405";
const DEFAULT_ENV: &str = "MISTRAL_API_KEY";

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

    let auth = mistral_fim::Auth::new(key);

    log::info!("auth: {:#?}", auth);

    let client = mistral_fim::Client::new(auth, url);

    log::info!("client: {:#?}", client);

    let mut body = mistral_fim::MessageBody::new(
        args.globals
            .model
            .unwrap_or(DEFAULT_MODEL.to_string())
            .as_ref(),
        prompt,
        args.globals.suffix,
    );

    body.temperature = args.globals.temperature;
    body.top_p = args.globals.top_p;
    if let Some(max_tokens) = args.globals.max_tokens {
        body.max_tokens = Some(max_tokens);
    };
    if let Some(min_tokens) = args.globals.min_tokens {
        body.min_tokens = Some(min_tokens);
    };

    log::info!("body: {:#?}", body);

    let stream = client.delta(&body)?;

    handle_stream(
        stream,
        args.globals.quiet.unwrap_or(false),
        args.globals.language,
    )
    .await
}
