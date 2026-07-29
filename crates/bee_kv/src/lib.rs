// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use async_trait::async_trait;
use thiserror::Error;

#[cfg(feature = "redis")]
mod redis_store;
#[cfg(feature = "redis")]
pub use redis_store::RedisStore;

/// Errors that can occur when interacting with a key-value store.
#[derive(Error, Debug)]
pub enum KvError {
    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("key not found: {0}")]
    NotFound(String),

    #[error("operation failed: {0}")]
    OperationFailed(String),
}

/// A generic async key-value store trait.
///
/// Implementors provide basic CRUD operations, expiration, atomic
/// increment, and batched get/set.
#[async_trait]
pub trait KvStore: Send + Sync {
    /// Get the value for `key`.  Returns `None` when the key does not exist.
    async fn get(&self, key: &str) -> Result<Option<String>, KvError>;

    /// Set `key` to `value`, overwriting any existing value.
    async fn set(&self, key: &str, value: &str) -> Result<(), KvError>;

    /// Delete `key`.
    async fn del(&self, key: &str) -> Result<(), KvError>;

    /// Returns `true` when `key` exists.
    async fn exists(&self, key: &str) -> Result<bool, KvError>;

    /// Atomically increment the integer stored at `key` by `amount`.
    /// Returns the new value.
    async fn incr(&self, key: &str, amount: i64) -> Result<i64, KvError>;

    /// Set a TTL (time-to-live) in seconds on `key`.
    async fn expire(&self, key: &str, seconds: i64) -> Result<(), KvError>;

    /// Batch get — returns values in the same order as the requested keys.
    /// Missing keys are represented as `None`.
    async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<String>>, KvError>;

    /// Batch set — each tuple is `(key, value)`.
    async fn mset(&self, pairs: &[(&str, &str)]) -> Result<(), KvError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// In-memory stub used by doc-tests and integration tests.
    pub struct StubKvStore {
        data: Mutex<std::collections::HashMap<String, String>>,
    }

    impl StubKvStore {
        pub fn new() -> Self {
            Self {
                data: Mutex::new(std::collections::HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl KvStore for StubKvStore {
        async fn get(&self, key: &str) -> Result<Option<String>, KvError> {
            let map = self.data.lock().unwrap();
            Ok(map.get(key).cloned())
        }

        async fn set(&self, key: &str, value: &str) -> Result<(), KvError> {
            let mut map = self.data.lock().unwrap();
            map.insert(key.to_string(), value.to_string());
            Ok(())
        }

        async fn del(&self, key: &str) -> Result<(), KvError> {
            let mut map = self.data.lock().unwrap();
            map.remove(key);
            Ok(())
        }

        async fn exists(&self, key: &str) -> Result<bool, KvError> {
            let map = self.data.lock().unwrap();
            Ok(map.contains_key(key))
        }

        async fn incr(&self, key: &str, amount: i64) -> Result<i64, KvError> {
            let mut map = self.data.lock().unwrap();
            let entry = map
                .entry(key.to_string())
                .or_insert_with(|| "0".to_string());
            let current: i64 = entry
                .parse()
                .map_err(|_| KvError::OperationFailed("value is not an integer".into()))?;
            let new_val = current + amount;
            *entry = new_val.to_string();
            Ok(new_val)
        }

        async fn expire(&self, _key: &str, _seconds: i64) -> Result<(), KvError> {
            Ok(())
        }

        async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<String>>, KvError> {
            let map = self.data.lock().unwrap();
            Ok(keys.iter().map(|k| map.get(*k).cloned()).collect())
        }

        async fn mset(&self, pairs: &[(&str, &str)]) -> Result<(), KvError> {
            let mut map = self.data.lock().unwrap();
            for (k, v) in pairs {
                map.insert(k.to_string(), v.to_string());
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_stub_get_set() {
        let store = StubKvStore::new();
        store.set("hello", "world").await.unwrap();
        assert_eq!(store.get("hello").await.unwrap(), Some("world".into()));
    }

    #[tokio::test]
    async fn test_stub_incr() {
        let store = StubKvStore::new();
        let val = store.incr("counter", 1).await.unwrap();
        assert_eq!(val, 1);
        let val = store.incr("counter", 4).await.unwrap();
        assert_eq!(val, 5);
    }

    #[tokio::test]
    async fn test_stub_mget_mset() {
        let store = StubKvStore::new();
        store
            .mset(&[("a", "1"), ("b", "2"), ("c", "3")])
            .await
            .unwrap();
        let vals = store.mget(&["a", "b", "missing"]).await.unwrap();
        assert_eq!(vals[0], Some("1".into()));
        assert_eq!(vals[1], Some("2".into()));
        assert_eq!(vals[2], None);
    }

    #[tokio::test]
    async fn test_stub_exists_del() {
        let store = StubKvStore::new();
        assert!(!store.exists("x").await.unwrap());
        store.set("x", "y").await.unwrap();
        assert!(store.exists("x").await.unwrap());
        store.del("x").await.unwrap();
        assert!(!store.exists("x").await.unwrap());
    }
}
