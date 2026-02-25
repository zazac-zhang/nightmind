// ============================================================
// Session Model
// ============================================================
//! Session entity and related structures.
//!
//! This module defines the Session model and state management
//! for tracking user sessions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Session state in the conversation flow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[repr(i32)]
pub enum SessionState {
    /// Initial warmup phase
    Warmup = 0,
    /// Deep dive into topic
    DeepDive = 1,
    /// Review and consolidation
    Review = 2,
    /// Planting seeds for next session
    Seed = 3,
    /// Session closing
    Closing = 4,
}

impl SessionState {
    /// All session states in order
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [Self::Warmup, Self::DeepDive, Self::Review, Self::Seed, Self::Closing]
    }

    /// Gets the next state in the flow
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Warmup => Some(Self::DeepDive),
            Self::DeepDive => Some(Self::Review),
            Self::Review => Some(Self::Seed),
            Self::Seed => Some(Self::Closing),
            Self::Closing => None,
        }
    }

    /// Returns the state name as a string
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warmup => "warmup",
            Self::DeepDive => "deep_dive",
            Self::Review => "review",
            Self::Seed => "seed",
            Self::Closing => "closing",
        }
    }
}

/// User learning session entity
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    /// Unique session ID
    pub id: Uuid,
    /// User who owns this session
    pub user_id: Uuid,
    /// Session title/topic
    pub title: String,
    /// Current session state
    pub state: SessionState,
    /// Topic stack for context (JSON serialized)
    pub topic_stack: Option<serde_json::Value>,
    /// Current cognitive load (0.0 - 1.0)
    pub cognitive_load: f32,
    /// Session start timestamp
    pub started_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity_at: DateTime<Utc>,
    /// Session end timestamp (null if active)
    pub ended_at: Option<DateTime<Utc>>,
    /// Session metadata (JSON)
    pub metadata: Option<serde_json::Value>,
}

/// Session creation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSession {
    /// User ID
    pub user_id: Uuid,
    /// Session title
    pub title: String,
    /// Optional initial state
    pub initial_state: Option<SessionState>,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Session update data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSession {
    /// Optional new title
    pub title: Option<String>,
    /// Optional new state
    pub state: Option<SessionState>,
    /// Optional new topic stack
    pub topic_stack: Option<serde_json::Value>,
    /// Optional new cognitive load
    pub cognitive_load: Option<f32>,
    /// Optional new metadata
    pub metadata: Option<serde_json::Value>,
}

/// Session summary for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session ID
    pub id: Uuid,
    /// Session title
    pub title: String,
    /// Current state
    pub state: SessionState,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// Whether the session is active
    pub is_active: bool,
    /// Duration in seconds (for active sessions)
    pub duration_seconds: Option<i64>,
}

