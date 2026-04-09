// ============================================================
// Fatigue Detection System
// ============================================================
//! Fatigue detection for evening memory consolidation sessions.
//!
//! This module provides a simplified fatigue detection system that works
//! without audio data, using only session metrics and observable patterns.

use crate::repository::models::session::Session;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Fatigue detection thresholds
#[derive(Debug, Clone)]
pub struct FatigueThresholds {
    /// Maximum session duration in seconds (default: 60 minutes)
    pub max_duration_seconds: i64,

    /// Maximum number of conversation turns (default: 40)
    pub max_turns: usize,

    /// High cognitive load threshold (0.0-1.0)
    pub high_load_threshold: f32,

    /// Fatigue increase per second of session (default: 0.1% per second)
    pub time_based_increase: f32,

    /// Maximum acceptable code/formula density (0.0-1.0)
    pub max_content_density: f32,
}

impl Default for FatigueThresholds {
    fn default() -> Self {
        Self {
            max_duration_seconds: 3600,  // 60 minutes
            max_turns: 40,
            high_load_threshold: 0.7,
            time_based_increase: 0.001, // 0.1% per second
            max_content_density: 0.4,
        }
    }
}

/// Fatigue detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatigueLevel {
    /// Fatigue score (0.0 - 1.0)
    ///
    /// - 0.0 - 0.3: Low fatigue, continue current activity
    /// - 0.3 - 0.5: Moderate fatigue, consider slowing down
    /// - 0.5 - 0.7: High fatigue, suggest taking a break
    /// - 0.7 - 1.0: Very high fatigue, prepare to close session
    pub score: f32,

    /// Human-readable reasons for the fatigue score
    pub reasons: Vec<String>,

    /// Suggested action based on fatigue level
    pub suggested_action: SuggestedAction,
}

/// Suggested actions based on fatigue level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestedAction {
    /// Continue current activity
    Continue,

    /// Slow down rhythm, increase pauses
    SlowDown,

    /// Suggest taking a short break
    TakeBreak,

    /// Prepare to close the session
    PrepareClosing,
}

/// Session metrics for fatigue calculation
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

/// Fatigue detector for evening sessions
pub struct FatigueDetector {
    thresholds: FatigueThresholds,
}

impl Default for FatigueDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FatigueDetector {
    /// Creates a new fatigue detector with default thresholds
    #[must_use]
    pub fn new() -> Self {
        Self {
            thresholds: FatigueThresholds::default(),
        }
    }

    /// Creates a new fatigue detector with custom thresholds
    #[must_use]
    pub fn with_thresholds(thresholds: FatigueThresholds) -> Self {
        Self { thresholds }
    }

    /// Detects fatigue level based on session metrics
    ///
    /// This analyzes multiple dimensions of fatigue:
    ///
    /// 1. **Time-based fatigue**: Linear increase over session duration
    /// 2. **Turn count fatigue**: Too many conversation turns
    /// 3. **Cognitive load fatigue**: Session has high intrinsic complexity
    /// 4. **Content density fatigue**: Too much code/formula content
    ///
    /// # Arguments
    ///
    /// * `session` - Current session state
    /// * `metrics` - Session metrics collected during conversation
    ///
    /// # Returns
    ///
    /// A fatigue level with score and suggested action
    ///
    /// # Example
    ///
    /// ```rust
    /// let detector = FatigueDetector::new();
    /// let session = Session::mock();
    /// let metrics = SessionMetrics::mock();
    ///
    /// let fatigue = detector.detect_fatigue(&session, &metrics);
    ///
    /// match fatigue.suggested_action {
    ///     SuggestedAction::Continue => println!("Keep going"),
    ///     SuggestedAction::SlowDown => println!("Let's take it easier"),
    ///     SuggestedAction::PrepareClosing => println!("Time to wrap up"),
    ///     _ => println!("Consider a break"),
    /// }
    /// ```
    pub fn detect_fatigue(
        &self,
        session: &Session,
        metrics: &SessionMetrics,
    ) -> FatigueLevel {
        let mut score = 0.0f32;
        let mut reasons = Vec::new();

        // 1. Time-based fatigue (linear growth)
        let duration = session.duration_seconds();
        let time_score = (duration as f32 * self.thresholds.time_based_increase)
            .min(0.3); // Cap at 30% contribution
        score += time_score;

        if time_score > 0.2 {
            reasons.push(format!(
                "会话时间较长（{}秒）",
                duration
            ));
        }

        // 2. Conversation turn count fatigue
        let turn_score = (metrics.turn_count as f32 / self.thresholds.max_turns as f32 * 0.2)
            .min(0.2); // Cap at 20% contribution
        score += turn_score;

        if turn_score > 0.15 {
            reasons.push(format!(
                "对话轮次较多（{}次）",
                metrics.turn_count
            ));
        }

        // 3. Cognitive load fatigue
        if session.cognitive_load > self.thresholds.high_load_threshold {
            let load_excess = (session.cognitive_load - 0.7) / 0.3;
            let load_score = load_excess.min(0.3) * 0.3;
            score += load_score;

            reasons.push(format!(
                "认知负荷较高（{:.0}）",
                session.cognitive_load
            ));
        }

        // 4. Content density fatigue
        if metrics.code_formula_density > self.thresholds.max_content_density {
            let density_excess = (metrics.code_formula_density - 0.4) / 0.6;
            let density_score = density_excess.min(1.0) * 0.2;
            score += density_score;

            reasons.push("技术内容密度较高".to_string());
        }

        // 5. Response time fatigue (if available)
        if let Some(response_time) = metrics.avg_response_time_ms {
            if response_time > 5000 {
                let response_factor = ((response_time - 5000) as f32 / 5000.0).min(0.1);
                score += response_factor;

                if response_factor > 0.05 {
                    reasons.push(format!(
                        "响应时间较长（{}ms）",
                        response_time
                    ));
                }
            }
        }

        FatigueLevel {
            score: score.min(1.0),
            reasons,
            suggested_action: self.suggest_action(score),
        }
    }

