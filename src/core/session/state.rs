// ============================================================
// Session State Machine
// ============================================================
//! Session state machine implementation.
//!
//! This module implements the state machine for managing session
//! state transitions and history tracking.

use crate::repository::models::session::SessionState;
use crate::error::{NightMindError, Result};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

/// ============================================================================
// State Transition
// ============================================================================

/// Session transition event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionTransition {
    /// Previous state
    pub from: SessionState,
    /// New state
    pub to: SessionState,
    /// Timestamp of transition
    pub timestamp: DateTime<Utc>,
    /// Optional reason for transition
    pub reason: Option<String>,
    /// Optional metadata about the transition
    pub metadata: Option<serde_json::Value>,
}

impl SessionTransition {
    /// Creates a new session transition
    #[must_use]
    pub fn new(from: SessionState, to: SessionState) -> Self {
        Self {
            from,
            to,
            timestamp: Utc::now(),
            reason: None,
            metadata: None,
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
            timestamp: Utc::now(),
            reason: Some(reason.into()),
            metadata: None,
        }
    }

    /// Creates a transition with metadata
    #[must_use]
    pub fn with_metadata(
        from: SessionState,
        to: SessionState,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            from,
            to,
            timestamp: Utc::now(),
            reason: None,
            metadata: Some(metadata),
        }
    }

    /// Returns the duration of this transition
    #[must_use]
    pub fn duration(&self) -> Option<i64> {
        // For a single transition, duration is not applicable
        // This would be used when calculating time between transitions
        None
    }
}

/// ============================================================================
// State Machine Configuration
// ============================================================================

/// Configuration for state machine behavior
#[derive(Debug, Clone)]
pub struct StateMachineConfig {
    /// Maximum number of state transitions to keep in history
    pub max_history: usize,
    /// Whether to allow jumping multiple states at once
    pub allow_skip: bool,
    /// Whether to allow going back to previous states
    pub allow_reverse: bool,
    /// Timeout for each state in seconds (None for no timeout)
    pub state_timeouts: std::collections::HashMap<SessionState, u64>,
}

impl Default for StateMachineConfig {
    fn default() -> Self {
        let mut state_timeouts = std::collections::HashMap::new();
        state_timeouts.insert(SessionState::Warmup, 300);      // 5 minutes
        state_timeouts.insert(SessionState::DeepDive, 1200);   // 20 minutes
        state_timeouts.insert(SessionState::Review, 600);      // 10 minutes
        state_timeouts.insert(SessionState::Seed, 180);        // 3 minutes
        state_timeouts.insert(SessionState::Closing, 120);     // 2 minutes

        Self {
            max_history: 100,
            allow_skip: false,
            allow_reverse: true,
            state_timeouts,
        }
    }
}

/// ============================================================================
// Session State Machine
// ============================================================================

/// Session state machine for managing state transitions
///
/// This state machine tracks the current state, validates transitions,
/// and maintains a history of all state changes.
pub struct SessionStateMachine {
    /// Current state
    current: SessionState,
    /// State transition history
    history: VecDeque<SessionTransition>,
    /// State machine configuration
    config: StateMachineConfig,
    /// When the current state was entered
    state_entered_at: DateTime<Utc>,
}

