use crate::core::CacheError;
use anyhow::Result as AnyhowResult;
use tonic::Status;

/// Convert CacheError to gRPC Status
impl From<CacheError> for Status {
    fn from(error: CacheError) -> Self {
        match error {
            CacheError::NotFound(msg) => Status::not_found(msg),
            CacheError::SerializationError(msg) => Status::invalid_argument(format!("Serialization error: {}", msg)),
            CacheError::DeserializationError(msg) => Status::invalid_argument(format!("Deserialization error: {}", msg)),
            CacheError::EvictionError(msg) => Status::internal(format!("Eviction error: {}", msg)),
            CacheError::StorageError(msg) => Status::internal(format!("Storage error: {}", msg)),
            CacheError::InvalidOperation(msg) => Status::invalid_argument(msg),
        }
    }
}

/// Server-specific error type using anyhow for application errors
pub type ServerResult<T> = AnyhowResult<T>;
