// ============================================================
// Redis Repository
// ============================================================
//! Redis connection and caching operations.
//!
//! This module provides Redis connectivity for caching and
//! session management with connection pooling support.

use crate::config::Settings;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Redis connection manager with connection pooling
#[derive(Clone)]
pub struct RedisManager {
    /// Redis client
    client: Arc<Client>,
    /// Key prefix for namespacing
    key_prefix: String,
}

impl RedisManager {
    /// Creates a new Redis manager from application settings
    ///
    /// This is the recommended way to create a Redis manager in production.
    ///
    /// # Arguments
    ///
    /// * `settings` - Application configuration
    ///
    /// # Returns
    ///
    /// A Redis manager or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis URL is invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nightmind::{config::Settings, repository::redis::RedisManager};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let settings = Settings::load()?;
    /// let redis = RedisManager::from_settings(&settings)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_settings(settings: &Settings) -> Result<Self, RedisError> {
        Self::new(&settings.redis.url, &settings.redis.key_prefix)
    }

    /// Creates a new Redis manager
    ///
    /// # Arguments
    ///
    /// * `url` - Redis connection URL
    /// * `key_prefix` - Prefix for all keys
    ///
    /// # Returns
    ///
    /// A Redis manager or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid
    pub fn new(url: &str, key_prefix: &str) -> Result<Self, RedisError> {
        let client = Client::open(url)?;
        Ok(Self {
            client: Arc::new(client),
            key_prefix: key_prefix.to_string(),
        })
    }

    /// Gets a multiplexed async connection
    ///
    /// This is the recommended way to get connections as it supports
    /// connection pooling and multiplexing.
    ///
    /// # Errors
    ///
    /// Returns an error if connection fails
    pub async fn get_connection(&self) -> Result<MultiplexedConnection, RedisError> {
        self.client.get_multiplexed_async_connection().await
    }

    /// Builds a full key with the configured prefix
    #[must_use]
    pub fn build_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }

    /// Sets a value with expiration
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - Value to store
    /// * `ttl_seconds` - Time to live in seconds
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn set_with_expiration(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: u64,
    ) -> Result<(), RedisError> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);
        conn.set_ex(full_key, value, ttl_seconds).await
    }

    /// Sets a value without expiration
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - Value to store
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn set(&self, key: &str, value: &str) -> Result<(), RedisError> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);
        conn.set(full_key, value).await
    }

    /// Gets a value by key
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    ///
    /// # Returns
    ///
    /// The cached value or None if not found
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn get(&self, key: &str) -> Result<Option<String>, RedisError> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);
        conn.get(full_key).await
    }

    /// Gets multiple keys at once
    ///
    /// # Arguments
    ///
    /// * `keys` - Keys to retrieve
    ///
    /// # Returns
    ///
    /// A vector of values (None for missing keys)
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<String>>, RedisError> {
        let mut conn = self.get_connection().await?;
        let full_keys: Vec<String> = keys.iter().map(|k| self.build_key(k)).collect();
        let values: Vec<Option<String>> = conn.mget(&full_keys).await?;
        Ok(values)
    }

    /// Sets multiple key-value pairs at once
    ///
    /// # Arguments
    ///
    /// * `pairs` - Key-value pairs to set
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn mset(&self, pairs: &[(String, String)]) -> Result<(), RedisError> {
        let mut conn = self.get_connection().await?;
        let full_pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| (self.build_key(k), v.clone()))
            .collect();
        conn.mset(&full_pairs).await
    }

    /// Deletes a key
    ///
    /// # Arguments
    ///
    /// * `key` - Key to delete
    ///
    /// # Returns
    ///
    /// True if the key was deleted
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn delete(&self, key: &str) -> Result<bool, RedisError> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);
        conn.del(full_key).await.map(|count: u64| count > 0)
    }

    /// Deletes multiple keys
    ///
    /// # Arguments
    ///
    /// * `keys` - Keys to delete
    ///
    /// # Returns
    ///
    /// Number of keys deleted
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn mdelete(&self, keys: &[&str]) -> Result<u64, RedisError> {
        let mut conn = self.get_connection().await?;
        let full_keys: Vec<String> = keys.iter().map(|k| self.build_key(k)).collect();
        conn.del(full_keys).await
    }

    /// Checks if a key exists
    ///
    /// # Arguments
    ///
    /// * `key` - Key to check
    ///
    /// # Returns
    ///
    /// True if the key exists
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn exists(&self, key: &str) -> Result<bool, RedisError> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);
        conn.exists(full_key).await
    }

    /// Sets the expiration time for a key
    ///
    /// # Arguments
    ///
    /// * `key` - Key to set expiration on
    /// * `ttl_seconds` - Time to live in seconds
    ///
    /// # Returns
    ///
    /// True if the timeout was set
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<bool, RedisError> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);
        conn.expire(full_key, ttl_seconds as i64).await
    }

    /// Gets the remaining time to live for a key
    ///
    /// # Arguments
    ///
    /// * `key` - Key to check
    ///
    /// # Returns
    ///
    /// Time to live in seconds, or None if key doesn't exist or has no expiration
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn ttl(&self, key: &str) -> Result<Option<i64>, RedisError> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);
        conn.ttl(full_key).await.map(|ttl| {
            if ttl < 0 {
                None
            } else {
                Some(ttl)
            }
        })
    }

    /// Increments a value
    ///
    /// # Arguments
    ///
    /// * `key` - Key to increment
    /// * `delta` - Amount to increment by
    ///
    /// # Returns
    ///
    /// The new value
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn incr_by(&self, key: &str, delta: i64) -> Result<i64, RedisError> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);
        conn.incr(full_key, delta).await
    }

    /// Adds a member to a sorted set
    ///
    /// # Arguments
    ///
    /// * `key` - Sorted set key
    /// * `member` - Member to add
    /// * `score` - Member's score
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn zadd(&self, key: &str, member: &str, score: i64) -> Result<(), RedisError> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);
        conn.zadd(full_key, member, score).await
    }

    /// Gets members from a sorted set by score range
    ///
    /// # Arguments
    ///
    /// * `key` - Sorted set key
    /// * `min` - Minimum score
    /// * `max` - Maximum score
    ///
    /// # Returns
    ///
    /// A vector of members in the score range
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn zrange_by_score(
        &self,
        key: &str,
        min: f64,
        max: f64,
    ) -> Result<Vec<String>, RedisError> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);
        conn.zrangebyscore(full_key, min, max).await
    }

    /// Performs a health check on the Redis connection
    ///
    /// # Errors
    ///
    /// Returns an error if Redis is not accessible
    pub async fn health_check(&self) -> Result<(), RedisError> {
        let mut conn = self.get_connection().await?;
        redis::cmd("PING").query_async(&mut conn).await
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// Cached value
    pub value: T,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Time to live in seconds
    pub ttl: Option<u64>,
}

