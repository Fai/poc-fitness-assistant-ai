//! Configuration management for the Fitness Assistant backend
//!
//! Configuration is loaded hierarchically:
//! 1. Default values (in code)
//! 2. TOML config files (config/development.toml or config/production.toml)
//! 3. Environment variables (prefix: FA__)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    #[serde(default)]
    pub ai: AiConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_secs: i64,
    pub refresh_token_expiry_secs: i64,
}

/// AI/LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub enabled: bool,
    pub ollama_url: String,
    pub model: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ollama_url: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            database: DatabaseConfig {
                url: "postgres://postgres:postgres@localhost:5432/fitness_assistant".to_string(),
                max_connections: 10,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
            },
            jwt: JwtConfig {
                secret: "development-secret-change-in-production".to_string(),
                access_token_expiry_secs: 3600,      // 1 hour
                refresh_token_expiry_secs: 604800,   // 7 days
            },
            ai: AiConfig::default(),
        }
    }
}

impl AppConfig {
    /// Load configuration from files and environment
    ///
    /// Loading order (later sources override earlier):
    /// 1. Default values
    /// 2. Config file based on RUST_ENV (development.toml or production.toml)
    /// 3. Environment variables with FA__ prefix
    pub fn load() -> Result<Self> {
        let env = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
        let config_file = format!("config/{}.toml", env);

        let config = config::Config::builder()
            // Start with defaults
            .add_source(config::Config::try_from(&AppConfig::default())?)
            // Load from environment-specific config file
            .add_source(
                config::File::with_name(&config_file)
                    .required(false)
            )
            // Override with environment variables (FA__ prefix)
            // e.g., FA__SERVER__PORT=9000 sets server.port
            .add_source(
                config::Environment::with_prefix("FA")
                    .separator("__")
            )
            .build()?;

        Ok(config.try_deserialize()?)
    }

    /// Check if running in production mode
    pub fn is_production() -> bool {
        env::var("RUST_ENV")
            .map(|v| v == "production")
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
        assert!(!config.ai.enabled);
    }

    #[test]
    fn test_is_production() {
        // Default should be false (development)
        assert!(!AppConfig::is_production());
    }
}
