use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};
use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod server;
mod gpt;

use crate::server::run_server;
use crate::config::{Config, Gpt};
use crate::client_pool::PollableClientFactory;
use crate::gpt::GptClient;

#[macro_use]
mod macros;
mod game_manager;
mod app_error;
mod client_pool;
mod token_gen;
mod game_prompt;
mod config;
mod locale;

struct GptClientFactory {
    config: Gpt,
}

impl GptClientFactory {
    fn new(config: &Config) -> Self {
        Self {
            config: config.gpt.clone(),
        }
    }
}

impl PollableClientFactory<GptClient> for GptClientFactory {
    fn build_client(&self) -> GptClient {
        GptClient::new(&self.config)
    }

    fn get_config(&self) -> &Gpt {
        &self.config
    }
}


fn read_config() -> Result<Config> {
    let config = {
        #[cfg(not(debug_assertions))]
        {
            let args: Vec<String> = env::args().collect();
            let config_path = args.get(1).ok_or_else(|| {
                anyhow::anyhow!("Config file path must be passed as first argument")
            })?;
            Config::read(config_path)?
        }
        #[cfg(debug_assertions)]
        {
            let config_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join("config.toml");
            let config_path = config_file.to_str().ok_or_else(|| {
                anyhow::anyhow!("Invalid UTF-8 in config path")
            })?;
            Config::read(config_path)?
        }
    };

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = read_config()?;
    
    // Initialize logging based on config
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into());
    
    if let Some(logfile) = &config.logfile {
        // Log to file - disable colors
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logfile)?;
        
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(file)
            .with_ansi(false)
            .init();
            
        info!("Logging to file: {}", logfile);
    } else {
        // Log to stdout (default) - keep colors
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_ansi(true)
            .init();
            
        info!("Logging to stdout");
    }
    
    // Initialize locale system
    locale::init_locale(&config);
    
    run_server(&config, Arc::new(GptClientFactory::new(&config))).await?;
    Ok(())
}

//  cargo build --release --target x86_64-unknown-linux-musl
