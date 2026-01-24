use std::env;

/// Application configuration loaded from environment variables
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables
    /// Returns error if required variables are missing
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::Missing("DATABASE_URL"))?;

        let redis_url = env::var("REDIS_URL")
            .map_err(|_| ConfigError::Missing("REDIS_URL"))?;

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|_| ConfigError::Invalid("SERVER_PORT"))?;

        Ok(Self {
            database_url,
            redis_url,
            server_host,
            server_port,
        })
    }

    /// Get the full server address for binding
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    /// Get the base URL for generating short URLs
    pub fn base_url(&self) -> String {
        if self.server_port == 80 {
            format!("http://{}", self.server_host)
        } else {
            format!("http://{}:{}", self.server_host, self.server_port)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    Missing(&'static str),
    #[error("Invalid environment variable: {0}")]
    Invalid(&'static str),
}