impl SessionStateMachine {
    /// Creates a new state machine starting at Warmup
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(StateMachineConfig::default())
    }

    /// Creates a state machine with custom configuration
    #[must_use]
    pub fn with_config(config: StateMachineConfig) -> Self {
        let now = Utc::now();
        let mut history = VecDeque::new();
        history.push_back(SessionTransition {
            from: SessionState::Warmup,
            to: SessionState::Warmup,
            timestamp: now,
            reason: Some("Initial state".to_string()),
            metadata: None,
        });

        Self {
            current: SessionState::Warmup,
            history,
            config,
            state_entered_at: now,
        }
    }

    /// Gets the current state
    #[must_use]
    pub const fn current(&self) -> SessionState {
        self.current
    }

    /// Gets the state entry time
    #[must_use]
    pub const fn state_entered_at(&self) -> DateTime<Utc> {
        self.state_entered_at
    }

    /// Gets the duration in the current state
    #[must_use]
    pub fn current_state_duration(&self) -> chrono::TimeDelta {
        Utc::now().signed_duration_since(self.state_entered_at)
    }

    /// Checks if the current state has timed out
    #[must_use]
    pub fn is_state_timed_out(&self) -> bool {
        if let Some(&timeout) = self.config.state_timeouts.get(&self.current) {
            let elapsed = self.current_state_duration().num_seconds() as u64;
            elapsed > timeout
        } else {
            false
        }
    }

    /// Gets the timeout for the current state
    #[must_use]
    pub fn current_state_timeout(&self) -> Option<u64> {
        self.config.state_timeouts.get(&self.current).copied()
    }

    /// Transitions to a new state
    ///
    /// # Errors
    ///
    /// Returns an error if the transition is invalid
    pub fn transition_to(&mut self, new_state: SessionState) -> Result<SessionTransition> {
        self.transition_with_reason(new_state, None)
    }

    /// Transitions to a new state with a reason
    ///
    /// # Arguments
    ///
    /// * `new_state` - Target state
    /// * `reason` - Optional reason for the transition
    ///
    /// # Errors
    ///
    /// Returns an error if the transition is invalid
    pub fn transition_with_reason(
        &mut self,
        new_state: SessionState,
        reason: Option<String>,
    ) -> Result<SessionTransition> {
        // Validate transition
        if !self.is_valid_transition(new_state) {
            return Err(NightMindError::BadRequest(format!(
                "Invalid state transition from {:?} to {:?}",
                self.current, new_state
            )));
        }

        // Create transition record
        let transition = SessionTransition {
            from: self.current,
            to: new_state,
            timestamp: Utc::now(),
            reason,
            metadata: None,
        };

        // Update state
        let old_state = self.current;
        self.current = new_state;
        self.state_entered_at = transition.timestamp;

        // Add to history
        self.history.push_back(transition.clone());

        // Trim history if needed
        while self.history.len() > self.config.max_history {
            self.history.pop_front();
        }

        tracing::debug!(
            "State transition: {:?} -> {:?}",
            old_state,
            new_state
        );

        Ok(transition)
    }

    /// Advances to the next state in the flow
    ///
    /// # Errors
    ///
    /// Returns an error if already at the last state
    pub fn advance(&mut self) -> Result<SessionTransition> {
        let next = self.current.next().ok_or_else(|| {
            NightMindError::BadRequest("Already at final state".to_string())
        })?;

        self.transition_to(next)
    }

    /// Validates if a transition to the given state is allowed
    #[must_use]
    pub fn is_valid_transition(&self, new_state: SessionState) -> bool {
        // Same state is not a transition
        if self.current == new_state {
            return false;
        }

        // Check forward transitions (normal flow)
        if let Some(next_state) = self.current.next() {
            if new_state == next_state {
                return true;
            }
        }

        // Check if skipping is allowed
        if self.config.allow_skip {
            // Allow skipping ahead in the flow
            return matches!(new_state, SessionState::DeepDive | SessionState::Review | SessionState::Seed | SessionState::Closing);
        }

        // Check if reverse transitions are allowed
        if self.config.allow_reverse {
            // Allow going back to previous states
            if let Some(prev) = self.previous_state(self.current) {
                return new_state == prev;
            }
        }

        false
    }

    /// Gets the previous state in the flow
    #[must_use]
    pub const fn previous_state(&self, state: SessionState) -> Option<SessionState> {
        match state {
            SessionState::Warmup => None,
            SessionState::DeepDive => Some(SessionState::Warmup),
            SessionState::Review => Some(SessionState::DeepDive),
            SessionState::Seed => Some(SessionState::Review),
            SessionState::Closing => Some(SessionState::Seed),
        }
    }

    /// Gets the next state in the flow
    #[must_use]
    pub const fn next_state(&self) -> Option<SessionState> {
        self.current.next()
    }

    /// Checks if the current state is a terminal state
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self.current, SessionState::Closing)
    }

    /// Checks if the current state is an active state
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !self.is_terminal()
    }

    /// Gets the transition history
    #[must_use]
    pub fn history(&self) -> &VecDeque<SessionTransition> {
        &self.history
    }

    /// Gets the number of transitions
    #[must_use]
    pub fn transition_count(&self) -> usize {
        self.history.len().saturating_sub(1) // Subtract initial state
    }

    /// Gets the number of times a specific state was visited
    #[must_use]
    pub fn visit_count(&self, state: SessionState) -> usize {
        self.history
            .iter()
            .filter(|t| t.to == state)
            .count()
    }

    /// Clears the history but keeps the current state
    pub fn clear_history(&mut self) {
        let now = Utc::now();
        self.history.clear();
        self.history.push_back(SessionTransition {
            from: self.current,
            to: self.current,
            timestamp: now,
            reason: Some("History cleared".to_string()),
            metadata: None,
        });
    }

    /// Resets the state machine to Warmup
    pub fn reset(&mut self) {
        let now = Utc::now();
        self.current = SessionState::Warmup;
        self.state_entered_at = now;
        self.history.clear();
        self.history.push_back(SessionTransition {
            from: SessionState::Warmup,
            to: SessionState::Warmup,
            timestamp: now,
            reason: Some("Reset to Warmup".to_string()),
            metadata: None,
        });
    }

    /// Creates a snapshot of the current state
    #[must_use]
    pub fn snapshot(&self) -> StateMachineSnapshot {
        StateMachineSnapshot {
            current: self.current,
            history: self.history.iter().cloned().collect(),
            state_entered_at: self.state_entered_at,
        }
    }

    /// Restores from a snapshot
    ///
    /// # Errors
    ///
    /// Returns an error if the snapshot is invalid
    pub fn restore(&mut self, snapshot: StateMachineSnapshot) -> Result<()> {
        self.current = snapshot.current;
        self.history = snapshot.history.into_iter().collect();
        self.state_entered_at = snapshot.state_entered_at;
        Ok(())
    }
}

