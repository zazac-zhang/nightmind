// ============================================================
// Session Snapshot
// ============================================================
//! Session snapshot functionality for persistence.
//!
//! This module handles creating and restoring session snapshots.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::SessionState;

/// A snapshot of a session's state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Snapshot ID
    pub id: Uuid,
    /// Session ID this snapshot represents
    pub session_id: Uuid,
    /// User ID for the session
    pub user_id: Uuid,
    /// Session state at time of snapshot
    pub state: SessionState,
    /// Messages in the session
    pub messages: Vec<SessionMessage>,
    /// Snapshot creation timestamp
    pub created_at: DateTime<Utc>,
    /// Session metadata
    pub metadata: SessionMetadata,
}

/// A message in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    /// Message ID
    pub id: Uuid,
    /// Message role (user/assistant/system)
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
}

/// Message role enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    /// Message from the user
    User,
    /// Message from the AI assistant
    Assistant,
    /// System message
    System,
}

impl MessageRole {
    /// Returns the display name of the role
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
        }
    }
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session title (auto-generated or user-provided)
    pub title: String,
    /// Topics discussed in the session
    pub topics: Vec<String>,
    /// Message count
    pub message_count: usize,
    /// Session duration in seconds
    pub duration_seconds: Option<u64>,
    /// Additional custom metadata
    pub custom: std::collections::HashMap<String, String>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            topics: Vec::new(),
            message_count: 0,
            duration_seconds: None,
            custom: std::collections::HashMap::new(),
        }
    }
}

/// Snapshot manager for creating and restoring snapshots
pub struct SnapshotManager;

impl SnapshotManager {
    /// Creates a snapshot from session data
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to snapshot
    /// * `user_id` - User ID for the session
    /// * `state` - Current session state
    /// * `messages` - Messages in the session
    /// * `metadata` - Session metadata
    ///
    /// # Returns
    ///
    /// A new session snapshot
    #[must_use]
    pub fn create(
        session_id: Uuid,
        user_id: Uuid,
        state: SessionState,
        messages: Vec<SessionMessage>,
        metadata: SessionMetadata,
    ) -> SessionSnapshot {
        SessionSnapshot {
            id: Uuid::new_v4(),
            session_id,
            user_id,
            state,
            messages,
            created_at: Utc::now(),
            metadata,
        }
    }

    /// Serializes a snapshot to JSON
    ///
    /// # Arguments
    ///
    /// * `snapshot` - Snapshot to serialize
    ///
    /// # Returns
    ///
    /// JSON string representation
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    pub fn serialize(snapshot: &SessionSnapshot) -> Result<String, serde_json::Error> {
        serde_json::to_string(snapshot)
    }

    /// Deserializes a snapshot from JSON
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string to deserialize
    ///
    /// # Returns
    ///
    /// Deserialized snapshot
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    pub fn deserialize(json: &str) -> Result<SessionSnapshot, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let session_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let state = SessionState::Warmup;
        let messages = vec![];
        let metadata = SessionMetadata::default();

        let snapshot = SnapshotManager::create(
            session_id,
            user_id,
            state,
            messages,
            metadata,
        );

        assert_eq!(snapshot.session_id, session_id);
        assert_eq!(snapshot.user_id, user_id);
        assert_eq!(snapshot.state, SessionState::Active);
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = SessionSnapshot {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            state: SessionState::Active,
            messages: vec![],
            created_at: Utc::now(),
            metadata: SessionMetadata::default(),
        };

        let json = SnapshotManager::serialize(&snapshot);
        assert!(json.is_ok());

        let deserialized = SnapshotManager::deserialize(&json.unwrap());
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap().session_id, snapshot.session_id);
    }
}
