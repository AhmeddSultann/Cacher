pub mod error;

use crate::core::Cache;
use crate::storage::ShardedCache;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

// Include the generated proto code
pub mod proto {
    tonic::include_proto!("cache");
}

use proto::{
    cache_service_server::{CacheService as CacheServiceTrait, CacheServiceServer},
    *,
};

/// gRPC cache service implementation
pub struct CacheServiceImpl {
    cache: Arc<ShardedCache<Vec<u8>, Vec<u8>>>,
}

impl CacheServiceImpl {
    /// Create a new cache service with the given cache backend
    pub fn new(cache: Arc<ShardedCache<Vec<u8>, Vec<u8>>>) -> Self {
        Self { cache }
    }
}

#[tonic::async_trait]
impl CacheServiceTrait for CacheServiceImpl {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let key = request.into_inner().key;
        
        match self.cache.get(&key).await {
            Ok(Some(value)) => Ok(Response::new(GetResponse {
                value: Some(value),
                found: true,
            })),
            Ok(None) => Ok(Response::new(GetResponse {
                value: None,
                found: false,
            })),
            Err(e) => Err(e.into()),
        }
    }

    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let req = request.into_inner();
        
        let result = if let Some(ttl) = req.ttl_seconds {
            self.cache.set_with_ttl(req.key, req.value, ttl).await
        } else {
            self.cache.set(req.key, req.value).await
        };

        match result {
            Ok(_) => Ok(Response::new(SetResponse { success: true })),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<DeleteResponse>, Status> {
        let key = request.into_inner().key;
        
        match self.cache.delete(&key).await {
            Ok(Some(_)) => Ok(Response::new(DeleteResponse { existed: true })),
            Ok(None) => Ok(Response::new(DeleteResponse { existed: false })),
            Err(e) => Err(e.into()),
        }
    }

    async fn clear(&self, _request: Request<ClearRequest>) -> Result<Response<ClearResponse>, Status> {
        match self.cache.clear().await {
            Ok(count) => Ok(Response::new(ClearResponse {
                items_removed: count as u64,
            })),
            Err(e) => Err(e.into()),
        }
    }

    async fn batch_get(&self, request: Request<BatchGetRequest>) -> Result<Response<BatchGetResponse>, Status> {
        let keys = request.into_inner().keys;
        let mut responses = Vec::with_capacity(keys.len());

        for key in keys {
            match self.cache.get(&key).await {
                Ok(Some(value)) => responses.push(GetResponse {
                    value: Some(value),
                    found: true,
                }),
                Ok(None) => responses.push(GetResponse {
                    value: None,
                    found: false,
                }),
                Err(e) => return Err(e.into()),
            }
        }

        Ok(Response::new(BatchGetResponse { values: responses }))
    }

    async fn batch_set(&self, request: Request<BatchSetRequest>) -> Result<Response<BatchSetResponse>, Status> {
        let entries = request.into_inner().entries;
        let mut count = 0u64;

        for entry in entries {
            let result = if let Some(ttl) = entry.ttl_seconds {
                self.cache.set_with_ttl(entry.key, entry.value, ttl).await
            } else {
                self.cache.set(entry.key, entry.value).await
            };

            match result {
                Ok(_) => count += 1,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(Response::new(BatchSetResponse { items_set: count }))
    }

    async fn contains(&self, request: Request<ContainsRequest>) -> Result<Response<ContainsResponse>, Status> {
        let key = request.into_inner().key;
        
        match self.cache.contains_key(&key).await {
            Ok(exists) => Ok(Response::new(ContainsResponse { exists })),
            Err(e) => Err(e.into()),
        }
    }

    async fn stats(&self, _request: Request<StatsRequest>) -> Result<Response<StatsResponse>, Status> {
        let metrics = self.cache.metrics();
        let snapshot = metrics.snapshot();

        match self.cache.len().await {
            Ok(total) => Ok(Response::new(StatsResponse {
                total_entries: total as u64,
                hits: snapshot.hits,
                misses: snapshot.misses,
                hit_rate: snapshot.hit_rate,
            })),
            Err(e) => Err(e.into()),
        }
    }
}

/// Start the gRPC server
pub async fn start_server(addr: std::net::SocketAddr, cache: Arc<ShardedCache<Vec<u8>, Vec<u8>>>) -> anyhow::Result<()> {
    let service = CacheServiceImpl::new(cache);
    
    println!("🚀 Cache server starting on {}", addr);
    
    Server::builder()
        .add_service(CacheServiceServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
}
