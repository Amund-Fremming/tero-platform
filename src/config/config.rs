use core::fmt;
use std::env;

use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub static CONFIG: Lazy<AppConfig> =
    Lazy::new(|| AppConfig::load().unwrap_or_else(|e| panic!("{}", e)));

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth0: Auth0Config,
    pub database_url: String,
}

fn default_address() -> String {
    "127.0.0.1".into()
}

fn default_port() -> String {
    "3000".into()
}

fn default_page_size() -> u8 {
    20
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_port")]
    pub port: String,
    pub gs_domain: String,
    #[serde(default = "default_page_size")]
    pub page_size: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth0Config {
    pub domain: String,
    pub audience: String,
    pub webhook_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RunTime {
    Development,
    Production,
}

impl fmt::Display for RunTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunTime::Development => write!(f, "development"),
            RunTime::Production => write!(f, "production"),
        }
    }
}

impl AppConfig {
    fn load() -> Result<Self, ConfigError> {
        let runtime = match env::var("ENVIRONMENT")
            .expect("ENVIRONMENT not set")
            .as_str()
        {
            "DEVELOPMENT" => RunTime::Development,
            "PRODUCTION" => RunTime::Production,
            _ => panic!("Invalid environment set, must be either `DEVELOPMENT` or ´PRODUCTION´"),
        };

        let config: AppConfig = Config::builder()
            .add_source(File::with_name(&format!(
                "src/config/{}.toml",
                runtime.to_string()
            )))
            .add_source(Environment::with_prefix("TERO").separator("__"))
            .build()?
            .try_deserialize()?;

        debug!(
            "Loaded config: {}",
            serde_json::to_string_pretty(&config).unwrap()
        );

        Ok(config)
    }
}
