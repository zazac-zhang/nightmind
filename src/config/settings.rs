// ============================================================
// Configuration Settings
// ============================================================
//! Application configuration settings.
//!
//! This module defines the configuration structure for NightMind,
//! loaded from environment variables or config files.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Server configuration
    pub server: ServerConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Redis configuration
    pub redis: RedisConfig,
    /// AI/LLM configuration
    pub ai: AiConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Application-specific settings
    pub app: AppConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum request body size (in bytes)
    pub max_body_size: usize,
    /// WebSocket timeout (in seconds)
    pub websocket_timeout: u64,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Minimum number of connections
    pub min_connections: u32,
    /// Connection timeout (in seconds)
    pub timeout: u64,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL
    pub url: String,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Connection timeout (in seconds)
    pub timeout: u64,
    /// Key prefix for namespacing
    pub key_prefix: String,
}

/// AI/LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// API key for LLM provider
    pub api_key: String,
    /// Base URL for API requests
    pub base_url: String,
    /// Model identifier
    pub model: String,
    /// Maximum tokens per response
    pub max_tokens: u32,
    /// Temperature setting (0.0 - 1.0)
    pub temperature: f32,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json, pretty)
    pub format: String,
    /// Directory for log files
    pub directory: Option<PathBuf>,
    /// Whether to write logs to files
    pub file_logging: bool,
}

/// Application-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    pub name: String,
    /// Application environment (dev, staging, prod)
    pub environment: String,
    /// Session timeout (in seconds)
    pub session_timeout: u64,
    /// Maximum concurrent sessions per user
    pub max_sessions_per_user: u32,
}

/// Configuration validation error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Missing required configuration: {0}")]
    Missing(String),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
}

impl Settings {
    /// Load settings from configuration file and environment variables
    ///
    /// This method loads configuration in the following order (later overrides earlier):
    /// 1. config/default.toml (optional)
    /// 2. config/{environment}.toml (optional, based on APP_ENVIRONMENT)
    /// 3. config/local.toml (optional, for local development)
    /// 4. Environment variables with prefix NIGHTMIND_
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be read or parsed,
    /// or if required fields are missing.
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file if present (optional)
        let _ = dotenvy::dotenv();

        let mut settings = config::Config::builder();

        // Start with default configuration
        settings = settings
            .add_source(config::File::with_name("config/default").required(false));

        // Override with environment-specific config
        let env = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());
        settings = settings.add_source(
            config::File::with_name(&format!("config/{env}")).required(false),
        );

        // Override with local config (not in git)
        settings = settings
            .add_source(config::File::with_name("config/local").required(false));

        // Override with environment variables
        // Environment variables should be prefixed with NIGHTMIND_
        // Nested fields use double underscore, e.g., NIGHTMIND_DATABASE__URL
        settings = settings.add_source(
            config::Environment::with_prefix("NIGHTMIND")
                .prefix_separator("_")
                .separator("__")
                .try_parsing(true),
        );

        let settings: Settings = settings.build()?.try_deserialize()?;

        // Validate configuration
        settings.validate()?;

        Ok(settings)
    }

    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate server config
        if self.server.port == 0 {
            return Err(ConfigError::Invalid("Server port cannot be 0".to_string()));
        }

        // Validate database config
        if self.database.url.is_empty() {
            return Err(ConfigError::Missing("DATABASE__URL".to_string()));
        }
        if self.database.max_connections < self.database.min_connections {
            return Err(ConfigError::Invalid(
                "Database max_connections must be >= min_connections".to_string(),
            ));
        }

        // Validate AI config
        if self.ai.api_key.is_empty() {
            return Err(ConfigError::Missing("AI__API_KEY".to_string()));
        }
        if self.ai.model.is_empty() {
            return Err(ConfigError::Missing("AI__MODEL".to_string()));
        }
        if !(0.0..=2.0).contains(&self.ai.temperature) {
            return Err(ConfigError::Invalid(
                "AI temperature must be between 0.0 and 2.0".to_string(),
            ));
        }

        // Validate logging level
        match self.logging.level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {},
            _ => {
                return Err(ConfigError::Invalid(
                    format!("Invalid log level: {}", self.logging.level)
                ))
            }
        }

        Ok(())
    }

    /// Get the database URL for SQLx
    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// Get the Redis URL
    #[must_use]
    pub fn redis_url(&self) -> &str {
        &self.redis.url
    }

    /// Check if running in development mode
    #[must_use]
    pub fn is_dev(&self) -> bool {
        self.app.environment == "dev"
    }

    /// Check if running in production mode
    #[must_use]
    pub fn is_prod(&self) -> bool {
        self.app.environment == "prod"
    }

    /// Create default settings for testing
    #[must_use]
    pub fn test_default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                max_body_size: 10_485_760, // 10 MB
                websocket_timeout: 300,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/nightmind_test".to_string(),
                max_connections: 5,
                min_connections: 1,
                timeout: 30,
            },
            redis: RedisConfig {
                url: "redis://127.0.0.1".to_string(),
                max_connections: 10,
                timeout: 10,
                key_prefix: "nightmind:".to_string(),
            },
            ai: AiConfig {
                api_key: "test-key".to_string(),
                base_url: "https://api.example.com".to_string(),
                model: "test-model".to_string(),
                max_tokens: 2048,
                temperature: 0.7,
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                format: "pretty".to_string(),
                directory: None,
                file_logging: false,
            },
            app: AppConfig {
                name: "NightMind".to_string(),
                environment: "test".to_string(),
                session_timeout: 3600,
                max_sessions_per_user: 5,
            },
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::test_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.server.host, "127.0.0.1");
        assert_eq!(settings.server.port, 3000);
    }

    #[test]
    fn test_is_dev() {
        let mut settings = Settings::default();
        settings.app.environment = "dev".to_string();
        assert!(settings.is_dev());
        assert!(!settings.is_prod());
    }

    #[test]
    fn test_is_prod() {
        let mut settings = Settings::default();
        settings.app.environment = "prod".to_string();
        assert!(settings.is_prod());
        assert!(!settings.is_dev());
    }

    #[test]
    fn test_database_url() {
        let settings = Settings::default();
        assert_eq!(settings.database_url(), "postgresql://localhost/nightmind_test");
    }

    #[test]
    fn test_validate_invalid_port() {
        let mut settings = Settings::default();
        settings.server.port = 0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_missing_database_url() {
        let mut settings = Settings::default();
        settings.database.url = String::new();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_temperature() {
        let mut settings = Settings::default();
        settings.ai.temperature = 3.0;
        assert!(settings.validate().is_err());
    }
}
