// ============================================================
// Session Management Module
// ============================================================
//! Session state management and lifecycle.
//!
//! This module provides session state machine, topic stack management,
//! and session lifecycle tracking.

pub mod manager;
pub mod snapshot;
pub mod state;
pub mod topic_stack;

// Re-export SessionState from repository models for convenience
pub use crate::repository::models::session::{
    Session, SessionState, SessionSummary,
    CreateSession, UpdateSession,
};
