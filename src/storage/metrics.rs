use crossbeam::utils::CachePadded;
use std::sync::atomic::{AtomicU64, Ordering};

/// Cache metrics with cache-line padding to prevent false sharing
/// 
/// Each atomic counter is padded to occupy a separate cache line (typically 64 bytes).
/// This prevents false sharing on multi-core systems, which is critical for HFT performance.
/// Without padding, different cores writing to adjacent counters would invalidate each
/// other's cache lines, causing severe performance degradation.
#[derive(Debug)]
pub struct CacheMetrics {
    /// Total cache hits (successful gets)
    hits: CachePadded<AtomicU64>,
    
    /// Total cache misses (failed gets)
    misses: CachePadded<AtomicU64>,
    
    /// Total set operations
    sets: CachePadded<AtomicU64>,
    
    /// Total delete operations
    deletes: CachePadded<AtomicU64>,
    
    /// Total evictions
    evictions: CachePadded<AtomicU64>,
}

impl CacheMetrics {
    /// Create new metrics instance with all counters at zero
    pub fn new() -> Self {
        Self {
            hits: CachePadded::new(AtomicU64::new(0)),
            misses: CachePadded::new(AtomicU64::new(0)),
            sets: CachePadded::new(AtomicU64::new(0)),
            deletes: CachePadded::new(AtomicU64::new(0)),
            evictions: CachePadded::new(AtomicU64::new(0)),
        }
    }

    /// Record a cache hit
    #[inline]
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    #[inline]
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a set operation
    #[inline]
    pub fn record_set(&self) {
        self.sets.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a delete operation
    #[inline]
    pub fn record_delete(&self) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an eviction
    #[inline]
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Get total hits
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Get total misses
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Get total sets
    pub fn sets(&self) -> u64 {
        self.sets.load(Ordering::Relaxed)
    }

    /// Get total deletes
    pub fn deletes(&self) -> u64 {
        self.deletes.load(Ordering::Relaxed)
    }

    /// Get total evictions
    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }

    /// Calculate hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits();
        let total = hits + self.misses();
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            hits: self.hits(),
            misses: self.misses(),
            sets: self.sets(),
            deletes: self.deletes(),
            evictions: self.evictions(),
            hit_rate: self.hit_rate(),
        }
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of cache metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub deletes: u64,
    pub evictions: u64,
    pub hit_rate: f64,
}
