use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_exp_hours")]
    pub jwt_exp_hours: u64,
    pub admin_username: String,
    pub admin_password: String,
    pub session_secret: String,
}

fn default_host() -> String { "127.0.0.1".to_string() }
fn default_port() -> u16 { 3000 }
fn default_jwt_secret() -> String { "change-this-in-production".to_string() }
fn default_jwt_exp_hours() -> u64 { 72 }

pub fn load_config(path: &Path) -> anyhow::Result<Config> {
    let config_path = std::env::var("CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| path.to_path_buf());
    let content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
