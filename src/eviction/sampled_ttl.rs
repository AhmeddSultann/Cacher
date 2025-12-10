use super::{EntryMetadata, EvictionPolicy};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// Sampled TTL eviction policy optimized for HFT
/// 
/// This implementation provides:
/// - Zero overhead on cache reads (no per-access tracking)
/// - Background sampling task that randomly checks keys for expiration
/// - No lock contention on the read path
/// - Configurable sample size and interval
/// 
/// This is significantly faster than LRU for high-frequency trading because:
/// 1. No linked list updates on every access
/// 2. No lock acquisition on reads
/// 3. Eviction happens asynchronously in the background
#[derive(Clone)]
pub struct SampledTtlEviction {
    /// Default TTL for entries without explicit TTL
    default_ttl_seconds: Option<u64>,
    
    /// Sample size per eviction cycle
    sample_size: usize,
    
    /// Interval between sampling cycles
    sample_interval: Duration,
}

impl SampledTtlEviction {
    /// Create a new sampled TTL eviction policy
    /// 
    /// # Arguments
    /// * `default_ttl_seconds` - Default TTL for entries (None = no expiration)
    /// * `sample_size` - Number of random keys to check per cycle
    /// * `sample_interval` - Duration between sampling cycles
    pub fn new(
        default_ttl_seconds: Option<u64>,
        sample_size: usize,
        sample_interval: Duration,
    ) -> Self {
        Self {
            default_ttl_seconds,
            sample_size,
            sample_interval,
        }
    }

    /// Create with sensible defaults for HFT
    /// - No default TTL (entries don't expire unless explicitly set)
    /// - Sample 100 keys per cycle
    /// - Run every 1 second
    pub fn default_hft() -> Self {
        Self::new(None, 100, Duration::from_secs(1))
    }

    /// Get the sample size
    pub fn sample_size(&self) -> usize {
        self.sample_size
    }

    /// Get the sample interval
    pub fn sample_interval(&self) -> Duration {
        self.sample_interval
    }
}

#[async_trait]
impl EvictionPolicy for SampledTtlEviction {
    fn on_insert(&self, ttl_seconds: Option<u64>) -> EntryMetadata {
        // Use explicit TTL if provided, otherwise use default
        let ttl = ttl_seconds.or(self.default_ttl_seconds);
        EntryMetadata::new(ttl)
    }

    fn should_evict(&self, metadata: &EntryMetadata) -> bool {
        metadata.is_expired()
    }

    async fn start_background_task(&self) {
        // Note: The actual background task will be started by the storage backend
        // because it needs access to the cache entries to sample from.
        // This method is here for trait completeness and future extensibility.
    }
}

/// Background eviction task
/// 
/// This function should be spawned as a tokio task by the storage backend.
/// It periodically samples random keys and evicts expired entries.
/// 
/// # Type Parameters
/// * `K` - Key type
/// * `F` - Function type for getting random keys
/// * `E` - Function type for evicting keys
/// 
/// # Arguments
/// * `policy` - The eviction policy
/// * `get_random_keys` - Function to get N random keys from the cache
/// * `evict_key` - Function to evict a specific key
pub async fn run_background_eviction<K, F, E, Fut1, Fut2>(
    policy: Arc<SampledTtlEviction>,
    get_random_keys: F,
    evict_key: E,
) where
    K: Send + 'static,
    F: Fn(usize) -> Fut1 + Send + Sync + 'static,
    E: Fn(K) -> Fut2 + Send + Sync + 'static,
    Fut1: std::future::Future<Output = Vec<(K, EntryMetadata)>> + Send,
    Fut2: std::future::Future<Output = ()> + Send,
{
    let mut interval = tokio::time::interval(policy.sample_interval());
    
    loop {
        interval.tick().await;
        
        // Get random sample of keys
        let samples = get_random_keys(policy.sample_size()).await;
        
        // Check each sampled entry for expiration
        for (key, metadata) in samples {
            if policy.should_evict(&metadata) {
                evict_key(key).await;
            }
        }
    }
}
