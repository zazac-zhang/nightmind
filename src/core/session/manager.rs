// ============================================================
// Session Manager
// ============================================================
//! Session lifecycle management.
//!
//! This module handles the creation, tracking, and cleanup of
//! user sessions.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::SessionState;

/// Active session data
#[derive(Debug, Clone)]
pub struct ActiveSession {
    /// Unique session identifier
    pub id: Uuid,
    /// Associated user ID
    pub user_id: Uuid,
    /// Current session state
    pub state: SessionState,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
}

/// Session manager for tracking active sessions
pub struct SessionManager {
    /// Active sessions by session ID
    sessions: Arc<RwLock<HashMap<Uuid, ActiveSession>>>,
    /// Sessions by user ID
    user_sessions: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    /// Session timeout in seconds
    session_timeout: u64,
}

impl SessionManager {
    /// Creates a new session manager
    ///
    /// # Arguments
    ///
    /// * `session_timeout` - Session timeout in seconds
    #[must_use]
    pub fn new(session_timeout: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout,
        }
    }

    /// Creates a new session for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID to create session for
    ///
    /// # Returns
    ///
    /// The new session ID
    pub async fn create_session(&self, user_id: Uuid) -> Uuid {
        let session_id = Uuid::new_v4();
        let now = Utc::now();

        let session = ActiveSession {
            id: session_id,
            user_id,
            state: SessionState::Warmup,  // Initial state for new sessions
            started_at: now,
            last_activity: now,
        };

        // Add to sessions map
        self.sessions.write().await.insert(session_id, session);

        // Add to user sessions
        {
            let mut user_sessions = self.user_sessions.write().await;
            user_sessions
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(session_id);
        }

        session_id
    }

    /// Gets a session by ID
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to look up
    ///
    /// # Returns
    ///
    /// The session if found and active
    pub async fn get_session(&self, session_id: Uuid) -> Option<ActiveSession> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).cloned()
    }

    /// Updates session activity timestamp
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session to update
    pub async fn update_activity(&self, session_id: Uuid) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.last_activity = Utc::now();
        }
    }

    /// Ends a session
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session to end
    ///
    /// # Returns
    ///
    /// True if the session was found and removed
    pub async fn end_session(&self, session_id: Uuid) -> bool {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.remove(&session_id) {
            // Remove from user sessions
            let mut user_sessions = self.user_sessions.write().await;
            if let Some(user_session_list) = user_sessions.get_mut(&session.user_id) {
                user_session_list.retain(|id| *id != session_id);
            }
            true
        } else {
            false
        }
    }

    /// Gets all active sessions for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID to look up
    ///
    /// # Returns
    ///
    /// Vector of active session IDs
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Vec<Uuid> {
        let user_sessions = self.user_sessions.read().await;
        user_sessions
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Cleans up expired sessions
    ///
    /// # Returns
    ///
    /// Number of sessions cleaned up
    pub async fn cleanup_expired(&self) -> usize {
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(self.session_timeout as i64);

        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;

        let expired: Vec<_> = sessions
            .iter()
            .filter(|(_, s)| now.signed_duration_since(s.last_activity) > timeout)
            .map(|(id, s)| (*id, s.user_id))
            .collect();

        let count = expired.len();

        for (session_id, user_id) in expired {
            sessions.remove(&session_id);
            if let Some(user_session_list) = user_sessions.get_mut(&user_id) {
                user_session_list.retain(|id| *id != session_id);
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let manager = SessionManager::new(3600);
        let user_id = Uuid::new_v4();

        let session_id = manager.create_session(user_id).await;

        let session = manager.get_session(session_id).await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().user_id, user_id);
    }

    #[tokio::test]
    async fn test_session_ending() {
        let manager = SessionManager::new(3600);
        let user_id = Uuid::new_v4();

        let session_id = manager.create_session(user_id).await;
        let ended = manager.end_session(session_id).await;

        assert!(ended);
        assert!(manager.get_session(session_id).await.is_none());
    }
}
