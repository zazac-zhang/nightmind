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

// Re-export common types
pub use db::{BaseRepository, DbPool, PoolStats, QueryBuilder, Repository};
pub use redis::{CacheEntry, CacheOps, RedisManager, SessionCache};
pub use user::{PgUserRepository, UserRepository};
pub use session::{PgSessionRepository, SessionRepository};
pub use models::{
    Session, SessionState, SessionSummary, User, UserRole,
    CreateUser, LoginCredentials, UpdateUser, CreateSession, UpdateSession,
};