impl Session {
    /// Creates a new session
    #[must_use]
    pub fn create(data: CreateSession) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            user_id: data.user_id,
            title: data.title,
            state: data.initial_state.unwrap_or(SessionState::Warmup),
            topic_stack: None,
            cognitive_load: 0.0,
            started_at: now,
            last_activity_at: now,
            ended_at: None,
            metadata: data.metadata,
        }
    }

    /// Updates the session with new data
    pub fn update(&mut self, data: UpdateSession) {
        if let Some(title) = data.title {
            self.title = title;
        }
        if let Some(state) = data.state {
            self.state = state;
        }
        if let Some(topic_stack) = data.topic_stack {
            self.topic_stack = Some(topic_stack);
        }
        if let Some(load) = data.cognitive_load {
            self.cognitive_load = load.clamp(0.0, 1.0);
        }
        if let Some(metadata) = data.metadata {
            self.metadata = Some(metadata);
        }
        self.last_activity_at = Utc::now();
    }

    /// Ends the session
    pub fn end(&mut self) {
        self.ended_at = Some(Utc::now());
        self.last_activity_at = Utc::now();
    }

    /// Checks if the session is active
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }

    /// Gets the session duration in seconds
    #[must_use]
    pub fn duration_seconds(&self) -> i64 {
        let end = self.ended_at.unwrap_or_else(Utc::now);
        end.signed_duration_since(self.started_at).num_seconds()
    }

    /// Advances to the next state
    ///
    /// # Returns
    ///
    /// The new state, or None if already at the end
    pub fn advance_state(&mut self) -> Option<SessionState> {
        if let Some(next_state) = self.state.next() {
            self.state = next_state;
            self.last_activity_at = Utc::now();
            Some(next_state)
        } else {
            None
        }
    }

    /// Gets a summary of the session
    #[must_use]
    pub fn to_summary(&self) -> SessionSummary {
        SessionSummary {
            id: self.id,
            title: self.title.clone(),
            state: self.state,
            started_at: self.started_at,
            is_active: self.is_active(),
            duration_seconds: if self.is_active() {
                Some(self.duration_seconds())
            } else {
                None
            },
        }
    }

    /// Updates the cognitive load level
    ///
    /// # Arguments
    ///
    /// * `load` - New cognitive load (will be clamped to 0.0 - 1.0)
    pub fn update_cognitive_load(&mut self, load: f32) {
        self.cognitive_load = load.clamp(0.0, 1.0);
        self.last_activity_at = Utc::now();
    }

    /// Checks if the session is idle (no activity for specified duration)
    ///
    /// # Arguments
    ///
    /// * `timeout_seconds` - Timeout in seconds
    #[must_use]
    pub fn is_idle(&self, timeout_seconds: i64) -> bool {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.last_activity_at).num_seconds();
        elapsed > timeout_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_values() {
        assert_eq!(SessionState::Warmup as i32, 0);
        assert_eq!(SessionState::DeepDive as i32, 1);
        assert_eq!(SessionState::Review as i32, 2);
        assert_eq!(SessionState::Seed as i32, 3);
        assert_eq!(SessionState::Closing as i32, 4);
    }

    #[test]
    fn test_session_state_next() {
        assert_eq!(SessionState::Warmup.next(), Some(SessionState::DeepDive));
        assert_eq!(SessionState::DeepDive.next(), Some(SessionState::Review));
        assert_eq!(SessionState::Review.next(), Some(SessionState::Seed));
        assert_eq!(SessionState::Seed.next(), Some(SessionState::Closing));
        assert_eq!(SessionState::Closing.next(), None);
    }

    #[test]
    fn test_session_state_as_str() {
        assert_eq!(SessionState::Warmup.as_str(), "warmup");
        assert_eq!(SessionState::DeepDive.as_str(), "deep_dive");
        assert_eq!(SessionState::Review.as_str(), "review");
        assert_eq!(SessionState::Seed.as_str(), "seed");
        assert_eq!(SessionState::Closing.as_str(), "closing");
    }

    #[test]
    fn test_create_session() {
        let data = CreateSession {
            user_id: Uuid::new_v4(),
            title: "Learning Rust".to_string(),
            initial_state: None,
            metadata: None,
        };

        let session = Session::create(data);
        assert!(!session.id.is_nil());
        assert_eq!(session.title, "Learning Rust");
        assert_eq!(session.state, SessionState::Warmup);
        assert!(session.is_active());
        assert!(session.topic_stack.is_none());
        assert_eq!(session.cognitive_load, 0.0);
    }

    #[test]
    fn test_update_session() {
        let mut session = Session::create(CreateSession {
            user_id: Uuid::new_v4(),
            title: "Test".to_string(),
            initial_state: None,
            metadata: None,
        });

        session.update(UpdateSession {
            title: Some("Updated Title".to_string()),
            state: Some(SessionState::DeepDive),
            cognitive_load: Some(0.75),
            topic_stack: None,
            metadata: None,
        });

        assert_eq!(session.title, "Updated Title");
        assert_eq!(session.state, SessionState::DeepDive);
        assert_eq!(session.cognitive_load, 0.75);
    }

    #[test]
    fn test_end_session() {
        let mut session = Session::create(CreateSession {
            user_id: Uuid::new_v4(),
            title: "Test".to_string(),
            initial_state: None,
            metadata: None,
        });

        assert!(session.is_active());
        session.end();
        assert!(!session.is_active());
        assert!(session.ended_at.is_some());
    }

    #[test]
    fn test_advance_state() {
        let mut session = Session::create(CreateSession {
            user_id: Uuid::new_v4(),
            title: "Test".to_string(),
            initial_state: Some(SessionState::Warmup),
            metadata: None,
        });

        assert_eq!(session.state, SessionState::Warmup);
        assert!(session.advance_state().is_some());
        assert_eq!(session.state, SessionState::DeepDive);

        // Advance through all states
        for _ in 0..4 {
            session.advance_state();
        }
        assert_eq!(session.state, SessionState::Closing);
        assert!(session.advance_state().is_none());
    }

    #[test]
    fn test_cognitive_load_clamping() {
        let mut session = Session::create(CreateSession {
            user_id: Uuid::new_v4(),
            title: "Test".to_string(),
            initial_state: None,
            metadata: None,
        });

        session.update_cognitive_load(1.5);
        assert_eq!(session.cognitive_load, 1.0);

        session.update_cognitive_load(-0.5);
        assert_eq!(session.cognitive_load, 0.0);
    }

    #[test]
    fn test_is_idle() {
        let mut session = Session::create(CreateSession {
            user_id: Uuid::new_v4(),
            title: "Test".to_string(),
            initial_state: None,
            metadata: None,
        });

        // Fresh session should not be idle
        assert!(!session.is_idle(3600));

        // Manually set old activity time
        session.last_activity_at = Utc::now() - chrono::Duration::seconds(4000);
        assert!(session.is_idle(3600));
    }

    #[test]
    fn test_to_summary() {
        let session = Session::create(CreateSession {
            user_id: Uuid::new_v4(),
            title: "Test Session".to_string(),
            initial_state: Some(SessionState::Review),
            metadata: None,
        });

        let summary = session.to_summary();
        assert_eq!(summary.id, session.id);
        assert_eq!(summary.title, "Test Session");
        assert_eq!(summary.state, SessionState::Review);
        assert!(summary.is_active);
        assert!(summary.duration_seconds.is_some());
    }
}
