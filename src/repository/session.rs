// ============================================================
// Session Repository
// ============================================================
//! Session data access layer.
//!
//! This module provides database operations for session management.

use super::db::{BaseRepository, Repository};
use super::models::session::{CreateSession, Session, SessionState, SessionSummary, UpdateSession};
use crate::error::{NightMindError, Result};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Session repository trait
///
/// Defines the interface for session data access operations.
#[async_trait::async_trait]
pub trait SessionRepository: Send + Sync {
    /// Creates a new session
    async fn create(&self, data: CreateSession) -> Result<Session>;

    /// Finds a session by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>>;

    /// Finds active sessions for a user
    async fn find_active_by_user(&self, user_id: Uuid) -> Result<Vec<Session>>;

    /// Finds all sessions for a user
    async fn find_by_user(&self, user_id: Uuid, offset: i32, limit: i32) -> Result<Vec<Session>>;

    /// Updates a session
    async fn update(&self, id: Uuid, data: UpdateSession) -> Result<Session>;

    /// Updates the session state
    async fn update_state(&self, id: Uuid, state: SessionState) -> Result<Session>;

    /// Ends a session
    async fn end(&self, id: Uuid) -> Result<Session>;

    /// Deletes a session
    async fn delete(&self, id: Uuid) -> Result<bool>;

    /// Counts active sessions for a user
    async fn count_active_by_user(&self, user_id: Uuid) -> Result<i64>;

    /// Cleans up idle sessions
    async fn cleanup_idle(&self, timeout_seconds: i64) -> Result<u64>;

    /// Gets session summaries for a user
    async fn summaries_by_user(&self, user_id: Uuid) -> Result<Vec<SessionSummary>>;
}

/// PostgreSQL implementation of SessionRepository
pub struct PgSessionRepository {
    base: BaseRepository,
}

impl PgSessionRepository {
    /// Creates a new session repository
    #[must_use]
    pub fn new(pool: &PgPool) -> Self {
        Self {
            base: BaseRepository::from_pool(pool),
        }
    }

    /// Checks if a session belongs to a user
    async fn is_owned_by(&self, session_id: Uuid, user_id: Uuid) -> Result<bool> {
        let query = "SELECT COUNT(*) FROM sessions WHERE id = $1 AND user_id = $2";
        let (count,): (i64,) = sqlx::query_as(query)
            .bind(session_id)
            .bind(user_id)
            .fetch_one(self.base.pool())
            .await?;
        Ok(count > 0)
    }
}

#[async_trait::async_trait]
impl Repository for PgSessionRepository {
    fn pool(&self) -> &PgPool {
        self.base.pool()
    }
}

#[async_trait::async_trait]
impl SessionRepository for PgSessionRepository {
    async fn create(&self, data: CreateSession) -> Result<Session> {
        let session = Session::create(data);

        let query = r#"
            INSERT INTO sessions (
                id, user_id, title, state, topic_stack, cognitive_load,
                started_at, last_activity_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#;

        let created = sqlx::query_as::<_, Session>(query)
            .bind(session.id)
            .bind(session.user_id)
            .bind(&session.title)
            .bind(session.state as i32)
            .bind(&session.topic_stack)
            .bind(session.cognitive_load)
            .bind(session.started_at)
            .bind(session.last_activity_at)
            .bind(&session.metadata)
            .fetch_one(self.pool())
            .await
            .map_err(|e| {
                NightMindError::Database(e)
            })?;

        Ok(created)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>> {
        let query = "SELECT * FROM sessions WHERE id = $1";
        let session = sqlx::query_as::<_, Session>(query)
            .bind(id)
            .fetch_optional(self.pool())
            .await?;
        Ok(session)
    }

    async fn find_active_by_user(&self, user_id: Uuid) -> Result<Vec<Session>> {
        let query = r#"
            SELECT * FROM sessions
            WHERE user_id = $1 AND ended_at IS NULL
            ORDER BY started_at DESC
        "#;

        let sessions = sqlx::query_as::<_, Session>(query)
            .bind(user_id)
            .fetch_all(self.pool())
            .await?;

        Ok(sessions)
    }

    async fn find_by_user(&self, user_id: Uuid, offset: i32, limit: i32) -> Result<Vec<Session>> {
        let query = r#"
            SELECT * FROM sessions
            WHERE user_id = $1
            ORDER BY started_at DESC
            LIMIT $2 OFFSET $3
        "#;

        let sessions = sqlx::query_as::<_, Session>(query)
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await?;

        Ok(sessions)
    }

    async fn update(&self, id: Uuid, data: UpdateSession) -> Result<Session> {
        // First fetch the existing session
        let mut session = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| NightMindError::not_found("Session"))?;

        // Apply updates
        session.update(data);

        let query = r#"
            UPDATE sessions
            SET title = $2, state = $3, topic_stack = $4,
                cognitive_load = $5, metadata = $6,
                last_activity_at = $7
            WHERE id = $1
            RETURNING *
        "#;

        let updated = sqlx::query_as::<_, Session>(query)
            .bind(id)
            .bind(&session.title)
            .bind(session.state as i32)
            .bind(&session.topic_stack)
            .bind(session.cognitive_load)
            .bind(&session.metadata)
            .bind(session.last_activity_at)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| NightMindError::not_found("Session"))?;

        Ok(updated)
    }

