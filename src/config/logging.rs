// ============================================================
// Logging Configuration
// ============================================================
//! Logging initialization and configuration.
//!
//! This module sets up structured logging for the application
//! with support for console output, file logging, and custom formats.

use super::Settings;
use std::path::Path;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

/// Log guard that must be kept alive for logging to work
///
/// When dropped, logging will stop. This should be stored in the
/// main function or a long-lived structure.
pub struct LogGuard {
    /// Guard for non-blocking writers
    _guard: Option<non_blocking::WorkerGuard>,
}

/// Initialize logging from application settings
///
/// This is the recommended way to initialize logging in production.
/// It reads the configuration from Settings and sets up appropriate
/// log levels, formats, and outputs.
///
/// # Arguments
///
/// * `settings` - Application configuration
///
/// # Returns
///
/// A `LogGuard` that must be kept alive for the duration of the program.
///
/// # Panics
///
/// Panics if the logger cannot be initialized.
///
/// # Examples
///
/// ```no_run
/// use nightmind::config::Settings;
/// use nightmind::config::logging::init_from_settings;
///
/// let settings = Settings::load().unwrap();
/// let _guard = init_from_settings(&settings);
/// ```
pub fn init_from_settings(settings: &Settings) -> LogGuard {
    let level = &settings.logging.level;
    let format_json = settings.logging.format == "json";
    let file_logging = settings.logging.file_logging;

    let guard = if file_logging {
        if let Some(ref log_dir) = settings.logging.directory {
            init_logging_with_files(level, format_json, log_dir)
        } else {
            init_logging(level, format_json)
        }
    } else {
        init_logging(level, format_json)
    };

    LogGuard { _guard: guard }
}

/// Initialize the logging system with console output
///
/// # Arguments
///
/// * `level` - Log level (trace, debug, info, warn, error)
/// * `format_json` - Whether to use JSON format
///
/// # Panics
///
/// Panics if the logger cannot be initialized (rare).
///
/// # Examples
///
/// ```
/// use nightmind::config::logging::init_logging;
///
/// // Initialize with info level and pretty format
/// init_logging("info", false);
/// ```
pub fn init_logging(level: &str, format_json: bool) -> Option<non_blocking::WorkerGuard> {
    let env_filter = create_env_filter(level);

    let registry = Registry::default().with(env_filter);

    if format_json {
        registry
            .with(fmt::layer().json().with_filter(
                tracing_subscriber::filter::filter_fn(|metadata| {
                    !metadata.target().starts_with("hyper")
                        && !metadata.target().starts_with("tokio")
                        && !metadata.target().starts_with("h2")
                },
            ))
            )
            .init();
    } else {
        registry
            .with(
                fmt::layer()
                    .pretty()
                    .with_target(true)
                    .with_line_number(true)
                    .with_filter(
                        tracing_subscriber::filter::filter_fn(|metadata| {
                            !metadata.target().starts_with("hyper")
                                && !metadata.target().starts_with("tokio")
                                && !metadata.target().starts_with("h2")
                        },
                    ),
                )
                .boxed(),
            )
            .init();
    }

    None
}

