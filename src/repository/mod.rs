// ============================================================
// Repository Module
// ============================================================
//! Data access layer.
//!
//! This module provides database operations and data models
//! for persistent storage.

pub mod db;
pub mod redis;
pub mod models;
pub mod user;
pub mod session;
pub mod knowledge;
pub mod review_interval;
pub mod migration;

// Re-export common types
pub use db::{BaseRepository, DbPool, PoolStats, QueryBuilder, Repository};
pub use redis::{CacheEntry, CacheOps, RedisManager, SessionCache};
pub use user::{PgUserRepository, UserRepository};
pub use session::{PgSessionRepository, SessionRepository};
pub use knowledge::{
    KnowledgeRepository, PostgresKnowledgeRepository,
    KnowledgeSearchQuery, RepositoryError as KnowledgeError,
};
pub use review_interval::{
    ReviewIntervalRepository, PostgresReviewIntervalRepository,
    RepositoryError as ReviewIntervalError,
};
pub use migration::{MigrationManager, MigrationStatus, run_migrations, get_migration_status};
pub use models::{
    Session, SessionState, SessionSummary, User, UserRole,
    CreateUser, LoginCredentials, UpdateUser, CreateSession, UpdateSession,
    KnowledgePoint, CreateKnowledgePoint, UpdateKnowledgePoint,
    ReviewInterval, CreateReviewInterval, UpdateReviewInterval,
    SubmitReviewRequest, ReviewStats,
};
