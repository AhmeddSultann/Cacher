use cacher::{ShardedCache, SampledTtlEviction};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the cache with HFT-optimized settings
    let eviction = SampledTtlEviction::new(
        None,                        // No default TTL
        100,                         // Sample 100 keys per cycle
        Duration::from_secs(1),      // Run every second
    );
    
    let cache = Arc::new(ShardedCache::new(10_000, eviction));
    
    // Server address
    let addr: SocketAddr = "0.0.0.0:50051".parse()?;
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  High-Performance Concurrent Caching Engine               ║");
    println!("║  Optimized for Statistical Arbitrage                      ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("⚡ Features:");
    println!("  • Lock-free concurrent reads (DashMap)");
    println!("  • Zero-copy serialization (rkyv)");
    println!("  • Sampled TTL eviction (no read overhead)");
    println!("  • Cache-padded metrics (prevents false sharing)");
    println!();
    
    // Start the gRPC server
    cacher::server::start_server(addr, cache).await?;
    
    Ok(())
}