/// Initialize logging with both console and file output
///
/// # Arguments
///
/// * `level` - Log level
/// * `format_json` - Whether to use JSON format
/// * `log_dir` - Directory for log files
///
/// # Returns
///
/// A `WorkerGuard` that must be kept alive for file logging to work.
///
/// # Panics
///
/// Panics if log files cannot be created or the logger cannot be initialized.
///
/// # Examples
///
/// ```no_run
/// use nightmind::config::logging::init_logging_with_files;
/// use std::path::Path;
///
/// let guard = init_logging_with_files("debug", false, Path::new("./logs"));
/// // Keep guard alive
/// ```
pub fn init_logging_with_files(
    level: &str,
    format_json: bool,
    log_dir: &Path,
) -> Option<non_blocking::WorkerGuard> {
    // Ensure log directory exists
    std::fs::create_dir_all(log_dir).expect("Failed to create log directory");

    let env_filter = create_env_filter(level);

    // File appender with daily rotation
    let file_appender = rolling::daily(log_dir, "nightmind");
    let (non_blocking_file, guard) = non_blocking(file_appender);

    let registry = Registry::default().with(env_filter);

    if format_json {
        // JSON format for both console and file
        registry
            .with(
                fmt::layer()
                    .json()
                    .with_writer(std::io::stdout)
                    .with_filter(
                        tracing_subscriber::filter::filter_fn(|metadata| {
                            !metadata.target().starts_with("hyper")
                                && !metadata.target().starts_with("tokio")
                                && !metadata.target().starts_with("h2")
                        },
                    ),
                )
                .boxed(),
            )
            .with(
                fmt::layer()
                    .json()
                    .with_writer(non_blocking_file)
                    .with_ansi(false),
            )
            .init();
    } else {
        // Pretty format for console, plain for file
        registry
            .with(
                fmt::layer()
                    .pretty()
                    .with_target(true)
                    .with_line_number(true)
                    .with_writer(std::io::stdout)
                    .with_filter(
                        tracing_subscriber::filter::filter_fn(|metadata| {
                            !metadata.target().starts_with("hyper")
                                && !metadata.target().starts_with("tokio")
                                && !metadata.target().starts_with("h2")
                        },
                    ),
                )
                .boxed(),
            )
            .with(
                fmt::layer()
                    .with_writer(non_blocking_file)
                    .with_ansi(false)
                    .with_target(true)
                    .with_thread_ids(false),
            )
            .init();
    }

    Some(guard)
}

/// Create an environment filter for log level
///
/// First tries to read from RUST_LOG environment variable,
/// otherwise uses the provided default level.
fn create_env_filter(default_level: &str) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Parse default level and add some default directives
        let mut filter = EnvFilter::new(default_level);

        // Set info level for common crates
        filter = filter
            .add_directive("axum=info".parse().unwrap())
            .add_directive("hyper=info".parse().unwrap())
            .add_directive("tokio=info".parse().unwrap())
            .add_directive("sqlx=info".parse().unwrap())
            .add_directive("nightmind=debug".parse().unwrap());

        filter
    })
}

/// Get the current log level as a string
///
/// # Arguments
///
/// * `settings` - Application configuration
///
/// # Returns
///
/// The current log level
#[must_use]
pub fn get_log_level(settings: &Settings) -> &str {
    &settings.logging.level
}

/// Check if debug logging is enabled
///
/// # Arguments
///
/// * `settings` - Application configuration
///
/// # Returns
///
/// true if debug or trace logging is enabled
#[must_use]
pub fn is_debug_enabled(settings: &Settings) -> bool {
    matches!(
        settings.logging.level.to_lowercase().as_str(),
        "debug" | "trace"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging_pretty() {
        // Should not panic
        let _guard = init_logging("debug", false);
        tracing::info!("Test log message");
    }

    #[test]
    fn test_init_logging_json() {
        // Should not panic
        let _guard = init_logging("info", true);
        tracing::info!("Test JSON log message");
    }

    #[test]
    fn test_init_logging_with_temp_dir() {
        let temp_dir = std::env::temp_dir().join("nightmind_test_logs");
        let guard = init_logging_with_files("debug", false, &temp_dir);
        assert!(guard.is_some());

        tracing::info!("Test file log message");

        // Guard will be dropped here
    }

    #[test]
    fn test_debug_enabled() {
        let settings = crate::config::Settings::test_default();
        assert!(is_debug_enabled(&settings));

        let mut settings_prod = settings.clone();
        settings_prod.logging.level = "info".to_string();
        assert!(!is_debug_enabled(&settings_prod));
    }

    #[test]
    fn test_get_log_level() {
        let settings = crate::config::Settings::test_default();
        assert_eq!(get_log_level(&settings), "debug");
    }
}