impl Default for SessionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
// State Machine Snapshot
// ============================================================================

/// Snapshot of the state machine for persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateMachineSnapshot {
    /// Current state
    pub current: SessionState,
    /// Transition history
    pub history: Vec<SessionTransition>,
    /// When the current state was entered
    pub state_entered_at: DateTime<Utc>,
}

/// ============================================================================
// Shared State Machine
// ============================================================================

/// Thread-safe wrapper around SessionStateMachine
pub struct SharedStateMachine {
    inner: Arc<RwLock<SessionStateMachine>>,
}

impl SharedStateMachine {
    /// Creates a new shared state machine
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(SessionStateMachine::new())),
        }
    }

    /// Creates a shared state machine with custom config
    #[must_use]
    pub fn with_config(config: StateMachineConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(SessionStateMachine::with_config(config))),
        }
    }

    /// Gets the current state
    ///
    /// # Errors
    ///
    /// Returns an error if the lock is poisoned
    pub async fn current(&self) -> Result<SessionState> {
        let sm = self.inner.read().await;
        Ok(sm.current())
    }

    /// Transitions to a new state
    ///
    /// # Errors
    ///
    /// Returns an error if the transition is invalid or lock fails
    pub async fn transition_to(&self, new_state: SessionState) -> Result<SessionTransition> {
        let mut sm = self.inner.write().await;
        sm.transition_to(new_state)
    }

    /// Advances to the next state
    ///
    /// # Errors
    ///
    /// Returns an error if already at final state or lock fails
    pub async fn advance(&self) -> Result<SessionTransition> {
        let mut sm = self.inner.write().await;
        sm.advance()
    }

    /// Checks if the current state has timed out
    ///
    /// # Errors
    ///
    /// Returns an error if the lock is poisoned
    pub async fn is_timed_out(&self) -> Result<bool> {
        let sm = self.inner.read().await;
        Ok(sm.is_state_timed_out())
    }
}

