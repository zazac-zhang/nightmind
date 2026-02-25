// ============================================================
// Database Migration Module
// ============================================================
//! Database migration management.
//!
//! This module provides utilities for running and managing database migrations.

use sqlx::{PgPool, Executor, Row};
use std::path::Path;
use tracing::{info, warn, error};

use crate::error::{NightMindError, Result};

/// Migration manager for handling database schema changes
pub struct MigrationManager {
    pool: PgPool,
}

impl MigrationManager {
    /// Creates a new migration manager
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Runs all pending migrations
    ///
    /// # Errors
    ///
    /// Returns an error if migration fails
    pub async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations...");

        // Create migrations table if it doesn't exist
        self.create_migrations_table().await?;

        // Get list of migration files
        let migrations = self.get_migration_files()?;

        // Get already applied migrations
        let applied = self.get_applied_migrations().await?;

        // Run pending migrations
        for migration in &migrations {
            if !applied.contains(&migration.version) {
                info!("Applying migration: {}", migration.version);
                self.run_migration(migration).await?;
            }
        }

        info!("All migrations completed successfully");
        Ok(())
    }

    /// Creates the migrations tracking table
    ///
    /// # Errors
    ///
    /// Returns an error if table creation fails
    async fn create_migrations_table(&self) -> Result<()> {
        let query = r#"
            CREATE TABLE IF NOT EXISTS _schema_migrations (
                version VARCHAR(14) PRIMARY KEY,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
        "#;

        self.pool.execute(query)
            .await
            .map_err(|e| NightMindError::Internal(format!("Failed to create migrations table: {}", e)))?;

        Ok(())
    }

    /// Gets the list of migration files from the migrations directory
    ///
    /// # Errors
    ///
    /// Returns an error if reading migrations fails
    fn get_migration_files(&self) -> Result<Vec<MigrationFile>> {
        let migrations_dir = std::env::var("MIGRATIONS_DIR")
            .unwrap_or_else(|_| "./migrations".to_string());

        let dir = Path::new(&migrations_dir);
        if !dir.exists() {
            warn!("Migrations directory not found: {}", migrations_dir);
            return Ok(Vec::new());
        }

        let mut migrations = Vec::new();

        for entry in std::fs::read_dir(dir)
            .map_err(|e| NightMindError::Internal(format!("Failed to read migrations directory: {}", e)))?
        {
            let entry = entry.map_err(|e| NightMindError::Internal(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            // Only process .up.sql files
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".up.sql") {
                    // Extract version number from filename (e.g., "001_initial.up.sql" -> "001")
                    if let Some(version) = name.split('_').next() {
                        migrations.push(MigrationFile {
                            version: version.to_string(),
                            name: name.to_string(),
                            path: path.clone(),
                        });
                    }
                }
            }
        }

        // Sort by version
        migrations.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(migrations)
    }

    /// Gets the list of already applied migrations
    ///
    /// # Errors
    ///
    /// Returns an error if querying fails
    async fn get_applied_migrations(&self) -> Result<Vec<String>> {
        let query = r#"
            SELECT version FROM _schema_migrations ORDER BY version;
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| NightMindError::Internal(format!("Failed to query applied migrations: {}", e)))?;

        let versions = rows.iter()
            .filter_map(|row| row.try_get(0).ok())
            .collect();

        Ok(versions)
    }

    /// Runs a single migration
    ///
    /// # Errors
    ///
    /// Returns an error if migration fails
    async fn run_migration(&self, migration: &MigrationFile) -> Result<()> {
        // Read migration SQL file
        let sql = std::fs::read_to_string(&migration.path)
            .map_err(|e| NightMindError::Internal(format!("Failed to read migration file {}: {}", migration.name, e)))?;

        // Execute migration in a transaction
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| NightMindError::Internal(format!("Failed to begin transaction: {}", e)))?;

        // Execute migration SQL
        tx.execute(sql.as_str())
            .await
            .map_err(|e| NightMindError::Internal(format!("Failed to execute migration {}: {}", migration.version, e)))?;

        // Record migration
        let record_query = r#"
            INSERT INTO _schema_migrations (version) VALUES ($1);
        "#;

        sqlx::query(record_query)
            .bind(&migration.version)
            .execute(&mut *tx)
            .await
            .map_err(|e| NightMindError::Internal(format!("Failed to record migration {}: {}", migration.version, e)))?;

        // Commit transaction
        tx.commit()
            .await
            .map_err(|e| NightMindError::Internal(format!("Failed to commit migration transaction: {}", e)))?;

        info!("Migration {} applied successfully", migration.version);
        Ok(())
    }

    /// Rollback the last migration (if possible)
    ///
    /// # Errors
    ///
    /// Returns an error if rollback fails
    pub async fn rollback_last_migration(&self) -> Result<()> {
        let applied = self.get_applied_migrations().await?;

        if applied.is_empty() {
            warn!("No migrations to rollback");
            return Ok(());
        }

        let last_version = applied.last().ok_or_else(|| NightMindError::Internal("No migrations found".to_string()))?;

        // Find and run the down migration file
        let migrations_dir = std::env::var("MIGRATIONS_DIR")
            .unwrap_or_else(|_| "./migrations".to_string());

        let dir = Path::new(&migrations_dir);
        let down_file_name = format!("{}_initial.down.sql", last_version);

        for entry in std::fs::read_dir(dir)
            .map_err(|e| NightMindError::Internal(format!("Failed to read migrations directory: {}", e)))?
        {
            let entry = entry.map_err(|e| NightMindError::Internal(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == down_file_name {
                    let sql = std::fs::read_to_string(&path)
                        .map_err(|e| NightMindError::Internal(format!("Failed to read rollback file {}: {}", name, e)))?;

                    let mut tx = self.pool.begin()
                        .await
                        .map_err(|e| NightMindError::Internal(format!("Failed to begin transaction: {}", e)))?;

                    tx.execute(sql.as_str())
                        .await
                        .map_err(|e| NightMindError::Internal(format!("Failed to execute rollback: {}", e)))?;

                    // Remove migration record
                    let delete_query = r#"
                        DELETE FROM _schema_migrations WHERE version = $1;
                    "#;

                    sqlx::query(delete_query)
                        .bind(last_version)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| NightMindError::Internal(format!("Failed to delete migration record: {}", e)))?;

                    tx.commit()
                        .await
                        .map_err(|e| NightMindError::Internal(format!("Failed to commit rollback transaction: {}", e)))?;

                    info!("Migration {} rolled back successfully", last_version);
                    return Ok(());
                }
            }
        }

        warn!("No rollback file found for migration {}", last_version);
        Ok(())
    }

    /// Gets the status of all migrations
    ///
    /// # Errors
    ///
    /// Returns an error if status check fails
    pub async fn get_migration_status(&self) -> Result<MigrationStatus> {
        let migrations = self.get_migration_files()?;
        let applied = self.get_applied_migrations().await?;

        let pending: Vec<_> = migrations.iter()
            .filter(|m| !applied.contains(&m.version))
            .map(|m| m.version.clone())
            .collect();

        let applied_count = applied.len();
        let pending_count = pending.len();

        Ok(MigrationStatus {
            applied,
            pending,
            applied_count,
            pending_count,
        })
    }
}

