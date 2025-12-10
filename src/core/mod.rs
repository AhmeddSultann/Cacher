pub mod error;

pub use error::{CacheError, CacheResult};

use async_trait::async_trait;

/// Generic cache trait supporting async operations
/// 
/// This trait provides the core interface for any cache implementation.
/// All methods return Results - no unwrap() calls in production code.
#[async_trait]
pub trait Cache<K, V>: Send + Sync
where
    K: Send + Sync,
    V: Send + Sync,
{
    /// Retrieve a value by key
    async fn get(&self, key: &K) -> CacheResult<Option<V>>;

    /// Insert or update a key-value pair
    /// Returns the previous value if it existed
    async fn set(&self, key: K, value: V) -> CacheResult<Option<V>>;

    /// Insert with TTL (time-to-live in seconds)
    /// Use 0 for no expiration
    async fn set_with_ttl(&self, key: K, value: V, ttl_seconds: u64) -> CacheResult<Option<V>>;

    /// Remove a key-value pair
    /// Returns the value if it existed
    async fn delete(&self, key: &K) -> CacheResult<Option<V>>;

    /// Check if a key exists
    async fn contains_key(&self, key: &K) -> CacheResult<bool>;

    /// Get the number of entries
    async fn len(&self) -> CacheResult<usize>;

    /// Check if the cache is empty
    async fn is_empty(&self) -> CacheResult<bool> {
        Ok(self.len().await? == 0)
    }

    /// Clear all entries
    async fn clear(&self) -> CacheResult<usize>;
}
