// ============================================================
// Session Flow Controller
// ============================================================
//! Session flow controller for cognitive load management.
//!
//! This module implements the five-stage evening session flow with
//! cognitive load targets and automatic state transitions based on
//! fatigue detection.

use crate::core::content::fatigue::{FatigueDetector, SuggestedAction};
use crate::repository::models::session::SessionState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session flow controller
///
/// Manages the progression through the five evening session stages:
/// Warmup → DeepDive → Review → Seed → Closing
pub struct SessionFlowController {
    /// Current session state
    current_state: SessionState,

    /// Time spent in current state (seconds)
    time_in_state: i64,

    /// Fatigue detector
    fatigue_detector: FatigueDetector,

    /// State start time
    state_start_time: DateTime<Utc>,
}

impl SessionFlowController {
    /// Creates a new session flow controller
    #[must_use]
    pub fn new(initial_state: SessionState) -> Self {
        Self {
            current_state: initial_state,
            time_in_state: 0,
            fatigue_detector: FatigueDetector::new(),
            state_start_time: Utc::now(),
        }
    }

    /// Gets the current session state
    #[must_use]
    pub const fn current_state(&self) -> SessionState {
        self.current_state
    }

    /// Updates the time spent in current state
    pub fn update_time(&mut self, seconds: i64) {
        self.time_in_state = seconds;
    }

    /// Gets the cognitive target for the current session state
    #[must_use]
    pub fn cognitive_target(&self) -> CognitiveTarget {
        match self.current_state {
            SessionState::Warmup => CognitiveTarget {
                load_range: (0.2, 0.4),
                goal: "激活今日学习记忆，建立安全氛围",
                topics: vec!["今日回顾", "轻松话题"],
                max_duration: 300, // 5 minutes
                description: "轻松的开场，帮助用户进入学习状态",
            },

            SessionState::DeepDive => CognitiveTarget {
                load_range: (0.5, 0.7),
                goal: "深度理解1-2个核心概念，主动回忆测试",
                topics: vec!["核心知识", "难点突破"],
                max_duration: 1200, // 20 minutes
                description: "核心学习阶段，深度探索和主动回忆",
            },

            SessionState::Review => CognitiveTarget {
                load_range: (0.4, 0.6),
                goal: "间隔复习，构建知识关联",
                topics: vec!["待复习内容", "关联问题"],
                max_duration: 1200, // 20 minutes
                description: "复习阶段，巩固记忆痕迹",
            },

            SessionState::Seed => CognitiveTarget {
                load_range: (0.2, 0.3),
                goal: "埋下明天的学习钩子",
                topics: vec!["明日预告", "开放性问题"],
                max_duration: 600, // 10 minutes
                description: "沉淀阶段，为下次学习做准备",
            },

            SessionState::Closing => CognitiveTarget {
                load_range: (0.1, 0.2),
                goal: "心理放松，积极暗示，准备入睡",
                topics: vec!["总结", "正面肯定"],
                max_duration: 300, // 5 minutes
                description: "温柔收尾，确保良好的睡眠质量",
            },
        }
    }

    /// Decides whether to transition to the next state
    ///
    /// This evaluates multiple triggers:
    ///
    /// 1. **Time limit**: Has max duration for this state been reached?
    /// 2. **Fatigue level**: Is cognitive load too high?
    /// 3. **Phase-specific conditions**: Completion metrics for current phase
    ///
    /// # Arguments
    ///
    /// * `session` - Current session
    /// * `metrics` - Session metrics collected during conversation
    ///
    /// # Returns
    ///
    /// `Some(next_state)` if transition is recommended, `None` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// let controller = SessionFlowController::new(SessionState::DeepDive);
    ///
    /// // After 20 minutes of DeepDive...
    /// if let Some(next_state) = controller.should_transition(&session, &metrics) {
    ///     assert_eq!(next_state, SessionState::Review);
    /// }
    /// ```
    pub fn should_transition(
        &self,
        session: &crate::repository::models::session::Session,
        metrics: &SessionMetrics,
    ) -> Option<SessionState> {
        let target = self.cognitive_target();

        // 1. Check time limit
        if self.time_in_state > target.max_duration {
            return Some(self.current_state.next()?);
        }

        // 2. Check fatigue level
        let fatigue = self.fatigue_detector.detect_fatigue(session, metrics);

        match fatigue.suggested_action {
            SuggestedAction::PrepareClosing => {
                // High fatigue: skip to Closing
                return Some(SessionState::Closing);
            },

            SuggestedAction::TakeBreak => {
                // Moderate fatigue: adjust state if needed
                if self.current_state == SessionState::DeepDive {
                    // DeepDive too intense, move to Review early
                    return Some(SessionState::Review);
                }
                // Otherwise continue with reduced content density
            },

            SuggestedAction::SlowDown => {
                // Low fatigue: reduce content density but stay in state
                // (handled by content transformation system)
            },

            SuggestedAction::Continue => {
                // Continue current state
            },
        }

        // 3. Check phase-specific conditions
        match self.current_state {
            SessionState::Warmup => {
                // Warmup: require user engagement > 60%
                if metrics.engagement_score > 0.6 {
                    return Some(SessionState::DeepDive);
                }
            },

            SessionState::Review => {
                // Review: complete > 80% of due reviews
                if metrics.review_completion_rate > 0.8 {
                    return Some(SessionState::Seed);
                }
            },

            SessionState::Seed => {
                // Seed: either time-based or low engagement
                if metrics.engagement_score < 0.3 {
                    return Some(SessionState::Closing);
                }
            },

            _ => {
                // DeepDive and Closing have no phase-specific triggers
            },
        }

        None
    }

