use thiserror::Error;

/// Cache operation errors
#[derive(Error, Debug, Clone)]
pub enum CacheError {
    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("Serialization failed: {0}")]
    SerializationError(String),

    #[error("Deserialization failed: {0}")]
    DeserializationError(String),

    #[error("Eviction policy error: {0}")]
    EvictionError(String),

    #[error("Storage backend error: {0}")]
    StorageError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type CacheResult<T> = Result<T, CacheError>;
