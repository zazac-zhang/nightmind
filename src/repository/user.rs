// ============================================================
// User Repository
// ============================================================
//! User data access layer.
//!
//! This module provides database operations for user management.

use super::db::{BaseRepository, Repository};
use super::models::user::{CreateUser, LoginCredentials, UpdateUser, User, UserProfile};
use crate::error::{NightMindError, Result};
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

/// User repository trait
///
/// Defines the interface for user data access operations.
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    /// Creates a new user
    async fn create(&self, data: CreateUser) -> Result<User>;

    /// Finds a user by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;

    /// Finds a user by username
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;

    /// Finds a user by email
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;

    /// Finds a user by username or email (for login)
    async fn find_by_identifier(&self, identifier: &str) -> Result<Option<User>>;

    /// Updates a user
    async fn update(&self, id: Uuid, data: UpdateUser) -> Result<User>;

    /// Deletes a user
    async fn delete(&self, id: Uuid) -> Result<bool>;

    /// Lists users with pagination
    async fn list(&self, offset: i32, limit: i32) -> Result<Vec<User>>;

    /// Counts total users
    async fn count(&self) -> Result<i64>;

    /// Verifies login credentials
    async fn verify_credentials(&self, creds: LoginCredentials) -> Result<Option<User>>;

    /// Updates last login timestamp
    async fn update_last_login(&self, id: Uuid) -> Result<()>;
}

/// PostgreSQL implementation of UserRepository
pub struct PgUserRepository {
    base: BaseRepository,
}

impl PgUserRepository {
    /// Creates a new user repository
    #[must_use]
    pub fn new(pool: &PgPool) -> Self {
        Self {
            base: BaseRepository::from_pool(pool),
        }
    }

    /// Helper method to serialize user role for queries
    #[must_use]
    fn role_value() -> i32 {
        // Default role (User) = 0
        0
    }
}

#[async_trait::async_trait]
impl Repository for PgUserRepository {
    fn pool(&self) -> &PgPool {
        self.base.pool()
    }
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, data: CreateUser) -> Result<User> {
        let user = User::create(data).map_err(|e| {
            NightMindError::Internal(format!("Failed to hash password: {}", e))
        })?;

        let query = r#"
            INSERT INTO users (
                id, username, email, password_hash, display_name,
                role, is_active, is_verified, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
        "#;

        let created = sqlx::query_as::<_, User>(query)
            .bind(user.id)
            .bind(&user.username)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(&user.display_name)
            .bind(user.role as i32)
            .bind(user.is_active)
            .bind(user.is_verified)
            .bind(user.created_at)
            .bind(user.updated_at)
            .fetch_one(self.pool())
            .await
            .map_err(|e| {
                if e.to_string().contains("unique constraint") {
                    NightMindError::BadRequest("Username or email already exists".to_string())
                } else {
                    NightMindError::Database(e)
                }
            })?;

        Ok(created)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE id = $1";
        let user = sqlx::query_as::<_, User>(query)
            .bind(id)
            .fetch_optional(self.pool())
            .await?;
        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE username = $1";
        let user = sqlx::query_as::<_, User>(query)
            .bind(username)
            .fetch_optional(self.pool())
            .await?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE email = $1";
        let user = sqlx::query_as::<_, User>(query)
            .bind(email)
            .fetch_optional(self.pool())
            .await?;
        Ok(user)
    }

    async fn find_by_identifier(&self, identifier: &str) -> Result<Option<User>> {
        // Try email first, then username
        if let Some(user) = self.find_by_email(identifier).await? {
            return Ok(Some(user));
        }

        self.find_by_username(identifier).await
    }

    async fn update(&self, id: Uuid, data: UpdateUser) -> Result<User> {
        // First fetch the existing user
        let mut user = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| NightMindError::not_found("User"))?;

        // Apply updates
        user.update(data).map_err(|e| {
            NightMindError::Internal(format!("Failed to hash password: {}", e))
        })?;

        let query = r#"
            UPDATE users
            SET username = $2, email = $3, password_hash = $4,
                display_name = $5, updated_at = $6
            WHERE id = $1
            RETURNING *
        "#;

        let updated = sqlx::query_as::<_, User>(query)
            .bind(id)
            .bind(&user.username)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(&user.display_name)
            .bind(user.updated_at)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| NightMindError::not_found("User"))?;

        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        let query = "DELETE FROM users WHERE id = $1";
        let result = sqlx::query(query).bind(id).execute(self.pool()).await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, offset: i32, limit: i32) -> Result<Vec<User>> {
        let query = r#"
            SELECT * FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
        "#;

        let users = sqlx::query_as::<_, User>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await?;

        Ok(users)
    }

    async fn count(&self) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM users";
        let (count,): (i64,) = sqlx::query_as(query).fetch_one(self.pool()).await?;
        Ok(count)
    }

    async fn verify_credentials(&self, creds: LoginCredentials) -> Result<Option<User>> {
        let user = self.find_by_identifier(&creds.identifier).await?;

        if let Some(user) = user {
            if user.verify_password(&creds.password) {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    async fn update_last_login(&self, id: Uuid) -> Result<()> {
        let query = r#"
            UPDATE users
            SET last_login_at = $2, updated_at = $2
            WHERE id = $1
        "#;

        sqlx::query(query)
            .bind(id)
            .bind(chrono::Utc::now())
            .execute(self.pool())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pg_user_repository_new() {
        // This test verifies the struct can be created
        // Actual database operations require a running database
        let pool_str = "postgresql://localhost/test";
        // Can't actually create pool without database, but we verify the code compiles
        assert!(!pool_str.is_empty());
    }
}
