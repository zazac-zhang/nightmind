// ============================================================
// Knowledge Repository
// ============================================================
//! Knowledge base data access layer.
//!
//! This module provides database operations for knowledge management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::Row;

/// Knowledge entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Knowledge {
    /// Unique knowledge identifier
    pub id: Uuid,
    /// Knowledge title
    pub title: String,
    /// Knowledge content
    pub content: String,
    /// Associated user ID
    pub user_id: Uuid,
    /// Tags for categorization (stored as JSONB in DB)
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Vector embedding for similarity search
    pub embedding: Option<Vec<f32>>,
}

/// Create knowledge request
#[derive(Debug, Clone)]
pub struct CreateKnowledgeRequest {
    /// Knowledge title
    pub title: String,
    /// Knowledge content
    pub content: String,
    /// User ID creating the knowledge
    pub user_id: Uuid,
    /// Optional tags
    pub tags: Vec<String>,
}

/// Update knowledge request
#[derive(Debug, Clone)]
pub struct UpdateKnowledgeRequest {
    /// Knowledge ID to update
    pub id: Uuid,
    /// New title (optional)
    pub title: Option<String>,
    /// New content (optional)
    pub content: Option<String>,
    /// New tags (optional)
    pub tags: Option<Vec<String>>,
}

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
    async fn create(&self, request: CreateKnowledgeRequest) -> Result<Knowledge, RepositoryError>;

    /// Gets a knowledge entry by ID
    ///
    /// # Errors
    ///
    /// Returns an error if not found
    async fn get_by_id(&self, id: Uuid) -> Result<Knowledge, RepositoryError>;

    /// Updates a knowledge entry
    ///
    /// # Errors
    ///
    /// Returns an error if update fails
    async fn update(&self, request: UpdateKnowledgeRequest) -> Result<Knowledge, RepositoryError>;

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
    ) -> Result<Vec<Knowledge>, RepositoryError>;

    /// Searches knowledge by query
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails
    async fn search(&self, query: KnowledgeSearchQuery) -> Result<Vec<Knowledge>, RepositoryError>;

    /// Updates the embedding for a knowledge entry
    ///
    /// # Errors
    ///
    /// Returns an error if update fails
    async fn update_embedding(
        &self,
        id: Uuid,
        embedding: Vec<f32>,
    ) -> Result<Knowledge, RepositoryError>;
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
    async fn create(&self, request: CreateKnowledgeRequest) -> Result<Knowledge, RepositoryError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let tags_json = serde_json::to_value(&request.tags).unwrap();

        let row = sqlx::query(
            r#"
            INSERT INTO knowledge (id, title, content, user_id, tags, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&request.title)
        .bind(&request.content)
        .bind(request.user_id)
        .bind(tags_json)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        let tags: Vec<String> = serde_json::from_value(row.get("tags")).unwrap_or_default();

        Ok(Knowledge {
            id: row.get("id"),
            title: row.get("title"),
            content: row.get("content"),
            user_id: row.get("user_id"),
            tags,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            embedding: None,
        })
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Knowledge, RepositoryError> {
        let row = sqlx::query("SELECT * FROM knowledge WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(id.to_string()))?;

        let tags: Vec<String> = serde_json::from_value(row.get("tags")).unwrap_or_default();

        Ok(Knowledge {
            id: row.get("id"),
            title: row.get("title"),
            content: row.get("content"),
            user_id: row.get("user_id"),
            tags,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            embedding: None,
        })
    }

    async fn update(&self, request: UpdateKnowledgeRequest) -> Result<Knowledge, RepositoryError> {
        let now = Utc::now();
        let tags_json = request.tags.map(|t| serde_json::to_value(t).unwrap());

        let row = sqlx::query(
            r#"
            UPDATE knowledge
            SET title = COALESCE($2, title),
                content = COALESCE($3, content),
                tags = COALESCE($4, tags),
                updated_at = $5
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(request.id)
        .bind(request.title)
        .bind(request.content)
        .bind(tags_json)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound(request.id.to_string()))?;

        let tags: Vec<String> = serde_json::from_value(row.get("tags")).unwrap_or_default();

        Ok(Knowledge {
            id: row.get("id"),
            title: row.get("title"),
            content: row.get("content"),
            user_id: row.get("user_id"),
            tags,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            embedding: None,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM knowledge WHERE id = $1")
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
    ) -> Result<Vec<Knowledge>, RepositoryError> {
        let offset = (page - 1) * limit;

        let rows = sqlx::query(
            "SELECT * FROM knowledge WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<Knowledge> = rows
            .iter()
            .map(|row| {
                let tags: Vec<String> =
                    serde_json::from_value(row.get("tags")).unwrap_or_default();
                Knowledge {
                    id: row.get("id"),
                    title: row.get("title"),
                    content: row.get("content"),
                    user_id: row.get("user_id"),
                    tags,
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    embedding: None,
                }
            })
            .collect();

        Ok(items)
    }

    async fn search(&self, query: KnowledgeSearchQuery) -> Result<Vec<Knowledge>, RepositoryError> {
        let offset = (query.page - 1) * query.limit;

        let results = if let Some(search_query) = query.query {
            let pattern = format!("%{search_query}%");
            sqlx::query(
                "SELECT * FROM knowledge WHERE user_id = $1 AND (title ILIKE $2 OR content ILIKE $2) ORDER BY created_at DESC LIMIT $3 OFFSET $4"
            )
            .bind(query.user_id)
            .bind(pattern)
            .bind(query.limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT * FROM knowledge WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
            )
            .bind(query.user_id)
            .bind(query.limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        };

        let items: Vec<Knowledge> = results
            .iter()
            .map(|row| {
                let tags: Vec<String> =
                    serde_json::from_value(row.get("tags")).unwrap_or_default();
                Knowledge {
                    id: row.get("id"),
                    title: row.get("title"),
                    content: row.get("content"),
                    user_id: row.get("user_id"),
                    tags,
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    embedding: None,
                }
            })
            .collect();

        Ok(items)
    }

    async fn update_embedding(
        &self,
        id: Uuid,
        embedding: Vec<f32>,
    ) -> Result<Knowledge, RepositoryError> {
        let row = sqlx::query(
            "UPDATE knowledge SET embedding = $2, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(&embedding as &[f32])
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound(id.to_string()))?;

        let tags: Vec<String> = serde_json::from_value(row.get("tags")).unwrap_or_default();

        Ok(Knowledge {
            id: row.get("id"),
            title: row.get("title"),
            content: row.get("content"),
            user_id: row.get("user_id"),
            tags,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            embedding: Some(embedding),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_creation() {
        let request = CreateKnowledgeRequest {
            title: "Test Knowledge".to_string(),
            content: "Test content".to_string(),
            user_id: Uuid::new_v4(),
            tags: vec!["test".to_string(), "example".to_string()],
        };

        assert_eq!(request.title, "Test Knowledge");
        assert_eq!(request.tags.len(), 2);
    }
}
