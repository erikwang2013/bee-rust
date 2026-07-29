use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("key not found")]
    NotFound,
    #[error("connection error: {0}")]
    ConnectionError(String),
    #[error("serialization error: {0}")]
    SerializeError(String),
}

#[async_trait]
pub trait Cache: Send + Sync {
    /// Retrieve a value by key. Returns `None` if not found or expired.
    async fn get(&self, key: &str) -> Option<Vec<u8>>;

    /// Set a key-value pair with an optional TTL in seconds.
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<u64>) -> Result<(), CacheError>;

    /// Delete a key. Returns `Err(CacheError::NotFound)` if the key does not exist.
    async fn delete(&self, key: &str) -> Result<(), CacheError>;

    /// Increment a counter stored at `key` and return the new value.
    /// If the key does not exist, it is set to 0 before incrementing.
    async fn incr(&self, key: &str) -> Result<i64, CacheError>;
}

struct MemoryEntry {
    value: Vec<u8>,
    expires_at: Option<Instant>,
}

/// An in-memory cache backed by `Arc<RwLock<HashMap<String, MemoryEntry>>>` with TTL expiry.
pub struct MemoryCache {
    store: Arc<RwLock<HashMap<String, MemoryEntry>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let store = self.store.read().await;
        let entry = store.get(key)?;

        // Check TTL expiry
        if let Some(expires_at) = entry.expires_at {
            if Instant::now() >= expires_at {
                return None;
            }
        }

        Some(entry.value.clone())
    }

    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<u64>) -> Result<(), CacheError> {
        let expires_at = ttl.map(|seconds| Instant::now() + std::time::Duration::from_secs(seconds));
        let entry = MemoryEntry { value, expires_at };
        self.store.write().await.insert(key.to_string(), entry);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut store = self.store.write().await;
        if store.remove(key).is_some() {
            Ok(())
        } else {
            Err(CacheError::NotFound)
        }
    }

    async fn incr(&self, key: &str) -> Result<i64, CacheError> {
        let mut store = self.store.write().await;

        if let Some(entry) = store.get(key) {
            // Check TTL expiry
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() >= expires_at {
                    // Expired — reset to 1
                    let entry = MemoryEntry {
                        value: b"1".to_vec(),
                        expires_at: None,
                    };
                    store.insert(key.to_string(), entry);
                    return Ok(1);
                }
            }

            // Parse existing value as i64
            let current: i64 = String::from_utf8_lossy(&entry.value)
                .trim()
                .parse()
                .map_err(|_| CacheError::SerializeError(format!(
                    "value for key '{}' is not an integer",
                    key
                )))?;

            let new_value = current + 1;
            let entry = MemoryEntry {
                value: new_value.to_string().into_bytes(),
                expires_at: entry.expires_at,
            };
            store.insert(key.to_string(), entry);
            Ok(new_value)
        } else {
            // Key does not exist — initialize to 1
            let entry = MemoryEntry {
                value: b"1".to_vec(),
                expires_at: None,
            };
            store.insert(key.to_string(), entry);
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_get() {
        let cache = MemoryCache::new();
        cache.set("hello", b"world".to_vec(), None).await.unwrap();
        assert_eq!(cache.get("hello").await, Some(b"world".to_vec()));
    }

    #[tokio::test]
    async fn test_delete() {
        let cache = MemoryCache::new();
        cache.set("temp", b"data".to_vec(), None).await.unwrap();
        assert!(cache.delete("temp").await.is_ok());
        assert!(cache.get("temp").await.is_none());
        // Deleting a non-existent key should error
        assert!(cache.delete("temp").await.is_err());
    }

    #[tokio::test]
    async fn test_incr() {
        let cache = MemoryCache::new();
        let v = cache.incr("counter").await.unwrap();
        assert_eq!(v, 1);
        let v = cache.incr("counter").await.unwrap();
        assert_eq!(v, 2);
        let v = cache.incr("counter").await.unwrap();
        assert_eq!(v, 3);
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let cache = MemoryCache::new();
        cache.set("ephemeral", b"data".to_vec(), Some(1)).await.unwrap();
        assert_eq!(cache.get("ephemeral").await, Some(b"data".to_vec()));
        // Wait for the key to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        assert_eq!(cache.get("ephemeral").await, None);
    }
}
