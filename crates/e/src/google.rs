use es_stream::google;

use crate::prelude::*;

const DEFAULT_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const DEFAULT_MODEL: &str = "gemini-1.5-pro";

pub async fn run(args: Args) -> Result<()> {
    let key = match args.globals.api_key {
        Some(key) => key,
        None => {
            let environment_variable = match args.globals.api_env {
                Some(env) => env,
                None => "GOOGLE_API_KEY".to_string(),
            };
            std::env::var(environment_variable)?
        }
    };
    let url = match args.globals.api_base_url {
        Some(url) => url,
        None => DEFAULT_URL.to_string(),
    };

    let auth = google::Auth::new(key);

    let client = google::Client::new(auth, url);

    let contents = vec![google::Content {
        parts: vec![google::Part {
            text: args.globals.prompt.into_inner(),
        }],
        role: google::Role::User,
    }];

    let mut body = google::MessageBody::new(
        args.globals
            .model
            .unwrap_or(DEFAULT_MODEL.to_string())
            .as_ref(),
        contents,
    );

    if let Some(system) = args.globals.system {
        let system_message = google::Content {
            parts: vec![google::Part { text: system }],
            role: google::Role::User,
        };

        body.contents.insert(0, system_message);
    }

    body.generation_config = Some(google::GenerationConfig {
        max_output_tokens: Some(u32::try_from(args.globals.max_tokens)?),
        temperature: args.globals.temperature,
        top_p: args.globals.top_p,
        top_k: args.globals.top_k,
        ..Default::default()
    });

    let stream = client.delta(&body)?;

    handle_stream(stream, args.globals.quiet).await
}
