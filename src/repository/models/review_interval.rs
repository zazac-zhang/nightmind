// ============================================================
// Review Interval Model
// ============================================================
//! Review interval entity model for FSRS algorithm.
//!
//! This module defines the review interval structure based on the
//! review_intervals database table.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Review interval entity for FSRS algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewInterval {
    /// Unique review interval identifier
    pub id: Uuid,

    /// Associated knowledge point ID
    pub knowledge_point_id: Uuid,

    /// Associated user ID
    pub user_id: Uuid,

    /// Current interval in days
    pub interval_days: f32,

    /// Ease factor (stability multiplier)
    pub ease_factor: f32,

    /// Memory stability (optional advanced FSRS parameter)
    pub stability: Option<f32>,

    /// Memory retrievability (optional advanced FSRS parameter)
    pub retrievability: Option<f32>,

    /// Next review date
    pub next_review_date: NaiveDate,

    /// Last review date (optional)
    pub last_review_date: Option<NaiveDate>,

    /// Total number of reviews
    pub total_reviews: i32,

    /// Number of correct reviews
    pub correct_reviews: i32,

    /// Number of lapses (failed reviews)
    pub lapses: i32,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Create review interval request
#[derive(Debug, Clone)]
pub struct CreateReviewInterval {
    /// Knowledge point ID
    pub knowledge_point_id: Uuid,

    /// User ID
    pub user_id: Uuid,

    /// Initial interval in days (default: 1.0)
    pub interval_days: f32,

    /// Initial ease factor (default: 2.5)
    pub ease_factor: f32,

    /// First review date (default: today)
    pub next_review_date: NaiveDate,
}

/// Update review interval request
#[derive(Debug, Clone)]
pub struct UpdateReviewInterval {
    /// Review interval ID to update
    pub id: Uuid,

    /// New interval in days (optional)
    pub interval_days: Option<f32>,

    /// New ease factor (optional)
    pub ease_factor: Option<f32>,

    /// New stability (optional)
    pub stability: Option<f32>,

    /// New retrievability (optional)
    pub retrievability: Option<f32>,

    /// New next review date (optional)
    pub next_review_date: Option<NaiveDate>,
}

/// Review submission request
#[derive(Debug, Clone)]
pub struct SubmitReviewRequest {
    /// Review interval ID
    pub interval_id: Uuid,

    /// User's rating (1=Again, 2=Hard, 3=Good, 4=Easy)
    pub rating: u8,

    /// Time taken for review in seconds (optional)
    pub time_taken_seconds: Option<i32>,
}

/// Review statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStats {
    /// Total number of knowledge points with review intervals
    pub total_points: i64,

    /// Number of points due for review
    pub due_today: i64,

    /// Average interval across all points
    pub avg_interval: f32,

    /// Average ease factor across all points
    pub avg_ease: f32,

    /// Total number of reviews
    pub total_reviews: i64,

    /// Overall accuracy rate (0.0 - 1.0)
    pub accuracy_rate: f32,
}

impl ReviewInterval {
    /// Calculates the accuracy rate (correct / total reviews)
    #[must_use]
    pub fn accuracy_rate(&self) -> f32 {
        if self.total_reviews == 0 {
            0.0
        } else {
            self.correct_reviews as f32 / self.total_reviews as f32
        }
    }

    /// Checks if the review is due today or overdue
    #[must_use]
    pub fn is_due(&self) -> bool {
        self.next_review_date <= Utc::now().date_naive()
    }

    /// Gets the number of days overdue (negative if not overdue)
    #[must_use]
    pub fn days_overdue(&self) -> i64 {
        let today = Utc::now().date_naive();
        (today - self.next_review_date).num_days().max(0)
    }

    /// Gets the number of days until next review
    #[must_use]
    pub fn days_until_review(&self) -> i64 {
        let today = Utc::now().date_naive();
        (self.next_review_date - today).num_days().max(0)
    }
}

impl Default for CreateReviewInterval {
    fn default() -> Self {
        Self {
            knowledge_point_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            interval_days: 1.0,
            ease_factor: 2.5,
            next_review_date: Utc::now().date_naive(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_interval_accuracy() {
        let interval = ReviewInterval {
            id: Uuid::new_v4(),
            knowledge_point_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            interval_days: 3.0,
            ease_factor: 2.5,
            stability: None,
            retrievability: None,
            next_review_date: Utc::now().date_naive(),
            last_review_date: None,
            total_reviews: 10,
            correct_reviews: 8,
            lapses: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(interval.accuracy_rate(), 0.8);
    }

    #[test]
    fn test_review_interval_is_due() {
        let today = Utc::now().date_naive();

        // Due interval
        let due_interval = ReviewInterval {
            id: Uuid::new_v4(),
            knowledge_point_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            interval_days: 1.0,
            ease_factor: 2.5,
            stability: None,
            retrievability: None,
            next_review_date: today,
            last_review_date: None,
            total_reviews: 0,
            correct_reviews: 0,
            lapses: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(due_interval.is_due());

        // Future interval
        let future_interval = ReviewInterval {
            next_review_date: today + chrono::Duration::days(7),
            ..due_interval
        };

        assert!(!future_interval.is_due());
    }

    #[test]
    fn test_days_until_review() {
        let today = Utc::now().date_naive();

        let interval = ReviewInterval {
            id: Uuid::new_v4(),
            knowledge_point_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            interval_days: 1.0,
            ease_factor: 2.5,
            stability: None,
            retrievability: None,
            next_review_date: today + chrono::Duration::days(5),
            last_review_date: None,
            total_reviews: 0,
            correct_reviews: 0,
            lapses: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(interval.days_until_review(), 5);
        assert_eq!(interval.days_overdue(), 0);
    }

    #[test]
    fn test_create_review_interval_default() {
        let create = CreateReviewInterval::default();
        assert_eq!(create.interval_days, 1.0);
        assert_eq!(create.ease_factor, 2.5);
    }
}