impl<T> CacheEntry<T> {
    /// Creates a new cache entry
    #[must_use]
    pub fn new(value: T, ttl: Option<u64>) -> Self {
        Self {
            value,
            created_at: chrono::Utc::now(),
            ttl,
        }
    }
}

/// Generic cache operations
pub struct CacheOps;

impl CacheOps {
    /// Serializes a value for storage
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    pub fn serialize<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
        serde_json::to_string(value)
    }

    /// Deserializes a value from storage
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    pub fn deserialize<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Session cache operations
pub struct SessionCache {
    /// Redis manager
    redis: RedisManager,
}

impl SessionCache {
    /// Creates a new session cache
    #[must_use]
    pub fn new(redis: RedisManager) -> Self {
        Self { redis }
    }

    /// Creates a new session cache from settings
    ///
    /// # Errors
    ///
    /// Returns an error if Redis configuration is invalid
    pub fn from_settings(settings: &Settings) -> Result<Self, RedisError> {
        Ok(Self {
            redis: RedisManager::from_settings(settings)?,
        })
    }

    /// Stores session data
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier
    /// * `data` - Session data to cache
    /// * `ttl_seconds` - Time to live
    ///
    /// # Errors
    ///
    /// Returns an error if storage fails
    pub async fn store_session<T: Serialize>(
        &self,
        session_id: uuid::Uuid,
        data: &T,
        ttl_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("session:{}", session_id);
        let serialized = CacheOps::serialize(data)?;
        self.redis.set_with_expiration(&key, &serialized, ttl_seconds).await?;
        Ok(())
    }

    /// Retrieves session data
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier
    ///
    /// # Returns
    ///
    /// The cached session data or None if not found
    ///
    /// # Errors
    ///
    /// Returns an error if retrieval fails
    pub async fn get_session<T: for<'de> Deserialize<'de>>(
        &self,
        session_id: uuid::Uuid,
    ) -> Result<Option<T>, Box<dyn std::error::Error>> {
        let key = format!("session:{}", session_id);
        if let Some(serialized) = self.redis.get(&key).await? {
            let deserialized = CacheOps::deserialize(&serialized)?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }

    /// Deletes a session from cache
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails
    pub async fn delete_session(&self, session_id: uuid::Uuid) -> Result<bool, RedisError> {
        let key = format!("session:{}", session_id);
        self.redis.delete(&key).await
    }

    /// Extends the TTL of a session
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier
    /// * `ttl_seconds` - New TTL in seconds
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn extend_session(
        &self,
        session_id: uuid::Uuid,
        ttl_seconds: u64,
    ) -> Result<bool, RedisError> {
        let key = format!("session:{}", session_id);
        self.redis.expire(&key, ttl_seconds).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry() {
        let entry = CacheEntry::new("test value", Some(3600));
        assert_eq!(entry.value, "test value");
        assert_eq!(entry.ttl, Some(3600));
    }

    #[test]
    fn test_cache_ops_serialization() {
        let value = serde_json::json!({"key": "value"});
        let serialized = CacheOps::serialize(&value);
        assert!(serialized.is_ok());

        let deserialized: serde_json::Value = CacheOps::deserialize(&serialized.unwrap()).unwrap();
        assert_eq!(deserialized, value);
    }

    #[test]
    fn test_redis_manager_build_key() {
        let manager = RedisManager::new("redis://localhost", "test:").unwrap();
        assert_eq!(manager.build_key("mykey"), "test:mykey");
    }

    #[test]
    fn test_cache_entry_without_ttl() {
        let entry = CacheEntry::new("value", None);
        assert!(entry.ttl.is_none());
    }
}
