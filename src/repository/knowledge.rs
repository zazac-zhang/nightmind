// ============================================================
// Knowledge Repository
// ============================================================
//! Knowledge base data access layer.
//!
//! This module provides database operations for knowledge management.

use crate::repository::models::knowledge::{
    CreateKnowledgePoint, KnowledgePoint, UpdateKnowledgePoint,
};
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

/// Knowledge search query
#[derive(Debug, Clone)]
pub struct KnowledgeSearchQuery {
    /// User ID to search for
    pub user_id: Uuid,
    /// Search query string
    pub query: Option<String>,
    /// Tags to filter by
    pub tags: Option<Vec<String>>,
    /// Page number
    pub page: u32,
    /// Results per page
    pub limit: u32,
}

/// Knowledge repository trait
#[async_trait::async_trait]
pub trait KnowledgeRepository: Send + Sync {
    /// Creates a new knowledge entry
    ///
    /// # Errors
    ///
    /// Returns an error if creation fails
    async fn create(&self, request: CreateKnowledgePoint) -> Result<KnowledgePoint, RepositoryError>;

    /// Gets a knowledge entry by ID
    ///
    /// # Errors
    ///
    /// Returns an error if not found
    async fn get_by_id(&self, id: Uuid) -> Result<KnowledgePoint, RepositoryError>;

    /// Updates a knowledge entry
    ///
    /// # Errors
    ///
    /// Returns an error if update fails
    async fn update(&self, request: UpdateKnowledgePoint) -> Result<KnowledgePoint, RepositoryError>;

    /// Deletes a knowledge entry
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Lists knowledge for a user with pagination
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    async fn list_by_user(
        &self,
        user_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<Vec<KnowledgePoint>, RepositoryError>;

    /// Searches knowledge by query
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails
    async fn search(&self, query: KnowledgeSearchQuery) -> Result<Vec<KnowledgePoint>, RepositoryError>;

    /// Updates the embedding for a knowledge entry
    ///
    /// # Errors
    ///
    /// Returns an error if update fails
    async fn update_embedding(
        &self,
        id: Uuid,
        embedding: Vec<f32>,
    ) -> Result<KnowledgePoint, RepositoryError>;
}

/// Repository error type
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    /// Knowledge not found
    #[error("Knowledge not found: {0}")]
    NotFound(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Postgres knowledge repository implementation
pub struct PostgresKnowledgeRepository {
    /// Database pool
    pool: sqlx::PgPool,
}

impl PostgresKnowledgeRepository {
    /// Creates a new Postgres knowledge repository
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl KnowledgeRepository for PostgresKnowledgeRepository {
    async fn create(&self, request: CreateKnowledgePoint) -> Result<KnowledgePoint, RepositoryError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let tags_array = request.tags.as_slice();

        let row = sqlx::query(
            r#"
            INSERT INTO knowledge_points (
                id, user_id, content, content_type,
                source_type, source_id, title, summary, tags,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(request.user_id)
        .bind(&request.content)
        .bind(&request.content_type)
        .bind(&request.source_type)
        .bind(&request.source_id)
        .bind(&request.title)
        .bind(&request.summary)
        .bind(tags_array)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_knowledge_point(row))
    }

    async fn get_by_id(&self, id: Uuid) -> Result<KnowledgePoint, RepositoryError> {
        let row = sqlx::query("SELECT * FROM knowledge_points WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(id.to_string()))?;

        Ok(row_to_knowledge_point(row))
    }

    async fn update(&self, request: UpdateKnowledgePoint) -> Result<KnowledgePoint, RepositoryError> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            UPDATE knowledge_points
            SET content = COALESCE($2, content),
                content_type = COALESCE($3, content_type),
                title = COALESCE($4, title),
                summary = COALESCE($5, summary),
                tags = COALESCE($6, tags),
                updated_at = $7
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(request.id)
        .bind(request.content)
        .bind(request.content_type)
        .bind(request.title)
        .bind(request.summary)
        .bind(request.tags)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?
            .ok_or_else(|| RepositoryError::NotFound(request.id.to_string()))?;

        Ok(row_to_knowledge_point(row))
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM knowledge_points WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(id.to_string()));
        }

        Ok(())
    }

    async fn list_by_user(
        &self,
        user_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<Vec<KnowledgePoint>, RepositoryError> {
        let offset = (page - 1) * limit;

        let rows = sqlx::query(
            "SELECT * FROM knowledge_points WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<KnowledgePoint> = rows.into_iter().map(row_to_knowledge_point).collect();

        Ok(items)
    }

    async fn search(&self, query: KnowledgeSearchQuery) -> Result<Vec<KnowledgePoint>, RepositoryError> {
        let offset = (query.page - 1) * query.limit;

        let results = if let Some(search_query) = query.query {
            let pattern = format!("%{search_query}%");
            sqlx::query(
                "SELECT * FROM knowledge_points WHERE user_id = $1 AND (title ILIKE $2 OR content ILIKE $2) ORDER BY created_at DESC LIMIT $3 OFFSET $4"
            )
            .bind(query.user_id)
            .bind(pattern)
            .bind(query.limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT * FROM knowledge_points WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
            )
            .bind(query.user_id)
            .bind(query.limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        };

        let items: Vec<KnowledgePoint> = results.into_iter().map(row_to_knowledge_point).collect();

        Ok(items)
    }

    async fn update_embedding(
        &self,
        id: Uuid,
        embedding: Vec<f32>,
    ) -> Result<KnowledgePoint, RepositoryError> {
        // Note: pgvector embedding column would be updated here
        // For now, just update the timestamp
        let row = sqlx::query(
            "UPDATE knowledge_points SET updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(&embedding as &[f32])
        .fetch_optional(&self.pool)
        .await?
            .ok_or_else(|| RepositoryError::NotFound(id.to_string()))?;

        Ok(row_to_knowledge_point(row))
    }
}

/// Converts a database row to a KnowledgePoint
fn row_to_knowledge_point(row: sqlx::postgres::PgRow) -> KnowledgePoint {
    KnowledgePoint {
        id: row.get("id"),
        user_id: row.get("user_id"),
        content: row.get("content"),
        content_type: row.get("content_type"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        title: row.get("title"),
        summary: row.get("summary"),
        tags: row.try_get::<Vec<String>, _>("tags").unwrap_or_default(),
        parent_id: row.get("parent_id"),
        related_ids: row.try_get::<Vec<Uuid>, _>("related_ids").unwrap_or_default(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        last_reviewed_at: row.get("last_reviewed_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_creation() {
        let request = CreateKnowledgePoint {
            content: "Test content".to_string(),
            content_type: "concept".to_string(),
            source_type: "manual".to_string(),
            user_id: Uuid::new_v4(),
            title: Some("Test".to_string()),
            summary: None,
            tags: vec!["test".to_string()],
            source_id: None,
            parent_id: None,
        };

        assert_eq!(request.content, "Test content");
        assert_eq!(request.tags.len(), 1);
    }
}