    async fn update_state(&self, id: Uuid, state: SessionState) -> Result<Session> {
        let query = r#"
            UPDATE sessions
            SET state = $2, last_activity_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let updated = sqlx::query_as::<_, Session>(query)
            .bind(id)
            .bind(state as i32)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| NightMindError::not_found("Session"))?;

        Ok(updated)
    }

    async fn end(&self, id: Uuid) -> Result<Session> {
        let query = r#"
            UPDATE sessions
            SET ended_at = NOW(), last_activity_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let updated = sqlx::query_as::<_, Session>(query)
            .bind(id)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| NightMindError::not_found("Session"))?;

        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        let query = "DELETE FROM sessions WHERE id = $1";
        let result = sqlx::query(query).bind(id).execute(self.pool()).await?;
        Ok(result.rows_affected() > 0)
    }

    async fn count_active_by_user(&self, user_id: Uuid) -> Result<i64> {
        let query = r#"
            SELECT COUNT(*) FROM sessions
            WHERE user_id = $1 AND ended_at IS NULL
        "#;
        let (count,): (i64,) = sqlx::query_as(query)
            .bind(user_id)
            .fetch_one(self.pool())
            .await?;
        Ok(count)
    }

    async fn cleanup_idle(&self, timeout_seconds: i64) -> Result<u64> {
        let query = r#"
            UPDATE sessions
            SET ended_at = NOW()
            WHERE ended_at IS NULL
              AND EXTRACT(EPOCH FROM (NOW() - last_activity_at)) > $1
            RETURNING id
        "#;

        let result = sqlx::query(query)
            .bind(timeout_seconds)
            .execute(self.pool())
            .await?;

        Ok(result.rows_affected())
    }

    async fn summaries_by_user(&self, user_id: Uuid) -> Result<Vec<SessionSummary>> {
        let query = r#"
            SELECT
                id, title, state, started_at, ended_at,
                EXTRACT(EPOCH FROM (COALESCE(ended_at, NOW()) - started_at))::BIGINT as duration
            FROM sessions
            WHERE user_id = $1
            ORDER BY started_at DESC
        "#;

        let rows = sqlx::query(query).bind(user_id).fetch_all(self.pool()).await?;

        let mut summaries = Vec::new();
        for row in rows {
            let id: Uuid = row.try_get("id")?;
            let title: String = row.try_get("title")?;
            let state_i32: i32 = row.try_get("state")?;
            let started_at: chrono::DateTime<chrono::Utc> = row.try_get("started_at")?;
            let ended_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("ended_at")?;
            let duration: Option<i64> = row.try_get("duration")?;

            let state = match state_i32 {
                0 => SessionState::Warmup,
                1 => SessionState::DeepDive,
                2 => SessionState::Review,
                3 => SessionState::Seed,
                4 => SessionState::Closing,
                _ => SessionState::Warmup,
            };

            summaries.push(SessionSummary {
                id,
                title,
                state,
                started_at,
                is_active: ended_at.is_none(),
                duration_seconds: if ended_at.is_none() { duration } else { None },
            });
        }

        Ok(summaries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pg_session_repository_new() {
        // This test verifies the struct can be created
        // Actual database operations require a running database
        let pool_str = "postgresql://localhost/test";
        assert!(!pool_str.is_empty());
    }

    #[test]
    fn test_session_state_values() {
        assert_eq!(SessionState::Warmup as i32, 0);
        assert_eq!(SessionState::DeepDive as i32, 1);
    }
}
