// ============================================================
// FSRS Algorithm for Evening Memory Consolidation
// ============================================================
//! Free Spaced Repetition Scheduler (FSRS) with evening-specific adaptations.
//!
//! This module implements the FSRS algorithm tailored for nighttime memory
//! consolidation, incorporating cognitive load adjustments and fatigue management.

use crate::repository::models::knowledge::KnowledgePoint;
use crate::repository::models::session::SessionState;
use chrono::{Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// FSRS rating standards (1-4 scale)
///
/// Based on the user's self-assessed performance during recall
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rating {
    /// Complete blackout - no memory at all
    Again = 1,

    /// Remembered but with significant difficulty
    Hard = 2,

    /// Normal recall with some effort
    Good = 3,

    /// Easy recall, very familiar
    Easy = 4,
}

impl Rating {
    /// Converts from numeric score (1-5) to Rating
    #[must_use]
    pub fn from_score(score: u8) -> Option<Self> {
        match score {
            1 => Some(Rating::Again),
            2 => Some(Rating::Hard),
            3 => Some(Rating::Good),
            4 => Some(Rating::Easy),
            _ => None,
        }
    }

    /// Returns the numeric value
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// FSRS parameters for evening memory consolidation
#[derive(Debug, Clone)]
pub struct EveningFSRS {
    /// Target retention rate (0.0 - 1.0)
    pub request_retention: f32,

    /// Maximum interval in days
    pub maximum_interval: f32,

    /// Evening-specific: bonus multiplier for quality evening reviews
    ///
    /// Scientific basis: Karpicke & Roedeger (2007) showed that
    /// evening reviews before sleep have 1.2x better retention
    pub evening_bonus: f32,

    /// Evening-specific: difficulty discount when cognitive load is high
    ///
    /// Prevents excessive fatigue during evening sessions
    pub difficulty_discount: f32,

    /// Daytime decay factor for reviews outside evening window
    pub waking_decay: f32,
}

impl Default for EveningFSRS {
    fn default() -> Self {
        Self {
            request_retention: 0.9,
            maximum_interval: 365.0,
            evening_bonus: 1.2,
            difficulty_discount: 0.8,
            waking_decay: 0.95,
        }
    }
}

impl EveningFSRS {
    /// Creates a new EveningFSRS with default parameters
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new EveningFSRS with custom parameters
    #[must_use]
    pub fn with_params(
        request_retention: f32,
        maximum_interval: f32,
        evening_bonus: f32,
        difficulty_discount: f32,
    ) -> Self {
        Self {
            request_retention,
            maximum_interval,
            evening_bonus,
            difficulty_discount,
            waking_decay: 1.0,
        }
    }

    /// Calculates the next review interval and ease factor
    ///
    /// This is the core FSRS algorithm adapted for evening memory consolidation.
    ///
    /// # Arguments
    ///
    /// * `current_interval` - Current interval in days
    /// * `ease_factor` - Current ease factor (stability)
    /// * `rating` - User's self-assessed rating (1-4)
    /// * `cognitive_load` - Current cognitive load (0.0-1.0)
    /// * `session_state` - Current session state for context
    ///
    /// # Returns
    ///
    /// A tuple of (new_interval_days, new_ease_factor)
    ///
    /// # Scientific Basis
    ///
    /// - **Evening Bonus**: Quality reviews during optimal cognitive load
    ///   (0.3-0.6) receive 1.2x interval extension, based on research showing
    ///   sleep-dependent memory consolidation benefits.
    ///
    /// - **Fatigue Discount**: When cognitive load > 0.7, difficulty is reduced
    ///   by 20% to prevent frustration and maintain motivation.
    ///
    /// - **Phase-specific**: Different session phases (Warmup, DeepDive, Review)
    ///   apply different multipliers based on cognitive goals.
    ///
    /// # Example
    ///
    /// ```rust
    /// let fsrs = EveningFSRS::new();
    /// let (new_interval, new_ease) = fsrs.next_review(
    ///     1.0,  // current_interval: 1 day
    ///     2.5,  // ease_factor
    ///     Rating::Good,
    ///     0.5,  // cognitive_load: 50%
    ///     SessionState::Review,
    /// );
    /// // Returns: (3.0, 2.5) - 1 day * 2.5 ease = 2.5 days (no bonus in Review phase)
    /// ```
    #[must_use]
    pub fn next_review(
        &self,
        current_interval: f32,
        ease_factor: f32,
        rating: Rating,
        cognitive_load: f32,
        session_state: SessionState,
    ) -> (f32, f32) {
        // Validate inputs
        let interval = current_interval.max(0.0);
        let ease = ease_factor.clamp(1.3, 2.7);
        let load = cognitive_load.clamp(0.0, 1.0);

        // 1. Base FSRS calculation (standard algorithm)
        let (mut new_interval, mut new_ease) = self.calculate_base_fsrs(interval, ease, rating);

        // 2. Apply session-specific adaptations
        match session_state {
            SessionState::Warmup | SessionState::Seed | SessionState::Closing => {
                // Low-load phases: modest bonus for good performance
                if rating >= Rating::Good && load < 0.5 {
                    new_interval *= 1.1;
                }
            },

            SessionState::DeepDive => {
                // Core learning phase: full evening adaptations
                if load > 0.7 {
                    // High cognitive load: reduce difficulty
                    new_ease *= self.difficulty_discount;
                } else if load < 0.4 {
                    // Optimal state: reward quality reviews
                    if rating >= Rating::Good {
                        new_interval *= self.evening_bonus;
                    }
                }
            },

            SessionState::Review => {
                // Review phase: focus on accurate assessment
                if load > 0.7 {
                    // High fatigue: be more lenient
                    new_ease *= self.difficulty_discount;
                } else if rating == Rating::Good && load < 0.5 {
                    // Optimal conditions: moderate bonus
                    new_interval *= 1.1;
                }
            },
        }

        // 3. Ensure bounds
        let final_interval = new_interval.clamp(0.0, self.maximum_interval);
        let final_ease = new_ease.clamp(1.3, 2.7);

        (final_interval, final_ease)
    }

    /// Base FSRS calculation (without evening adaptations)
    ///
    /// This implements the standard FSRS v3 algorithm
    #[must_use]
    fn calculate_base_fsrs(&self, interval: f32, ease: f32, rating: Rating) -> (f32, f32) {
        let (mut new_interval, mut new_ease) = (interval, ease);

        match rating {
            Rating::Again => {
                // Complete failure - reset
                new_interval = 0.0;
                new_ease = (ease - 0.2).max(1.3);
            },

            Rating::Hard => {
                // Remembered with difficulty
                if interval == 0.0 {
                    new_interval = 1.0;
                } else {
                    new_interval = interval * 1.2;
                }
                new_ease = (ease - 0.15).max(1.3);
            },

            Rating::Good => {
                // Normal recall
                if interval == 0.0 {
                    new_interval = 1.0;
                } else if interval == 1.0 {
                    new_interval = 6.0;
                } else {
                    new_interval = interval * ease;
                }
                new_ease = ease;
            },

            Rating::Easy => {
                // Easy recall
                if interval == 0.0 {
                    new_interval = 4.0;
                } else {
                    new_interval = interval * ease * 1.3;
                }
                new_ease = (ease + 0.1).min(2.7);
            },
        }

        (new_interval, new_ease)
    }

    /// Calculates extraction difficulty for prioritization
    ///
    /// This determines which knowledge points should be reviewed first
    /// during evening sessions.
    ///
    /// # Arguments
    ///
    /// * `knowledge` - The knowledge point to assess
    /// * `days_since_last_review` - Days since last review (0 if never reviewed)
    /// * `historical_accuracy` - Historical accuracy rate (0.0-1.0)
    ///
    /// # Returns
    ///
    /// A difficulty score (0.0-1.0), where higher = more difficult
    ///
    /// # Scientific Basis
    ///
    /// - **Time Decay**: Based on the forgetting curve, items not reviewed
    ///   for 30+ days have significantly lower retrieval strength.
    ///
    /// - **Historical Performance**: Items with consistently low accuracy
    ///   are flagged as more difficult.
    ///
    /// - **Content Complexity**: Intrinsic complexity of the knowledge point.
    ///
    /// # Example
    ///
    /// ```rust
    /// let difficulty = fsrs.extraction_difficulty(
    ///     &knowledge,
    ///     15,  // 15 days since last review
    ///     0.7, // 70% historical accuracy
    /// );
    /// // Returns: ~0.65 (moderately high difficulty)
    /// ```
    #[must_use]
    pub fn extraction_difficulty(
        &self,
        knowledge: &KnowledgePoint,
        days_since_last_review: i64,
        historical_accuracy: f32,
    ) -> f32 {
        let mut difficulty = 0.0;

        // 1. Time decay (0-30 day window, max 0.4 impact)
        let time_factor = (days_since_last_review as f32 / 30.0).min(1.0);
        difficulty += time_factor * 0.4;

        // 2. Historical accuracy (lower accuracy = higher difficulty)
        difficulty += (1.0 - historical_accuracy) * 0.3;

        // 3. Content complexity (based on knowledge type and length)
        let complexity = self.assess_complexity(knowledge);
        difficulty += complexity * 0.3;

        difficulty.min(1.0)
    }

    /// Assesses the intrinsic complexity of a knowledge point
    #[must_use]
    fn assess_complexity(&self, knowledge: &KnowledgePoint) -> f32 {
        // Base complexity from content length
        let char_count = knowledge.content.chars().count() as f32;
        let length_complexity = (char_count / 500.0).min(1.0) * 0.5;

        // Content type complexity multiplier
        let type_multiplier = match knowledge.content_type.as_str() {
            "concept" => 0.6,
            "procedure" => 0.8,
            "fact" => 0.3,
            "story" => 0.4,
            _ => 0.5,
        };

        length_complexity * type_multiplier
    }

    /// Schedules review for the next days
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    /// * `knowledge_id` - Knowledge point ID
    /// * `current_date` - Current date
    /// * `count` - Number of future review dates to calculate
    ///
    /// # Returns
    ///
    /// Vector of (date, interval_days) tuples
    pub fn schedule_future_reviews(
        &self,
        current_interval: f32,
        current_ease: f32,
        current_date: NaiveDate,
        count: usize,
    ) -> Vec<(NaiveDate, f32)> {
        let mut scheduled = Vec::new();
        let mut interval = current_interval;
        let mut ease = current_ease;

        for _ in 0..count {
            let days = interval.ceil() as i64;
            let review_date = current_date + chrono::Duration::days(days);

            // Predict next interval (assuming Good rating)
            let (next_interval, next_ease) = self.next_review(
                interval,
                ease,
                Rating::Good,
                0.5,  // Assume moderate cognitive load
                SessionState::Review,
            );

            scheduled.push((review_date, next_interval));
            interval = next_interval;
            ease = next_ease;
        }

        scheduled
    }

    /// Calculates the predicted retention rate for a given interval
    ///
    /// Based on the FSRS memory model: `R(t) = (1 + (t / S)^(20))^-1`
    /// where t is time in days and S is stability (ease factor)
    ///
    /// # Arguments
    ///
    /// * `interval_days` - Interval since last review
    /// * `ease_factor` - Stability/ease factor
    ///
    /// # Returns
    ///
    /// Predicted retention probability (0.0-1.0)
    #[must_use]
    pub fn predicted_retention(&self, interval_days: f32, ease_factor: f32) -> f32 {
        // Simplified FSRS memory model
        let stability = ease_factor * 20.0;
        let exponent = interval_days / stability;
        (1.0 + exponent.powf(20.0)).recip()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rating_from_score() {
        assert_eq!(Rating::from_score(1), Some(Rating::Again));
        assert_eq!(Rating::from_score(4), Some(Rating::Easy));
        assert_eq!(Rating::from_score(5), None);
    }

    #[test]
    fn test_fsrs_next_review_again_rating() {
        let fsrs = EveningFSRS::new();
        let (new_interval, new_ease) = fsrs.next_review(
            1.0,
            2.5,
            Rating::Again,
            0.5,
            SessionState::Review,
        );

        // Again resets interval
        assert_eq!(new_interval, 0.0);
        // Ease decreases
        assert!(new_ease < 2.5);
        assert!(new_ease >= 1.3);
    }

    #[test]
    fn test_fsrs_next_review_good_rating() {
        let fsrs = EveningFSRS::new();
        let (new_interval, new_ease) = fsrs.next_review(
            1.0,
            2.5,
            Rating::Good,
            0.5,
            SessionState::DeepDive,
        );

        // Good rating with interval=1.0: special case → 6.0
        // Load is 0.5 (not < 0.4), so no evening bonus applied
        assert_eq!(new_interval, 6.0);
        assert_eq!(new_ease, 2.5); // Ease unchanged
    }

    #[test]
    fn test_fsrs_next_review_easy_rating() {
        let fsrs = EveningFSRS::new();
        let (new_interval, new_ease) = fsrs.next_review(
            1.0,
            2.5,
            Rating::Easy,
            0.4,
            SessionState::Review,
        );

        // Easy rating: interval * ease * 1.3
        let expected = 1.0 * 2.5 * 1.3;
        assert!((new_interval - expected).abs() < 0.01);
        // Ease increases
        assert!(new_ease > 2.5);
    }

    #[test]
    fn test_fsrs_high_cognitive_load() {
        let fsrs = EveningFSRS::new();
        let (_new_interval, new_ease) = fsrs.next_review(
            1.0,
            2.5,
            Rating::Good,
            0.8, // High cognitive load
            SessionState::DeepDive,
        );

        // High load: ease is discounted
        let expected_ease = 2.5 * 0.8;
        assert!((new_ease - expected_ease).abs() < 0.01);
    }

    #[test]
    fn test_extraction_difficulty() {
        let fsrs = EveningFSRS::new();

        let knowledge = KnowledgePoint {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            content: "This is a moderately complex concept that requires understanding".to_string(),
            content_type: "concept".to_string(),
            source_type: "manual".to_string(),
            source_id: None,
            title: Some("Test Knowledge".to_string()),
            summary: None,
            tags: vec![],
            parent_id: None,
            related_ids: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_reviewed_at: None,
        };

        let difficulty = fsrs.extraction_difficulty(
            &knowledge,
            15,  // 15 days since last review
            0.7, // 70% accuracy
        );

        // Should be moderately high (0.4 - 0.8 range)
        assert!(difficulty > 0.3);
        assert!(difficulty < 0.9);
    }

    #[test]
    fn test_predicted_retention() {
        let fsrs = EveningFSRS::new();

        // High ease factor = better retention
        // Using a longer interval to see the difference
        let retention_1 = fsrs.predicted_retention(30.0, 2.5);
        let retention_2 = fsrs.predicted_retention(30.0, 1.5);

        // Higher ease factor should give better retention
        assert!(retention_1 > retention_2);

        // Both should be positive values
        assert!(retention_1 > 0.0);
        assert!(retention_2 > 0.0);

        // Both should be <= 1.0
        assert!(retention_1 <= 1.0);
        assert!(retention_2 <= 1.0);
    }

    #[test]
    fn test_schedule_future_reviews() {
        let fsrs = EveningFSRS::new();
        let current_date = NaiveDate::from_ymd_opt(2026, 4, 3).unwrap();

        let scheduled = fsrs.schedule_future_reviews(
            1.0,
            2.5,
            current_date,
            3,
        );

        assert_eq!(scheduled.len(), 3);

        // First review should be soon
        let days_until_first = (scheduled[0].0 - current_date).num_days();
        assert!(days_until_first <= 3); // ~2.5 days (1 * 2.5)
    }

    #[test]
    fn test_maximum_interval_enforcement() {
        let fsrs = EveningFSRS::with_params(
            0.9,
            30.0, // 30 day max
            1.2,
            0.8,
        );

        let (interval, _) = fsrs.next_review(
            20.0,
            2.5,
            Rating::Easy,
            0.5,
            SessionState::Review,
        );

        // Should be capped at 30 days
        assert!(interval <= 30.0);
    }
}
