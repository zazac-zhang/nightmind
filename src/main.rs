// ============================================================
// NightMind - 睡前认知巩固 AI 伴学 Agent
// ============================================================
//!
//! NightMind is an AI-powered learning companion designed for
//! bedtime memory consolidation and learning reinforcement.

#![warn(missing_docs)]
#![warn(clippy::all)]

use std::time::Duration;
use tokio::signal;
use tracing::{info, error};

use nightmind::config::Settings;
use nightmind::api::router;
use nightmind::error::{NightMindError, Result};

/// NightMind server instance
struct NightMindServer {
    settings: Settings,
    listener: tokio::net::TcpListener,
}

impl NightMindServer {
    /// Creates a new server instance
    ///
    /// # Errors
    ///
    /// Returns an error if server initialization fails
    async fn new() -> Result<Self> {
        // Load configuration
        let settings = Settings::load()
            .map_err(|e| NightMindError::Config(e.to_string()))?;

        info!("Loaded configuration from environment");

        // Bind to address
        let addr = format!("{}:{}", settings.server.host, settings.server.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| NightMindError::Internal(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("Server listening on {}", addr);

        Ok(Self {
            settings,
            listener,
        })
    }

    /// Runs the server
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start or run
    async fn run(self) -> Result<()> {
        info!("Starting NightMind v{}", env!("CARGO_PKG_VERSION"));

        // Build the application router
        let app = router::create_router(&self.settings)
            .map_err(|e| NightMindError::Internal(format!("Failed to create router: {}", e)))?;

        // Create the server
        let server = axum::serve(self.listener, app)
            .with_graceful_shutdown(shutdown_signal());

        info!("Server ready at http://{}:{}",
            self.settings.server.host,
            self.settings.server.port
        );

        // Run the server
        server.await
            .map_err(|e| NightMindError::Internal(format!("Server error: {}", e)))?;

        info!("Server shutdown complete");

        Ok(())
    }
}

/// Initializes the logging system
fn init_logging() {
    // Set default log level from environment or use "info"
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&log_level));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();
}

/// Waits for a shutdown signal (Ctrl+C or terminate)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            info!("Received terminate signal, shutting down...");
        },
    }
}

/// Validates that required services are available
///
/// # Errors
///
/// Returns an error if any required service is unavailable
async fn validate_services(settings: &Settings) -> Result<()> {
    info!("Validating required services...");

    // Check database connection
    let db_pool = sqlx::PgPool::connect_lazy(&settings.database.url)
        .map_err(|e| NightMindError::Internal(format!("Failed to create database pool: {}", e)))?;

    sqlx::query("SELECT 1")
        .fetch_one(&db_pool)
        .await
        .map_err(|e| NightMindError::Internal(format!("Database health check failed: {}", e)))?;

    info!("Database connection validated");

    // Run database migrations
    info!("Running database migrations...");
    nightmind::repository::run_migrations(&db_pool).await
        .map_err(|e| NightMindError::Internal(format!("Migration failed: {}", e)))?;
    info!("Database migrations completed");

    // Check Redis connection
    let redis_client = redis::Client::open(settings.redis.url.clone())
        .map_err(|e| NightMindError::Internal(format!("Failed to create Redis client: {}", e)))?;

    let mut redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| NightMindError::Internal(format!("Redis connection failed: {}", e)))?;

    redis::cmd("PING")
        .query_async::<String>(&mut redis_conn)
        .await
        .map_err(|e| NightMindError::Internal(format!("Redis PING failed: {}", e)))?;

    info!("Redis connection validated");

    Ok(())
}

/// Application entry point
///
/// # Errors
///
/// Returns an error if the application fails to start
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging first
    init_logging();

    info!("====================================================");
    info!("NightMind AI Agent v{}", env!("CARGO_PKG_VERSION"));
    info!("====================================================");

    // Load configuration
    let settings = Settings::load()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    // Validate required services
    if let Err(e) = validate_services(&settings).await {
        error!("Service validation failed: {}", e);
        error!("Please ensure all required services are running:");
        error!("  - PostgreSQL: {}", settings.database.url);
        error!("  - Redis: {}", settings.redis.url);
        return Err(e.into());
    }

    info!("All services validated successfully");

    // Create and run the server
    let server = NightMindServer::new().await?;
    server.run().await?;

    Ok(())
}
