# Cacher - High-Performance Concurrent Caching Engine

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A production-grade, high-performance concurrent caching system designed for statistical arbitrage and high-frequency trading (HFT) applications.

## 🚀 Features

### Performance Optimizations
- **Lock-Free Reads**: Uses `DashMap` for concurrent access with zero lock contention on reads
- **Zero-Copy Serialization**: `rkyv` support for ultra-low latency deserialization
- **Cache-Padded Metrics**: Prevents false sharing on CPU cache lines using `crossbeam::CachePadded`
- **Native CPU Optimization**: Compiled with `-C target-cpu=native` for maximum throughput

### Eviction Strategy
- **Sampled TTL**: Background task randomly samples keys for expiration
- **Zero Read Overhead**: No per-access tracking means reads are never blocked by eviction
- **Configurable**: Adjust sample size and interval for your workload

### Architecture
- **Sharded Storage**: Automatic partitioning for parallel writes
- **Async-First**: Built on `tokio` multithreaded runtime
- **gRPC Interface**: Low-latency `tonic` server with batch operations
- **Production-Ready**: Strict error handling, no `unwrap()` in production code

## 📦 Installation

### Prerequisites
- Rust 1.75+ (2021 edition)
- Protocol Buffers compiler (`protoc`)

```bash
# Install protoc on macOS
brew install protobuf

# Install protoc on Ubuntu/Debian
apt-get install -y protobuf-compiler
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/cacher.git
cd cacher

# Build in release mode
cargo build --release

# Run the server
./target/release/cacher
```

## 🐳 Docker Deployment

Build and run using the provided multi-stage Dockerfile:

```bash
# Build the image
docker build -t cacher:latest .

# Run the container
docker run -p 50051:50051 cacher:latest

# Or with docker-compose
docker-compose up -d
```

The Docker image uses:
- Multi-stage builds for optimal caching
- Distroless base image (minimal attack surface)
- Non-root user (UID 65532)
- ~20MB final image size

## 🔧 Configuration

### Cache Settings

```rust
use cacher::{ShardedCache, SampledTtlEviction};
use std::time::Duration;

// Create eviction policy
let eviction = SampledTtlEviction::new(
    None,                       // Default TTL (None = no expiration)
    100,                        // Sample 100 keys per cycle
    Duration::from_secs(1),     // Run every 1 second
);

// Create cache
let cache = ShardedCache::new(10_000, eviction);
```

### Server Settings

Default address: `0.0.0.0:50051`

Modify in `src/main.rs`:
```rust
let addr: SocketAddr = "0.0.0.0:50051".parse()?;
```

## 📡 gRPC API

### Service Definition

```protobuf
service CacheService {
  rpc Get(GetRequest) returns (GetResponse);
  rpc Set(SetRequest) returns (SetResponse);
  rpc Delete(DeleteRequest) returns (DeleteResponse);
  rpc Clear(ClearRequest) returns (ClearResponse);
  rpc BatchGet(BatchGetRequest) returns (BatchGetResponse);
  rpc BatchSet(BatchSetRequest) returns (BatchSetResponse);
  rpc Contains(ContainsRequest) returns (ContainsResponse);
  rpc Stats(StatsRequest) returns (StatsResponse);
}
```

### Example Client (grpcurl)

```bash
# Set a value with 60 second TTL
grpcurl -plaintext -d '{
  "key": "dGVzdC1rZXk=",
  "value": "dGVzdC12YWx1ZQ==",
  "ttl_seconds": 60
}' localhost:50051 cache.CacheService/Set

# Get a value
grpcurl -plaintext -d '{
  "key": "dGVzdC1rZXk="
}' localhost:50051 cache.CacheService/Get

# Get statistics
grpcurl -plaintext localhost:50051 cache.CacheService/Stats
```

## 🏗️ Architecture

```
cacher/
├── src/
│   ├── core/           # Generic Cache trait and errors
│   ├── storage/        # Sharded DashMap implementation
│   │   ├── sharded.rs  # Lock-free concurrent cache
│   │   └── metrics.rs  # Cache-padded atomic counters
│   ├── eviction/       # Eviction policies
│   │   └── sampled_ttl.rs  # Background TTL sampler
│   ├── server/         # gRPC service layer
│   └── main.rs         # Application entry point
├── proto/
│   └── cache.proto     # Protocol buffer definitions
└── Dockerfile          # Multi-stage production build
```

## ⚡ Performance Characteristics

### Concurrency Model
- **Reads**: Lock-free, O(1) average case
- **Writes**: Sharded locks, parallel writes to different shards
- **Eviction**: Asynchronous background task, zero impact on reads

### Memory Layout
- Each metric counter padded to 64 bytes (typical cache line)
- Prevents false sharing on multi-core systems
- Critical for HFT performance

### Benchmarks

_(Run with `cargo bench` when benchmark suite is added)_

Expected throughput on modern hardware:
- Single-threaded reads: >10M ops/sec
- Concurrent reads (16 threads): >100M ops/sec
- Mixed workload (80% read): >50M ops/sec

## 🧪 Testing

```bash
# Run unit tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Run specific test
cargo test test_ttl_expiration
```

## 📊 Monitoring

The cache exposes metrics via the `Stats` gRPC endpoint:

```rust
pub struct StatsResponse {
    pub total_entries: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}
```

Integrate with your monitoring stack:
- Prometheus exporter (TODO)
- Grafana dashboards (TODO)
- OpenTelemetry traces (TODO)

## 🔒 Security

- **Distroless base image**: Minimal attack surface
- **Non-root execution**: Runs as UID 65532
- **No unwrap() calls**: Proper error handling throughout
- **Input validation**: gRPC request validation

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure `cargo clippy` and `cargo fmt` pass
5. Submit a pull request

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🙏 Acknowledgments

Built with:
- [tokio](https://tokio.rs/) - Async runtime
- [DashMap](https://github.com/xacrimon/dashmap) - Concurrent HashMap
- [rkyv](https://github.com/rkyv/rkyv) - Zero-copy serialization
- [tonic](https://github.com/hyperium/tonic) - gRPC framework
- [crossbeam](https://github.com/crossbeam-rs/crossbeam) - Concurrent primitives

## 📝 Roadmap

- [ ] Prometheus metrics exporter
- [ ] Redis protocol compatibility
- [ ] Persistent storage backend
- [ ] Clustered deployment support
- [ ] Advanced eviction policies (LFU, ARC)
- [ ] Compression support
- [ ] TLS/mTLS for gRPC

---

**Made for HFT. Built with Rust. Optimized for performance.**
