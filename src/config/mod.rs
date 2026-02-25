// ============================================================
// Configuration Module
// ============================================================
//! Application configuration management.
//!
//! This module provides configuration loading and validation
//! for the NightMind application.

pub mod logging;
pub mod settings;

// Re-export common types for convenience
pub use settings::{AiConfig, AppConfig, ConfigError, DatabaseConfig, LoggingConfig, RedisConfig, ServerConfig, Settings};
