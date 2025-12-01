use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server_port: String,
    pub database_url: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "3000".into());
        let database_url = env::var("DATABASE_URL").ok();
        Ok(Self {
            server_port,
            database_url,
        })
    }
}
