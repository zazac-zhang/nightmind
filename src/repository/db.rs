// ============================================================
// Database Repository
// ============================================================
//! Database connection and query management.
//!
//! This module provides database connectivity, connection pooling,
//! and query execution utilities for persistent data storage.

use crate::config::Settings;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

/// Database connection pool type alias
pub type DbPool = Pool<Postgres>;

/// Connection pool statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolStats {
    /// Total connections in the pool
    pub total_connections: u32,
    /// Idle connections available
    pub idle_connections: u32,
    /// Active connections in use
    pub active_connections: u32,
}

/// Creates a database connection pool from application settings
///
/// This is the recommended way to create a connection pool in production.
/// It reads the database configuration from Settings and creates an
/// optimized connection pool.
///
/// # Arguments
///
/// * `settings` - Application configuration
///
/// # Returns
///
/// A connection pool or an error
///
/// # Errors
///
/// Returns an error if:
/// - The database URL is invalid
/// - The connection cannot be established
/// - The pool configuration is invalid
///
/// # Examples
///
/// ```no_run
/// use nightmind::{config::Settings, repository::db::create_pool_from_settings};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let settings = Settings::load()?;
/// let pool = create_pool_from_settings(&settings).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_pool_from_settings(settings: &Settings) -> Result<DbPool, sqlx::Error> {
    let options = sqlx::postgres::PgPoolOptions::new()
        .max_connections(settings.database.max_connections)
        .min_connections(settings.database.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(settings.database.timeout))
        .test_before_acquire(true);

    options.connect(&settings.database.url).await
}

/// Creates a new database connection pool with custom configuration
///
/// # Arguments
///
/// * `database_url` - PostgreSQL connection URL
/// * `max_connections` - Maximum number of connections in the pool
/// * `min_connections` - Minimum number of connections to maintain
///
/// # Returns
///
/// A connection pool or an error
///
/// # Errors
///
/// Returns an error if the connection pool cannot be created
pub async fn create_pool(
    database_url: &str,
    max_connections: u32,
    min_connections: u32,
) -> Result<DbPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(database_url)
        .await
}

/// Database repository trait
///
/// This trait defines the common interface for all database repositories.
#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    /// Returns the database pool
    fn pool(&self) -> &DbPool;

    /// Checks database connectivity with a simple query
    ///
    /// # Errors
    ///
    /// Returns an error if the database is not accessible
    async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .fetch_one(self.pool())
            .await?;
        Ok(())
    }

    /// Gets connection pool statistics
    ///
    /// # Returns
    ///
    /// Statistics about the connection pool
    fn pool_stats(&self) -> PoolStats {
        let pool = self.pool();
        let total = pool.size();
        let idle = pool.num_idle() as u32;
        PoolStats {
            total_connections: total,
            idle_connections: idle,
            active_connections: total.saturating_sub(idle),
        }
    }

    /// Runs pending database migrations
    ///
    /// # Errors
    ///
    /// Returns an error if migrations fail
    async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!().run(self.pool()).await
    }
}

/// Base repository implementation
///
/// Provides common functionality for all repositories.
pub struct BaseRepository {
    /// Database connection pool
    pool: Arc<DbPool>,
}

impl BaseRepository {
    /// Creates a new base repository
    #[must_use]
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Creates a new base repository from a pool reference
    #[must_use]
    pub fn from_pool(pool: &DbPool) -> Self {
        Self {
            pool: Arc::new(pool.clone()),
        }
    }

    /// Returns a reference to the pool
    #[must_use]
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    /// Begins a new transaction
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be started
    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, Postgres>, sqlx::Error> {
        self.pool.begin().await
    }
}

#[async_trait::async_trait]
impl Repository for BaseRepository {
    fn pool(&self) -> &DbPool {
        &self.pool
    }
}

/// Transaction guard for executing operations within a transaction
///
/// This guard ensures that transactions are properly committed or rolled back.
/// If the guard is dropped without an explicit commit, the transaction will
/// need to be handled (async drop is not supported in Rust).
pub struct TransactionGuard<'a> {
    /// The active transaction
    tx: Option<sqlx::Transaction<'a, Postgres>>,
    /// Whether the transaction is committed
    committed: bool,
}

impl<'a> TransactionGuard<'a> {
    /// Creates a new transaction guard
    #[must_use]
    pub fn new(tx: sqlx::Transaction<'a, Postgres>) -> Self {
        Self {
            tx: Some(tx),
            committed: false,
        }
    }

