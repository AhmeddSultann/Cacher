use crate::core::{Cache, CacheResult};
use crate::eviction::{EntryMetadata, EvictionPolicy, SampledTtlEviction};
use crate::storage::metrics::CacheMetrics;
use async_trait::async_trait;
use dashmap::DashMap;
use rand::seq::IteratorRandom;
use std::hash::Hash;
use std::sync::Arc;

/// Entry stored in the cache with metadata
#[derive(Clone)]
struct CacheEntry<V> {
    value: V,
    metadata: EntryMetadata,
}

/// High-performance sharded cache using DashMap
/// 
/// This implementation provides:
/// - Lock-free reads via DashMap's concurrent HashMap
/// - Automatic sharding for parallel writes
/// - Zero-overhead eviction via background sampling
/// - Cache-padded metrics to prevent false sharing
/// 
/// Perfect for HFT scenarios where read latency and throughput are critical.
pub struct ShardedCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// The underlying concurrent map
    map: Arc<DashMap<K, CacheEntry<V>>>,
    
    /// Eviction policy
    eviction: Arc<SampledTtlEviction>,
    
    /// Performance metrics
    metrics: Arc<CacheMetrics>,
}

impl<K, V> ShardedCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new sharded cache with the given capacity hint
    /// 
    /// # Arguments
    /// * `_capacity` - Hint for initial capacity (DashMap will handle sharding)
    /// * `eviction` - Eviction policy to use
    pub fn new(_capacity: usize, eviction: SampledTtlEviction) -> Self {
        let map = Arc::new(DashMap::new());
        let eviction = Arc::new(eviction);
        let metrics = Arc::new(CacheMetrics::new());

        let cache = Self {
            map,
            eviction,
            metrics,
        };

        // Start background eviction task
        cache.start_eviction_task();

        cache
    }

    /// Create with default HFT settings
    pub fn new_hft(capacity: usize) -> Self {
        Self::new(capacity, SampledTtlEviction::default_hft())
    }

    /// Start the background eviction task
    fn start_eviction_task(&self) {
        let map = Arc::clone(&self.map);
        let eviction = Arc::clone(&self.eviction);
        let metrics = Arc::clone(&self.metrics);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(eviction.sample_interval());

            loop {
                interval.tick().await;

                // Sample random keys
                let samples: Vec<_> = {
                    let mut rng = rand::thread_rng();
                    map.iter()
                        .map(|entry| (entry.key().clone(), entry.value().metadata.clone()))
                        .choose_multiple(&mut rng, eviction.sample_size())
                };

                // Evict expired entries
                for (key, metadata) in samples {
                    if eviction.should_evict(&metadata) {
                        if map.remove(&key).is_some() {
                            metrics.record_eviction();
                        }
                    }
                }
            }
        });
    }

    /// Get metrics snapshot
    pub fn metrics(&self) -> Arc<CacheMetrics> {
        Arc::clone(&self.metrics)
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for ShardedCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    async fn get(&self, key: &K) -> CacheResult<Option<V>> {
        match self.map.get(key) {
            Some(entry) => {
                // Check if expired
                if self.eviction.should_evict(&entry.metadata) {
                    // Remove expired entry
                    drop(entry); // Release the read lock
                    self.map.remove(key);
                    self.metrics.record_miss();
                    self.metrics.record_eviction();
                    Ok(None)
                } else {
                    self.metrics.record_hit();
                    Ok(Some(entry.value.clone()))
                }
            }
            None => {
                self.metrics.record_miss();
                Ok(None)
            }
        }
    }

    async fn set(&self, key: K, value: V) -> CacheResult<Option<V>> {
        self.set_with_ttl(key, value, 0).await
    }

    async fn set_with_ttl(&self, key: K, value: V, ttl_seconds: u64) -> CacheResult<Option<V>> {
        let ttl = if ttl_seconds == 0 { None } else { Some(ttl_seconds) };
        let metadata = self.eviction.on_insert(ttl);
        let entry = CacheEntry { value, metadata };
        
        self.metrics.record_set();
        
        let previous = self.map.insert(key, entry);
        Ok(previous.map(|e| e.value))
    }

    async fn delete(&self, key: &K) -> CacheResult<Option<V>> {
        self.metrics.record_delete();
        
        match self.map.remove(key) {
            Some((_, entry)) => Ok(Some(entry.value)),
            None => Ok(None),
        }
    }

    async fn contains_key(&self, key: &K) -> CacheResult<bool> {
        if let Some(entry) = self.map.get(key) {
            // Check if expired
            if self.eviction.should_evict(&entry.metadata) {
                drop(entry);
                self.map.remove(key);
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    async fn len(&self) -> CacheResult<usize> {
        Ok(self.map.len())
    }

    async fn clear(&self) -> CacheResult<usize> {
        let count = self.map.len();
        self.map.clear();
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_basic_operations() {
        let cache = ShardedCache::new_hft(100);

        // Set and get
        cache.set("key1".to_string(), "value1".to_string()).await.unwrap();
        let result = cache.get(&"key1".to_string()).await.unwrap();
        assert_eq!(result, Some("value1".to_string()));

        // Delete
        let deleted = cache.delete(&"key1".to_string()).await.unwrap();
        assert_eq!(deleted, Some("value1".to_string()));

        // Get after delete
        let result = cache.get(&"key1".to_string()).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = ShardedCache::new(
            100,
            SampledTtlEviction::new(None, 10, Duration::from_millis(100)),
        );

        // Set with 1 second TTL
        cache.set_with_ttl("key1".to_string(), "value1".to_string(), 1).await.unwrap();

        // Should exist immediately
        let result = cache.get(&"key1".to_string()).await.unwrap();
        assert_eq!(result, Some("value1".to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should be expired
        let result = cache.get(&"key1".to_string()).await.unwrap();
        assert_eq!(result, None);
    }
}
