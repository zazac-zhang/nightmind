// ============================================================
// Data Models
// ============================================================
//! Database entity models.
//!
//! This module contains the data structures that represent
//! database entities.

pub mod user;
pub mod session;

// User-related types
pub use user::{User, UserRole, CreateUser, UpdateUser, LoginCredentials, UserProfile};

// Session-related types
pub use session::{Session, SessionState, SessionSummary, CreateSession, UpdateSession};