    /// Returns a reference to the transaction
    #[must_use]
    pub fn tx(&mut self) -> &mut sqlx::Transaction<'a, Postgres> {
        self.tx.as_mut().expect("Transaction already consumed")
    }

    /// Commits the transaction
    ///
    /// # Errors
    ///
    /// Returns an error if commit fails
    pub async fn commit(mut self) -> Result<(), sqlx::Error> {
        if let Some(tx) = self.tx.take() {
            tx.commit().await?;
            self.committed = true;
        }
        Ok(())
    }

    /// Rolls back the transaction
    pub async fn rollback(mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.rollback().await;
        }
    }
}

/// Query builder for common database operations
///
/// Provides utility methods for building common SQL queries.
pub struct QueryBuilder;

impl QueryBuilder {
    /// Creates a select by ID query
    #[must_use]
    pub fn select_by_id(table: &str) -> String {
        format!("SELECT * FROM {table} WHERE id = $1")
    }

    /// Creates a select query with a custom where clause
    #[must_use]
    pub fn select_where(table: &str, where_clause: &str) -> String {
        format!("SELECT * FROM {table} WHERE {where_clause}")
    }

    /// Creates an insert query
    #[must_use]
    pub fn insert(table: &str, columns: &[&str]) -> String {
        let placeholders: Vec<String> = (1..=columns.len())
            .map(|i| format!("${i}"))
            .collect();
        format!(
            "INSERT INTO {table} ({}) VALUES ({}) RETURNING *",
            columns.join(", "),
            placeholders.join(", ")
        )
    }

    /// Creates an update query
    #[must_use]
    pub fn update(table: &str, columns: &[&str]) -> String {
        let set_clause: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(i, col)| format!("{col} = ${}", i + 2))
            .collect();
        format!("UPDATE {table} SET {} WHERE id = $1 RETURNING *", set_clause.join(", "))
    }

    /// Creates a delete query
    #[must_use]
    pub fn delete(table: &str) -> String {
        format!("DELETE FROM {table} WHERE id = $1 RETURNING *")
    }

    /// Creates a count query
    #[must_use]
    pub fn count(table: &str) -> String {
        format!("SELECT COUNT(*) FROM {table}")
    }
}

/// Utility functions for common database operations
pub struct DbUtils;

impl DbUtils {
    /// Checks if a record exists by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    pub async fn exists_by_id(
        pool: &DbPool,
        table: &str,
        id: &str,
    ) -> Result<bool, sqlx::Error> {
        let query = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id = $1)");
        let (exists,): (bool,) = sqlx::query_as(&query).bind(id).fetch_one(pool).await?;
        Ok(exists)
    }

    /// Deletes a record by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the deletion fails
    pub async fn delete_by_id(
        pool: &DbPool,
        table: &str,
        id: &str,
    ) -> Result<u64, sqlx::Error> {
        let query = format!("DELETE FROM {table} WHERE id = $1");
        let result = sqlx::query(&query).bind(id).execute(pool).await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let insert = QueryBuilder::insert("users", &["name", "email"]);
        assert!(insert.contains("INSERT INTO users"));
        assert!(insert.contains("name, email"));
        assert!(insert.contains("$1, $2"));

        let update = QueryBuilder::update("users", &["name", "email"]);
        assert!(update.contains("UPDATE users SET"));
        assert!(update.contains("name = $2"));
        assert!(update.contains("email = $3"));

        let delete = QueryBuilder::delete("users");
        assert!(delete.contains("DELETE FROM users WHERE id = $1"));

        let count = QueryBuilder::count("users");
        assert_eq!(count, "SELECT COUNT(*) FROM users");
    }

    #[test]
    fn test_select_by_id() {
        let query = QueryBuilder::select_by_id("sessions");
        assert_eq!(query, "SELECT * FROM sessions WHERE id = $1");
    }

    #[test]
    fn test_select_where() {
        let query = QueryBuilder::select_where("users", "email = $1");
        assert_eq!(query, "SELECT * FROM users WHERE email = $1");
    }

    #[test]
    fn test_pool_stats_display() {
        let stats = PoolStats {
            total_connections: 10,
            idle_connections: 5,
            active_connections: 5,
        };
        // Test that the struct can be created and displayed
        assert_eq!(stats.total_connections, 10);
        assert_eq!(stats.active_connections, 5);
    }
}
