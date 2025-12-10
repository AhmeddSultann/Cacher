pub mod sampled_ttl;

pub use sampled_ttl::SampledTtlEviction;

use async_trait::async_trait;
use std::time::SystemTime;

/// Entry metadata for eviction tracking
#[derive(Debug, Clone)]
pub struct EntryMetadata {
    /// When the entry was inserted
    pub inserted_at: SystemTime,
    
    /// Optional TTL in seconds
    pub ttl_seconds: Option<u64>,
}

impl EntryMetadata {
    /// Create new metadata for an entry
    pub fn new(ttl_seconds: Option<u64>) -> Self {
        Self {
            inserted_at: SystemTime::now(),
            ttl_seconds,
        }
    }

    /// Check if this entry has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            if let Ok(elapsed) = self.inserted_at.elapsed() {
                return elapsed.as_secs() >= ttl;
            }
        }
        false
    }
}

/// Eviction policy trait for cache implementations
/// 
/// This trait supports background eviction strategies that don't require
/// per-access tracking, which is critical for HFT performance.
#[async_trait]
pub trait EvictionPolicy: Send + Sync {
    /// Called when a new entry is inserted
    /// Returns metadata to store with the entry
    fn on_insert(&self, ttl_seconds: Option<u64>) -> EntryMetadata;

    /// Check if an entry should be evicted based on its metadata
    fn should_evict(&self, metadata: &EntryMetadata) -> bool;

    /// Start background eviction task
    /// This should spawn a tokio task that periodically checks for expired entries
    async fn start_background_task(&self);
}
