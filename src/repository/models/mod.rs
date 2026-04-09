// ============================================================
// Data Models
// ============================================================
//! Database entity models.
//!
//! This module contains the data structures that represent
//! database entities.

pub mod user;
pub mod session;
pub mod knowledge;
pub mod review_interval;

// User-related types
pub use user::{User, UserRole, CreateUser, UpdateUser, LoginCredentials, UserProfile};

// Session-related types
pub use session::{Session, SessionState, SessionSummary, CreateSession, UpdateSession};

// Knowledge-related types
pub use knowledge::{
    KnowledgePoint,
    KnowledgeContentType,
    KnowledgeSourceType,
    CreateKnowledgePoint,
    UpdateKnowledgePoint,
};

// Review interval types
pub use review_interval::{
    ReviewInterval,
    CreateReviewInterval,
    UpdateReviewInterval,
    SubmitReviewRequest,
    ReviewStats,
};
