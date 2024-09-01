use clap::Parser;
use config_file::FromConfigFile;

mod anthropic;
mod args;
mod config;
mod error;
mod google;
mod openai;
mod prelude;
mod printer;

use crate::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let mut args = Args::parse();

    let mut api: Option<Api> = if let Some(api) = args.globals.api.clone() {
        Some(api.parse()?)
    } else {
        None
    };

    log::info!("info: {:#?}", args.globals);

    let home = std::env::var("HOME")?;
    let path = args.globals.config_file.clone().replace('~', &home);

    log::info!("path: {:#?}", path);

    // Check if `path` exists
    let config = if !std::path::Path::new(&path).exists() {
        Config::default()
    } else {
        Config::from_config_file(path)?
    };

    log::info!("config: {:#?}", config);

    if let Some(preset) = args.globals.preset.clone() {
        let p = config
            .presets
            .unwrap_or_default()
            .into_iter()
            .find(|p| p.name == preset);

        if let Some(p) = p {
            api = Some(p.api);

            if args.globals.top_p.is_none() {
                args.globals.top_p = p.top_p;
            }
            if args.globals.top_k.is_none() {
                args.globals.top_k = p.top_k;
            }
            if args.globals.temperature.is_none() {
                args.globals.temperature = p.temperature;
            }
            if args.globals.system.is_none() {
                args.globals.system = p.system;
            }
            if args.globals.max_tokens.is_none() {
                args.globals.max_tokens = p.max_tokens;
            }
            if args.globals.api_version.is_none() {
                args.globals.api_version = p.version;
            }
            if args.globals.api_env.is_none() {
                args.globals.api_env = p.env;
            }
            if args.globals.api_key.is_none() {
                args.globals.api_key = p.key;
            }
            if args.globals.api_base_url.is_none() {
                args.globals.api_base_url = p.base_url;
            }
            if args.globals.model.is_none() {
                args.globals.model = p.model;
            }
        }
    };

    if args.globals.top_p.is_none() {
        args.globals.top_p = config.top_p;
    }
    if args.globals.top_k.is_none() {
        args.globals.top_k = config.top_k;
    }
    if args.globals.temperature.is_none() {
        args.globals.temperature = config.temperature;
    }
    if args.globals.system.is_none() {
        args.globals.system = config.system;
    }
    if args.globals.max_tokens.is_none() {
        args.globals.max_tokens = config.max_tokens;
    }
    if args.globals.api_version.is_none() {
        args.globals.api_version = config.version;
    }
    if args.globals.api_env.is_none() {
        args.globals.api_env = config.env;
    }
    if args.globals.api_key.is_none() {
        args.globals.api_key = config.key;
    }
    if args.globals.api_base_url.is_none() {
        args.globals.api_base_url = config.base_url;
    }
    if args.globals.model.is_none() {
        args.globals.model = config.model;
    }
    if args.globals.quiet.is_none() {
        args.globals.quiet = config.quiet;
    }
    if api.is_none() {
        api = config.api;
    }

    log::info!("globals: {:#?}", args.globals);

    match api {
        Some(Api::OpenAi) => openai::run(args).await,
        Some(Api::Anthropic) => anthropic::run(args).await,
        Some(Api::Google) => google::run(args).await,
        None => Err(Error::ApiNotSpecified),
    }
}
