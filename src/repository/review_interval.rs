// ============================================================
// Review Interval Repository
// ============================================================
//! Review interval data access layer for FSRS algorithm.
//!
//! This module provides database operations for managing review intervals
//! and calculating next review dates using the FSRS algorithm.

use crate::core::memory::{EveningFSRS, Rating};
use crate::repository::models::review_interval::{
    CreateReviewInterval, ReviewInterval, ReviewStats, SubmitReviewRequest, UpdateReviewInterval,
};
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

/// Review interval repository trait
#[async_trait::async_trait]
pub trait ReviewIntervalRepository: Send + Sync {
    /// Creates a new review interval
    ///
    /// # Errors
    ///
    /// Returns an error if creation fails
    async fn create(&self, request: CreateReviewInterval) -> Result<ReviewInterval, RepositoryError>;

    /// Gets a review interval by ID
    ///
    /// # Errors
    ///
    /// Returns an error if not found
    async fn get_by_id(&self, id: Uuid) -> Result<ReviewInterval, RepositoryError>;

    /// Gets review interval by knowledge point ID
    ///
    /// # Errors
    ///
    /// Returns an error if not found
    async fn get_by_knowledge_point(
        &self,
        knowledge_point_id: Uuid,
    ) -> Result<Option<ReviewInterval>, RepositoryError>;

    /// Updates a review interval
    ///
    /// # Errors
    ///
    /// Returns an error if update fails
    async fn update(&self, request: UpdateReviewInterval) -> Result<ReviewInterval, RepositoryError>;

    /// Deletes a review interval
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Lists all due reviews for a user
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    async fn list_due_reviews(&self, user_id: Uuid) -> Result<Vec<ReviewInterval>, RepositoryError>;

    /// Lists all review intervals for a user
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<ReviewInterval>, RepositoryError>;

    /// Submits a review and updates the interval using FSRS
    ///
    /// # Errors
    ///
    /// Returns an error if submission or update fails
    async fn submit_review(
        &self,
        request: SubmitReviewRequest,
    ) -> Result<ReviewInterval, RepositoryError>;

    /// Gets review statistics for a user
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    async fn get_review_stats(&self, user_id: Uuid) -> Result<ReviewStats, RepositoryError>;

    /// Updates review interval using the database function
    ///
    /// # Errors
    ///
    /// Returns an error if update fails
    async fn update_review_interval(
        &self,
        interval_id: Uuid,
        rating: u8,
        time_taken: i32,
    ) -> Result<ReviewInterval, RepositoryError>;
}

/// Repository error type
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    /// Review interval not found
    #[error("Review interval not found: {0}")]
    NotFound(String),

    /// Invalid rating
    #[error("Invalid rating: {0}")]
    InvalidRating(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Postgres review interval repository implementation
pub struct PostgresReviewIntervalRepository {
    /// Database pool
    pool: sqlx::PgPool,
    /// FSRS algorithm instance
    fsrs: EveningFSRS,
}

impl PostgresReviewIntervalRepository {
    /// Creates a new Postgres review interval repository
    #[must_use]
    fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            fsrs: EveningFSRS::new(),
        }
    }

    /// Creates a new repository with custom FSRS parameters
    fn with_fsrs(pool: sqlx::PgPool, fsrs: EveningFSRS) -> Self {
        Self { pool, fsrs }
    }
}

