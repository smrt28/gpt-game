#![allow(unused_imports)]

#[macro_use]

mod server;
mod gpt;
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;
use tracing::info;
use crate::server::run_server;
use crate::config::{Config, Gpt};
use crate::client_pool::*;
use crate::gpt::GptClient;
use tracing_subscriber::EnvFilter;
use std::{env, fs};
use axum::extract::connect_info;

#[macro_use]
mod macros;
mod game_manager;
mod app_error;
mod client_pool;
mod token_gen;
mod game_prompt;
mod built_in_options;
mod config;

struct GptClientFactory {
    config: Gpt,
}

impl GptClientFactory {
    fn new(config: &Config) -> GptClientFactory {
        GptClientFactory {
            config: config.gpt.clone(),
        }
    }
}

impl PollableClientFactory::<GptClient> for GptClientFactory {
    fn build_client(&self) -> GptClient {
        GptClient::new(&self.config)
    }

    fn get_config(&self) -> &config::Gpt {
        &self.config
    }
}

fn app_root() -> PathBuf {
    #[cfg(debug_assertions)]
    let res = {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    };
    #[cfg(not(debug_assertions))]
    let res = {
        PathBuf::from(std::env::var("APP_ROOT")
            .expect("APP_ROOT env var must be set in release builds"))
    };

    info!("config: {:?}", res);
    res
}

fn read_config() -> Result<config::Config, anyhow::Error> {
    #[cfg(not(debug_assertions))]
    let config = {
        let args: Vec<String> = env::args().collect();
        config::Config::read(args.get(1)?)?;
    };

    #[cfg(debug_assertions)]
    let config = {
        let config_file = app_root()
            .join("assets")
            .join("config.toml");
        config::Config::read(config_file.to_str().unwrap())?
    };

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
        )
        .init();
    let config = read_config()?;
    run_server(&config, Arc::new(GptClientFactory::new(&config))).await?;
    Ok(())
}

//  cargo build --release --target x86_64-unknown-linux-musl