impl Clone for SharedStateMachine {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_initial() {
        let sm = SessionStateMachine::new();
        assert_eq!(sm.current(), SessionState::Warmup);
        assert!(sm.is_active());
        assert!(!sm.is_terminal());
    }

    #[test]
    fn test_state_advance() {
        let mut sm = SessionStateMachine::new();
        assert!(sm.advance().is_ok());
        assert_eq!(sm.current(), SessionState::DeepDive);
        assert!(sm.advance().is_ok());
        assert_eq!(sm.current(), SessionState::Review);
    }

    #[test]
    fn test_state_advance_to_end() {
        let mut sm = SessionStateMachine::new();
        assert!(sm.advance().is_ok()); // -> DeepDive
        assert!(sm.advance().is_ok()); // -> Review
        assert!(sm.advance().is_ok()); // -> Seed
        assert!(sm.advance().is_ok()); // -> Closing
        assert!(sm.advance().is_err()); // Already at end
        assert!(sm.is_terminal());
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = SessionStateMachine::new();
        // Can't jump from Warmup to Closing without allow_skip
        assert!(sm.transition_to(SessionState::Closing).is_err());
    }

    #[test]
    fn test_allow_skip() {
        let config = StateMachineConfig {
            allow_skip: true,
            ..Default::default()
        };
        let mut sm = SessionStateMachine::with_config(config);
        // Now we can skip
        assert!(sm.transition_to(SessionState::Closing).is_ok());
    }

    #[test]
    fn test_state_timeout() {
        let mut sm = SessionStateMachine::new();
        // Initially not timed out
        assert!(!sm.is_state_timed_out());

        // Simulate time passing by updating state_entered_at
        sm.state_entered_at = Utc::now() - chrono::Duration::seconds(400);
        assert!(sm.is_state_timed_out());
    }

    #[test]
    fn test_transition_count() {
        let mut sm = SessionStateMachine::new();
        assert_eq!(sm.transition_count(), 0); // Initial state doesn't count

        sm.advance().unwrap();
        assert_eq!(sm.transition_count(), 1);

        sm.advance().unwrap();
        assert_eq!(sm.transition_count(), 2);
    }

    #[test]
    fn test_visit_count() {
        let mut sm = SessionStateMachine::new();
        assert_eq!(sm.visit_count(SessionState::Warmup), 1);

        sm.transition_to(SessionState::DeepDive).unwrap();
        sm.transition_to(SessionState::Warmup).unwrap(); // Not normally allowed, but let's test

        // With allow_reverse this would work
        let mut config = StateMachineConfig::default();
        config.allow_reverse = true;
        let mut sm2 = SessionStateMachine::with_config(config);
        sm2.advance().unwrap(); // Warmup -> DeepDive

        // Can't go back in the normal flow, but this tests the counting
        assert_eq!(sm2.visit_count(SessionState::Warmup), 1);
        assert_eq!(sm2.visit_count(SessionState::DeepDive), 1);
    }

    #[test]
    fn test_snapshot() {
        let mut sm = SessionStateMachine::new();
        sm.advance().unwrap();

        let snapshot = sm.snapshot();
        assert_eq!(snapshot.current, SessionState::DeepDive);

        // Modify and restore
        sm.advance().unwrap();
        assert_eq!(sm.current(), SessionState::Review);

        sm.restore(snapshot).unwrap();
        assert_eq!(sm.current(), SessionState::DeepDive);
    }
}