#[async_trait::async_trait]
impl ReviewIntervalRepository for PostgresReviewIntervalRepository {
    async fn create(&self, request: CreateReviewInterval) -> Result<ReviewInterval, RepositoryError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO review_intervals (
                id, knowledge_point_id, user_id,
                interval_days, ease_factor,
                next_review_date, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(request.knowledge_point_id)
        .bind(request.user_id)
        .bind(request.interval_days)
        .bind(request.ease_factor)
        .bind(request.next_review_date)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_review_interval(row))
    }

    async fn get_by_id(&self, id: Uuid) -> Result<ReviewInterval, RepositoryError> {
        let row = sqlx::query("SELECT * FROM review_intervals WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(id.to_string()))?;

        Ok(row_to_review_interval(row))
    }

    async fn get_by_knowledge_point(
        &self,
        knowledge_point_id: Uuid,
    ) -> Result<Option<ReviewInterval>, RepositoryError> {
        let row = sqlx::query("SELECT * FROM review_intervals WHERE knowledge_point_id = $1")
            .bind(knowledge_point_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(row_to_review_interval))
    }

    async fn update(&self, request: UpdateReviewInterval) -> Result<ReviewInterval, RepositoryError> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            UPDATE review_intervals
            SET interval_days = COALESCE($2, interval_days),
                ease_factor = COALESCE($3, ease_factor),
                stability = COALESCE($4, stability),
                retrievability = COALESCE($5, retrievability),
                next_review_date = COALESCE($6, next_review_date),
                updated_at = $7
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(request.id)
        .bind(request.interval_days)
        .bind(request.ease_factor)
        .bind(request.stability)
        .bind(request.retrievability)
        .bind(request.next_review_date)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound(request.id.to_string()))?;

        Ok(row_to_review_interval(row))
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM review_intervals WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(id.to_string()));
        }

        Ok(())
    }

    async fn list_due_reviews(&self, user_id: Uuid) -> Result<Vec<ReviewInterval>, RepositoryError> {
        let today = Utc::now().date_naive();

        let rows = sqlx::query(
            r#"
            SELECT * FROM review_intervals
            WHERE user_id = $1 AND next_review_date <= $2
            ORDER BY next_review_date ASC
            "#,
        )
        .bind(user_id)
        .bind(today)
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<ReviewInterval> = rows.into_iter().map(row_to_review_interval).collect();

        Ok(items)
    }

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<ReviewInterval>, RepositoryError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM review_intervals
            WHERE user_id = $1
            ORDER BY next_review_date ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<ReviewInterval> = rows.into_iter().map(row_to_review_interval).collect();

        Ok(items)
    }

    async fn submit_review(
        &self,
        request: SubmitReviewRequest,
    ) -> Result<ReviewInterval, RepositoryError> {
        // Get current interval
        let current = self.get_by_id(request.interval_id).await?;

        // Validate rating
        let rating = Rating::from_score(request.rating)
            .ok_or_else(|| RepositoryError::InvalidRating(format!("Invalid rating: {}", request.rating)))?;

        // Calculate next interval using FSRS
        let (new_interval, new_ease) = self.fsrs.next_review(
            current.interval_days,
            current.ease_factor,
            rating,
            0.5, // Assume moderate cognitive load
            crate::repository::models::SessionState::Review,
        );

        // Update interval
        let now = Utc::now();
        let new_review_date = Utc::now().date_naive()
            + chrono::Duration::days(new_interval.ceil() as i64);

        let row = sqlx::query(
            r#"
            UPDATE review_intervals
            SET interval_days = $2,
                ease_factor = $3,
                next_review_date = $4,
                last_review_date = $5,
                total_reviews = total_reviews + 1,
                correct_reviews = correct_reviews + CASE WHEN $6 >= 3 THEN 1 ELSE 0 END,
                lapses = lapses + CASE WHEN $6 = 1 THEN 1 ELSE 0 END,
                updated_at = $7
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(request.interval_id)
        .bind(new_interval)
        .bind(new_ease)
        .bind(new_review_date)
        .bind(now.date_naive())
        .bind(request.rating as i32)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_review_interval(row))
    }

    async fn get_review_stats(&self, user_id: Uuid) -> Result<ReviewStats, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_points,
                COUNT(*) FILTER (WHERE next_review_date <= CURRENT_DATE) as due_today,
                COALESCE(AVG(interval_days), 0) as avg_interval,
                COALESCE(AVG(ease_factor), 0) as avg_ease,
                COALESCE(SUM(total_reviews), 0) as total_reviews,
                COALESCE(
                    SUM(correct_reviews)::float / NULLIF(SUM(total_reviews), 0),
                    0
                ) as accuracy_rate
            FROM review_intervals
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ReviewStats {
            total_points: row.get("total_points"),
            due_today: row.get("due_today"),
            avg_interval: row.get("avg_interval"),
            avg_ease: row.get("avg_ease"),
            total_reviews: row.get("total_reviews"),
            accuracy_rate: row.get("accuracy_rate"),
        })
    }

    async fn update_review_interval(
        &self,
        interval_id: Uuid,
        rating: u8,
        time_taken: i32,
    ) -> Result<ReviewInterval, RepositoryError> {
        // Validate rating
        if !(1..=4).contains(&rating) {
            return Err(RepositoryError::InvalidRating(format!("Invalid rating: {}", rating)));
        }

        // Call the database function
        let _row = sqlx::query("SELECT update_review_interval($1, $2, $3)")
            .bind(interval_id)
            .bind(rating as i32)
            .bind(time_taken)
            .fetch_one(&self.pool)
            .await?;

        // Fetch the updated interval
        self.get_by_id(interval_id).await
    }
}

/// Converts a database row to a ReviewInterval
fn row_to_review_interval(row: sqlx::postgres::PgRow) -> ReviewInterval {
    ReviewInterval {
        id: row.get("id"),
        knowledge_point_id: row.get("knowledge_point_id"),
        user_id: row.get("user_id"),
        interval_days: row.get("interval_days"),
        ease_factor: row.get("ease_factor"),
        stability: row.get("stability"),
        retrievability: row.get("retrievability"),
        next_review_date: row.get("next_review_date"),
        last_review_date: row.get("last_review_date"),
        total_reviews: row.get("total_reviews"),
        correct_reviews: row.get("correct_reviews"),
        lapses: row.get("lapses"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_stats_creation() {
        let stats = ReviewStats {
            total_points: 100,
            due_today: 15,
            avg_interval: 5.5,
            avg_ease: 2.6,
            total_reviews: 500,
            accuracy_rate: 0.85,
        };

        assert_eq!(stats.total_points, 100);
        assert_eq!(stats.due_today, 15);
        assert_eq!(stats.avg_interval, 5.5);
    }

    #[test]
    fn test_submit_review_request() {
        let request = SubmitReviewRequest {
            interval_id: Uuid::new_v4(),
            rating: 3, // Good
            time_taken_seconds: Some(30),
        };

        assert_eq!(request.rating, 3);
        assert_eq!(request.time_taken_seconds, Some(30));
    }

    #[test]
    fn test_create_review_interval_default() {
        let create = CreateReviewInterval::default();
        assert_eq!(create.interval_days, 1.0);
        assert_eq!(create.ease_factor, 2.5);
    }
}
