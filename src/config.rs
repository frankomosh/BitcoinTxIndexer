use config::{Config as ConfigBuilder, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    // Bitcoin node
    pub bitcoin_rpc_url: String,
    pub bitcoin_rpc_user: String,
    pub bitcoin_rpc_pass: String,
    pub bitcoin_network: String,

    // Database
    pub database_url: String,

    // API
    pub api_host: String,
    pub api_port: u16,

    // Indexer
    pub indexer_start_height: u64,
    pub indexer_batch_size: usize,
    pub indexer_poll_interval_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok();

        let config = ConfigBuilder::builder()
            .add_source(Environment::default())
            .build()?;

        config.try_deserialize()
    }
}
