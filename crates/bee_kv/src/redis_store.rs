// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use async_trait::async_trait;

use crate::{KvError, KvStore};

/// A [`KvStore`] backed by a Redis server over an async multiplexed
/// connection.
#[cfg(feature = "redis")]
pub struct RedisStore {
    conn: redis::aio::MultiplexedConnection,
}

#[cfg(feature = "redis")]
impl RedisStore {
    /// Create a new `RedisStore` by connecting to the given `addr` (e.g.
    /// `"redis://127.0.0.1:6379"`).
    pub async fn new(addr: &str) -> Result<Self, KvError> {
        let client = redis::Client::open(addr)
            .map_err(|e| KvError::ConnectionError(format!("failed to create client: {e}")))?;
        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| KvError::ConnectionError(format!("failed to connect: {e}")))?;
        Ok(Self { conn })
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl KvStore for RedisStore {
    async fn get(&mut self, key: &str) -> Result<Option<String>, KvError> {
        redis::cmd("GET")
            .arg(key)
            .query_async(&mut self.conn)
            .await
            .map_err(|e| KvError::OperationFailed(e.to_string()))
    }

    async fn set(&mut self, key: &str, value: &str) -> Result<(), KvError> {
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query_async(&mut self.conn)
            .await
            .map_err(|e| KvError::OperationFailed(e.to_string()))
    }

    async fn del(&mut self, key: &str) -> Result<(), KvError> {
        redis::cmd("DEL")
            .arg(key)
            .query_async(&mut self.conn)
            .await
            .map_err(|e| KvError::OperationFailed(e.to_string()))
    }

    async fn exists(&mut self, key: &str) -> Result<bool, KvError> {
        redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut self.conn)
            .await
            .map_err(|e| KvError::OperationFailed(e.to_string()))
    }

    async fn incr(&mut self, key: &str, amount: i64) -> Result<i64, KvError> {
        if amount == 1 {
            redis::cmd("INCR")
                .arg(key)
                .query_async(&mut self.conn)
                .await
                .map_err(|e| KvError::OperationFailed(e.to_string()))
        } else {
            redis::cmd("INCRBY")
                .arg(key)
                .arg(amount)
                .query_async(&mut self.conn)
                .await
                .map_err(|e| KvError::OperationFailed(e.to_string()))
        }
    }

    async fn expire(&mut self, key: &str, seconds: i64) -> Result<(), KvError> {
        redis::cmd("EXPIRE")
            .arg(key)
            .arg(seconds)
            .query_async(&mut self.conn)
            .await
            .map_err(|e| KvError::OperationFailed(e.to_string()))
    }

    async fn mget(&mut self, keys: &[&str]) -> Result<Vec<Option<String>>, KvError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        redis::cmd("MGET")
            .arg(keys)
            .query_async(&mut self.conn)
            .await
            .map_err(|e| KvError::OperationFailed(e.to_string()))
    }

    async fn mset(&mut self, pairs: &[(&str, &str)]) -> Result<(), KvError> {
        if pairs.is_empty() {
            return Ok(());
        }
        let mut cmd = redis::cmd("MSET");
        for (k, v) in pairs {
            cmd.arg(k).arg(v);
        }
        cmd.query_async(&mut self.conn)
            .await
            .map_err(|e| KvError::OperationFailed(e.to_string()))
    }
}