    /// Suggests an action based on fatigue score
    fn suggest_action(&self, score: f32) -> SuggestedAction {
        match score {
            s if s < 0.3 => SuggestedAction::Continue,
            s if s < 0.5 => SuggestedAction::SlowDown,
            s if s < 0.7 => SuggestedAction::TakeBreak,
            _ => SuggestedAction::PrepareClosing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_fatigue_thresholds_default() {
        let thresholds = FatigueThresholds::default();
        assert_eq!(thresholds.max_duration_seconds, 3600);
        assert_eq!(thresholds.max_turns, 40);
        assert_eq!(thresholds.high_load_threshold, 0.7);
    }

    #[test]
    fn test_fatigue_detection_low_fatigue() {
        let detector = FatigueDetector::new();

        // Mock session with low fatigue indicators
        let session = Session {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            title: "Test Session".to_string(),
            state: crate::repository::models::session::SessionState::Warmup,
            topic_stack: None,
            cognitive_load: 0.3,
            started_at: Utc::now(),
            last_activity_at: Utc::now(),
            ended_at: None,
            metadata: None,
        };

        let metrics = SessionMetrics {
            turn_count: 5,
            code_formula_density: 0.1,
            engagement_score: 0.8,
            review_completion_rate: 0.5,
            avg_response_time_ms: Some(2000),
        };

        let fatigue = detector.detect_fatigue(&session, &metrics);

        // Should have low fatigue
        assert!(fatigue.score < 0.3);
        assert!(matches!(fatigue.suggested_action, SuggestedAction::Continue));
    }

    #[test]
    fn test_fatigue_detection_high_fatigue() {
        let detector = FatigueDetector::new();

        // Mock session with high fatigue indicators
        let mut session = Session {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            title: "Test Session".to_string(),
            state: crate::repository::models::session::SessionState::DeepDive,
            topic_stack: None,
            cognitive_load: 0.85, // High cognitive load
            started_at: Utc::now() - chrono::Duration::seconds(2700), // 45 min ago
            last_activity_at: Utc::now(),
            ended_at: None,
            metadata: None,
        };

        let metrics = SessionMetrics {
            turn_count: 35, // Many turns
            code_formula_density: 0.6, // High code density
            engagement_score: 0.4,
            review_completion_rate: 0.7,
            avg_response_time_ms: Some(8000), // Slow responses
        };

        let fatigue = detector.detect_fatigue(&session, &metrics);

        // Should have high fatigue
        assert!(fatigue.score > 0.7);
        assert!(matches!(fatigue.suggested_action, SuggestedAction::PrepareClosing));
        assert!(!fatigue.reasons.is_empty());
    }

    #[test]
    fn test_fatigue_detection_time_based() {
        let detector = FatigueDetector::new();

        // Short session: low fatigue
        let short_session = Session {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            title: "Short Session".to_string(),
            state: crate::repository::models::session::SessionState::Warmup,
            topic_stack: None,
            cognitive_load: 0.3,
            started_at: Utc::now() - chrono::Duration::seconds(300), // 5 min
            last_activity_at: Utc::now(),
            ended_at: None,
            metadata: None,
        };

        let metrics = SessionMetrics::default();

        let fatigue_short = detector.detect_fatigue(&short_session, &metrics);

        // Long session: high fatigue
        let mut long_session = short_session.clone();
        long_session.started_at = Utc::now() - chrono::Duration::seconds(3300); // 55 min

        let fatigue_long = detector.detect_fatigue(&long_session, &metrics);

        assert!(fatigue_long.score > fatigue_short.score);
    }

    #[test]
    fn test_fatigue_detection_cognitive_load() {
        let detector = FatigueDetector::new();

        // Low cognitive load
        let mut session_low = Session {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            title: "Test".to_string(),
            state: crate::repository::models::session::SessionState::Warmup,
            topic_stack: None,
            cognitive_load: 0.4,
            started_at: Utc::now(),
            last_activity_at: Utc::now(),
            ended_at: None,
            metadata: None,
        };

        let metrics = SessionMetrics::default();

        let fatigue_low = detector.detect_fatigue(&session_low, &metrics);

        // High cognitive load
        session_low.cognitive_load = 0.9;
        let fatigue_high = detector.detect_fatigue(&session_low, &metrics);

        assert!(fatigue_high.score > fatigue_low.score);
    }

    #[test]
    fn test_suggested_action() {
        let detector = FatigueDetector::new();

        // Test each action boundary
        assert!(matches!(
            detector.suggest_action(0.2),
            SuggestedAction::Continue
        ));
        assert!(matches!(
            detector.suggest_action(0.4),
            SuggestedAction::SlowDown
        ));
        assert!(matches!(
            detector.suggest_action(0.6),
            SuggestedAction::TakeBreak
        ));
        assert!(matches!(
            detector.suggest_action(0.8),
            SuggestedAction::PrepareClosing
        ));
    }
}
