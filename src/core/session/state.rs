// ============================================================
// Session State
// ============================================================
//! Session state definitions and transitions.
//!
//! This module defines the possible states of a session and
/// manages state transitions.

use serde::{Deserialize, Serialize};

/// Session state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is newly created and active
    Active,
    /// Session is paused (user stepped away)
    Paused,
    /// Session is being archived
    Archiving,
    /// Session has ended normally
    Completed,
    /// Session ended due to error
    Failed,
    /// Session timed out
    Timeout,
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Paused => write!(f, "paused"),
            Self::Archiving => write!(f, "archiving"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Timeout => write!(f, "timeout"),
        }
    }
}

impl SessionState {
    /// Returns whether the session is in an active state
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns whether the session is in a terminal state
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Timeout)
    }

    /// Returns whether the session can receive new messages
    #[must_use]
    pub const fn can_receive_messages(&self) -> bool {
        matches!(self, Self::Active | Self::Paused)
    }

    /// Returns the display name of the state
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Paused => "Paused",
            Self::Archiving => "Archiving",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Timeout => "Timeout",
        }
    }
}

/// Session transition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTransition {
    /// Previous state
    pub from: SessionState,
    /// New state
    pub to: SessionState,
    /// Timestamp of transition
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional reason for transition
    pub reason: Option<String>,
}

impl SessionTransition {
    /// Creates a new session transition
    #[must_use]
    pub fn new(from: SessionState, to: SessionState) -> Self {
        Self {
            from,
            to,
            timestamp: chrono::Utc::now(),
            reason: None,
        }
    }

    /// Creates a transition with a reason
    #[must_use]
    pub fn with_reason(
        from: SessionState,
        to: SessionState,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            from,
            to,
            timestamp: chrono::Utc::now(),
            reason: Some(reason.into()),
        }
    }
}

/// Validates whether a state transition is allowed
#[must_use]
pub fn is_valid_transition(from: SessionState, to: SessionState) -> bool {
    match (from, to) {
        // Active can go to any state except back to itself
        (SessionState::Active, SessionState::Paused) => true,
        (SessionState::Active, SessionState::Archiving) => true,
        (SessionState::Active, SessionState::Completed) => true,
        (SessionState::Active, SessionState::Failed) => true,
        (SessionState::Active, SessionState::Timeout) => true,

        // Paused can resume or end
        (SessionState::Paused, SessionState::Active) => true,
        (SessionState::Paused, SessionState::Archiving) => true,
        (SessionState::Paused, SessionState::Completed) => true,
        (SessionState::Paused, SessionState::Timeout) => true,

        // Archiving can complete
        (SessionState::Archiving, SessionState::Completed) => true,
        (SessionState::Archiving, SessionState::Failed) => true,

        // Terminal states cannot transition
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_display() {
        assert_eq!(SessionState::Active.display_name(), "Active");
        assert_eq!(SessionState::Paused.display_name(), "Paused");
    }

    #[test]
    fn test_active_state() {
        assert!(SessionState::Active.is_active());
        assert!(!SessionState::Completed.is_active());
    }

    #[test]
    fn test_terminal_state() {
        assert!(SessionState::Completed.is_terminal());
        assert!(SessionState::Failed.is_terminal());
        assert!(!SessionState::Active.is_terminal());
    }

    #[test]
    fn test_valid_transitions() {
        assert!(is_valid_transition(SessionState::Active, SessionState::Paused));
        assert!(is_valid_transition(SessionState::Paused, SessionState::Active));
        assert!(!is_valid_transition(SessionState::Completed, SessionState::Active));
    }
}