    /// Advances to the next state (explicit transition)
    ///
    /// # Returns
    ///
    /// `Some(new_state)` if successful, `None` if already at Closing
    pub fn advance(&mut self) -> Option<SessionState> {
        match self.current_state.next() {
            Some(next_state) => {
                self.current_state = next_state;
                self.time_in_state = 0;
                self.state_start_time = Utc::now();
                Some(next_state)
            },
            None => None,
        }
    }

    /// Gets the elapsed time in current state
    pub fn elapsed_in_state(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.state_start_time)
            .num_seconds()
    }
}

/// Cognitive target for a session state
#[derive(Debug, Clone)]
pub struct CognitiveTarget {
    /// Target cognitive load range (min, max)
    pub load_range: (f32, f32),

    /// Goal description for this phase
    pub goal: &'static str,

    /// Suggested topics for this phase
    pub topics: Vec<&'static str>,

    /// Maximum duration in seconds
    pub max_duration: i64,

    /// Detailed description of this phase
    pub description: &'static str,
}

/// Session metrics for flow control
#[derive(Debug, Clone)]
pub struct SessionMetrics {
    /// Number of conversation turns
    pub turn_count: usize,

    /// Code and formula content density (0.0 - 1.0)
    pub code_formula_density: f32,

    /// User engagement score (0.0 - 1.0)
    pub engagement_score: f32,

    /// Review completion rate (0.0 - 1.0)
    pub review_completion_rate: f32,

    /// Average response time in milliseconds
    pub avg_response_time_ms: Option<i64>,
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self {
            turn_count: 0,
            code_formula_density: 0.0,
            engagement_score: 0.5,
            review_completion_rate: 0.0,
            avg_response_time_ms: None,
        }
    }
}

/// Transition trigger for state changes
#[derive(Debug, Clone)]
pub enum TransitionTrigger {
    /// User engagement threshold reached
    UserEngagement(f32),

    /// Cognitive load threshold reached
    CognitiveLoad(f32),

    /// Review completion threshold reached
    ReviewProgress(f32),

    /// Time limit or fatigue detected
    TimeOrFatigue,

    /// User explicitly requested transition
    UserInitiated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::models::session::SessionState;
    use uuid::Uuid;

    #[test]
    fn test_cognitive_target_warmup() {
        let controller = SessionFlowController::new(SessionState::Warmup);
        let target = controller.cognitive_target();

        assert_eq!(target.load_range, (0.2, 0.4));
        assert_eq!(target.max_duration, 300);
        assert!(target.topics.contains(&"今日回顾"));
    }

    #[test]
    fn test_cognitive_target_deepdive() {
        let controller = SessionFlowController::new(SessionState::DeepDive);
        let target = controller.cognitive_target();

        assert_eq!(target.load_range, (0.5, 0.7));
        assert_eq!(target.max_duration, 1200);
        assert!(target.topics.contains(&"核心知识"));
    }

    #[test]
    fn test_should_transition_time_limit() {
        let controller = SessionFlowController::new(SessionState::DeepDive);

        // Mock session that's been in DeepDive for 20 minutes
        let session = create_mock_session_with_state(SessionState::DeepDive);
        controller.time_in_state = 1200; // 20 minutes

        let metrics = SessionMetrics::default();

        let next = controller.should_transition(&session, &metrics);

        // Should transition due to time limit
        assert_eq!(next, Some(SessionState::Review));
    }

    #[test]
    fn test_should_transition_high_fatigue() {
        let controller = SessionFlowController::new(SessionState::DeepDive);

        // Mock session with high cognitive load
        let mut session = create_mock_session_with_state(SessionState::DeepDive);
        session.cognitive_load = 0.85;

        let metrics = SessionMetrics::default();

        let next = controller.should_transition(&session, &metrics);

        // High fatigue should trigger Closing
        assert_eq!(next, Some(SessionState::Closing));
    }

    #[test]
    fn test_should_transition_warmup_engagement() {
        let controller = SessionFlowController::new(SessionState::Warmup);

        let session = create_mock_session_with_state(SessionState::Warmup);

        let mut metrics = SessionMetrics::default();
        metrics.engagement_score = 0.7; // Good engagement

        let next = controller.should_transition(&session, &metrics);

        // High engagement in Warmup should trigger DeepDive
        assert_eq!(next, Some(SessionState::DeepDive));
    }

    #[test]
    fn test_should_transition_review_completion() {
        let controller = SessionFlowController::new(SessionState::Review);

        let session = create_mock_session_with_state(SessionState::Review);

        let mut metrics = SessionMetrics::default();
        metrics.review_completion_rate = 0.85; // > 80% complete

        let next = controller.should_transition(&session, &metrics);

        // High review completion should trigger Seed
        assert_eq!(next, Some(SessionState::Seed));
    }

    #[test]
    fn test_advance_state() {
        let mut controller = SessionFlowController::new(SessionState::Warmup);

        let next = controller.advance();
        assert_eq!(next, Some(SessionState::DeepDive));
        assert_eq!(controller.current_state, SessionState::DeepDive);

        let next = controller.advance();
        assert_eq!(next, Some(SessionState::Review));
        assert_eq!(controller.current_state, SessionState::Review);

        // Final advance to Closing
        controller.advance();
        assert_eq!(controller.current_state, SessionState::Closing);

        // Already at Closing, no more states
        let next = controller.advance();
        assert_eq!(next, None);
    }

    // Helper function to create mock sessions
    fn create_mock_session_with_state(state: SessionState) -> crate::repository::models::session::Session {
        crate::repository::models::session::Session {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            title: "Test Session".to_string(),
            state,
            topic_stack: None,
            cognitive_load: 0.5,
            started_at: Utc::now(),
            last_activity_at: Utc::now(),
            ended_at: None,
            metadata: None,
        }
    }
}
