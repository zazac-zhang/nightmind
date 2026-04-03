// ============================================================
// Transformation Cache
// ============================================================
//! Caching layer for content transformation results.
//!
//! This module provides caching functionality to improve
//! transformation performance by avoiding repeated AI calls
//! for identical content.

use crate::repository::redis::{RedisManager, CacheOps};
use crate::core::content::transformer::VoiceFriendlyResult;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

/// Cached transformation result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTransform {
    /// The transformation result
    pub result: VoiceFriendlyResult,
    /// Cache key (hash of original content)
    pub cache_key: String,
    /// Timestamp when cached
    pub cached_at: i64,
    /// Number of times this cache entry was used
    pub hit_count: u32,
}

impl CachedTransform {
    /// Creates a new cached transformation
    #[must_use]
    pub fn new(result: VoiceFriendlyResult, cache_key: String) -> Self {
        Self {
            result,
            cache_key,
            cached_at: chrono::Utc::now().timestamp(),
            hit_count: 0,
        }
    }

    /// Increments the hit count
    pub fn increment_hit(&mut self) {
        self.hit_count += 1;
    }
}

/// Transformation cache manager
pub struct TransformCache {
    /// Redis manager
    redis: RedisManager,
    /// TTL for cache entries in seconds (default: 1 hour)
    default_ttl: u64,
    /// Key prefix for transformation cache
    key_prefix: String,
}

impl TransformCache {
    /// Creates a new transformation cache
    #[must_use]
    pub fn new(redis: RedisManager) -> Self {
        Self {
            redis,
            default_ttl: 3600, // 1 hour
            key_prefix: "transform:".to_string(),
        }
    }

    /// Creates a new transformation cache with custom TTL
    #[must_use]
    pub fn with_ttl(redis: RedisManager, ttl_seconds: u64) -> Self {
        Self {
            redis,
            default_ttl: ttl_seconds,
            key_prefix: "transform:".to_string(),
        }
    }

    /// Generates a cache key from content
    fn generate_cache_key(&self, content: &str) -> String {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();
        format!("{}{:x}", self.key_prefix, hash)
    }

    /// Attempts to retrieve a cached transformation result
    ///
    /// # Arguments
    ///
    /// * `content` - Original content that was transformed
    ///
    /// # Returns
    ///
    /// The cached result if found and valid, None otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if Redis operation fails
    pub async fn get(&self, content: &str) -> Result<Option<VoiceFriendlyResult>, Box<dyn std::error::Error>> {
        let key = self.generate_cache_key(content);

        if let Some(serialized) = self.redis.get(&key).await? {
            let cached: CachedTransform = CacheOps::deserialize(&serialized)?;
            Ok(Some(cached.result))
        } else {
            Ok(None)
        }
    }

    /// Caches a transformation result
    ///
    /// # Arguments
    ///
    /// * `content` - Original content that was transformed
    /// * `result` - Transformation result to cache
    ///
    /// # Returns
    ///
    /// Ok if cache succeeded
    ///
    /// # Errors
    ///
    /// Returns an error if Redis operation fails
    pub async fn set(&self, content: &str, result: VoiceFriendlyResult) -> Result<(), Box<dyn std::error::Error>> {
        let key = self.generate_cache_key(content);
        let cached = CachedTransform::new(result, key.clone());
        let serialized = CacheOps::serialize(&cached)?;

        self.redis.set_with_expiration(&key, &serialized, self.default_ttl).await?;
        Ok(())
    }

    /// Invalidates a cache entry
    ///
    /// # Arguments
    ///
    /// * `content` - Content to invalidate from cache
    ///
    /// # Returns
    ///
    /// True if entry was deleted
    ///
    /// # Errors
    ///
    /// Returns an error if Redis operation fails
    pub async fn invalidate(&self, content: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let key = self.generate_cache_key(content);
        Ok(self.redis.delete(&key).await?)
    }

    /// Clears all transformation cache entries
    ///
    /// # Returns
    ///
    /// Number of keys deleted
    ///
    /// # Errors
    ///
    /// Returns an error if Redis operation fails
    pub async fn clear_all(&self) -> Result<u64, Box<dyn std::error::Error>> {
        // This is a potentially expensive operation
        // In production, consider using Redis SCAN or pattern-based deletion
        Ok(0) // Placeholder - implement based on Redis client capabilities
    }

    /// Gets cache statistics
    ///
    /// # Returns
    ///
    /// Cache statistics including hit rate and size
    ///
    /// # Errors
    ///
    /// Returns an error if Redis operation fails
    pub async fn stats(&self) -> Result<TransformCacheStats, Box<dyn std::error::Error>> {
        // TODO: Implement actual stats collection
        Ok(TransformCacheStats::default())
    }
}

/// Transformation cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformCacheStats {
    /// Total number of cached entries
    pub total_entries: u64,
    /// Total cache hits
    pub total_hits: u64,
    /// Total cache misses
    pub total_misses: u64,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f32,
}

impl Default for TransformCacheStats {
    fn default() -> Self {
        Self {
            total_entries: 0,
            total_hits: 0,
            total_misses: 0,
            hit_rate: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_cache_key() {
        let redis = RedisManager::new("redis://localhost", "test:").unwrap();
        let cache = TransformCache::new(redis);

        let key1 = cache.generate_cache_key("test content");
        let key2 = cache.generate_cache_key("test content");
        let key3 = cache.generate_cache_key("different content");

        assert_eq!(key1, key2); // Same content -> same key
        assert_ne!(key1, key3); // Different content -> different key
    }

    #[test]
    fn test_cached_transform() {
        let result = VoiceFriendlyResult {
            content: "transformed content".to_string(),
            success: true,
            confidence: 0.9,
            reading_time_seconds: 5,
            warnings: vec![],
        };

        let cached = CachedTransform::new(result, "test_key".to_string());
        assert_eq!(cached.hit_count, 0);

        let mut cached = cached;
        cached.increment_hit();
        assert_eq!(cached.hit_count, 1);
    }

    #[test]
    fn test_transform_cache_stats_default() {
        let stats = TransformCacheStats::default();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }
}