/// Represents a migration file
#[derive(Debug, Clone)]
struct MigrationFile {
    /// Migration version (e.g., "001")
    version: String,
    /// Migration file name
    name: String,
    /// Full path to the migration file
    path: std::path::PathBuf,
}

/// Status of all migrations
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    /// List of applied migration versions
    pub applied: Vec<String>,
    /// List of pending migration versions
    pub pending: Vec<String>,
    /// Number of applied migrations
    pub applied_count: usize,
    /// Number of pending migrations
    pub pending_count: usize,
}

/// Runs migrations using the given database pool
///
/// # Errors
///
/// Returns an error if migration fails
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    let manager = MigrationManager::new(pool.clone());
    manager.run_migrations().await
}

/// Gets migration status using the given database pool
///
/// # Errors
///
/// Returns an error if status check fails
pub async fn get_migration_status(pool: &PgPool) -> Result<MigrationStatus> {
    let manager = MigrationManager::new(pool.clone());
    manager.get_migration_status().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_status_creation() {
        let status = MigrationStatus {
            applied: vec!["001".to_string(), "002".to_string()],
            pending: vec!["003".to_string()],
            applied_count: 2,
            pending_count: 1,
        };

        assert_eq!(status.applied_count, 2);
        assert_eq!(status.pending_count, 1);
    }
}
