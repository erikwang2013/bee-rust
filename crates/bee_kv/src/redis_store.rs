// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use std::time::Duration;

use async_trait::async_trait;
use deadpool_redis::{Runtime, Timeouts};

use crate::{KvError, KvStore};

/// A [`KvStore`] backed by a pool of Redis connections.
///
/// # Reconnect
///
/// A checkout recycles its connection with a PING, so a connection that died
/// while parked is discarded and replaced on the next checkout — a Redis
/// restart or a dropped socket needs no action from the caller. The *command*
/// that runs into the break fails with a [`KvError`], and the *next* one
/// succeeds on a fresh connection. There is no background health-check
/// thread. When every connection in the pool is busy, a checkout waits up to
/// 30 seconds and then fails with [`KvError::ConnectionError`].
#[cfg(feature = "redis")]
pub struct RedisStore {
    pool: deadpool_redis::Pool,
}

#[cfg(feature = "redis")]
impl RedisStore {
    /// Create a new `RedisStore` by connecting to the given `addr` (e.g.
    /// `"redis://127.0.0.1:6379"`) with at most 16 pooled connections.
    ///
    /// Connects eagerly: a bad address fails here rather than on the first
    /// command.
    pub async fn new(addr: &str) -> Result<Self, KvError> {
        Self::with_pool_size(addr, 16).await
    }

    /// Create a new `RedisStore` with a pool of at most `max_size`
    /// connections.
    ///
    /// Connects eagerly, like [`RedisStore::new`].
    pub async fn with_pool_size(addr: &str, max_size: usize) -> Result<Self, KvError> {
        Ok(Self { pool: connect_pool(addr, max_size).await? })
    }
}

/// Build a pool for `addr` and prove it serves a command, so a bad address
/// fails here rather than on the first store operation.
pub(crate) async fn connect_pool(
    addr: &str,
    max_size: usize,
) -> Result<deadpool_redis::Pool, KvError> {
    let pool = deadpool_redis::Config::from_url(addr)
        .builder()
        .map_err(conn_err)?
        .max_size(max_size.max(1))
        .timeouts(Timeouts {
            wait: Some(Duration::from_secs(30)),
            create: Some(Duration::from_secs(5)),
            recycle: Some(Duration::from_secs(5)),
        })
        .runtime(Runtime::Tokio1)
        .build()
        .map_err(conn_err)?;
    let mut conn = pool.get().await.map_err(conn_err)?;
    redis::cmd("PING").query_async::<String>(&mut conn).await.map_err(conn_err)?;
    Ok(pool)
}

#[cfg(feature = "redis")]
#[async_trait]
impl KvStore for RedisStore {
    async fn get(&self, key: &str) -> Result<Option<String>, KvError> {
        let mut conn = self.pool.get().await.map_err(conn_err)?;
        redis::cmd("GET").arg(key).query_async(&mut conn).await.map_err(op_err)
    }

    async fn set(&self, key: &str, value: &str) -> Result<(), KvError> {
        let mut conn = self.pool.get().await.map_err(conn_err)?;
        redis::cmd("SET").arg(key).arg(value).query_async(&mut conn).await.map_err(op_err)
    }

    async fn del(&self, key: &str) -> Result<(), KvError> {
        let mut conn = self.pool.get().await.map_err(conn_err)?;
        redis::cmd("DEL").arg(key).query_async(&mut conn).await.map_err(op_err)
    }

    async fn exists(&self, key: &str) -> Result<bool, KvError> {
        let mut conn = self.pool.get().await.map_err(conn_err)?;
        redis::cmd("EXISTS").arg(key).query_async(&mut conn).await.map_err(op_err)
    }

    async fn incr(&self, key: &str, amount: i64) -> Result<i64, KvError> {
        let mut conn = self.pool.get().await.map_err(conn_err)?;
        incr_cmd(key, amount).query_async(&mut conn).await.map_err(op_err)
    }

    async fn expire(&self, key: &str, seconds: i64) -> Result<(), KvError> {
        let mut conn = self.pool.get().await.map_err(conn_err)?;
        redis::cmd("EXPIRE").arg(key).arg(seconds).query_async(&mut conn).await.map_err(op_err)
    }

    async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<String>>, KvError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let mut conn = self.pool.get().await.map_err(conn_err)?;
        redis::cmd("MGET").arg(keys).query_async(&mut conn).await.map_err(op_err)
    }

    async fn mset(&self, pairs: &[(&str, &str)]) -> Result<(), KvError> {
        if pairs.is_empty() {
            return Ok(());
        }
        let mut cmd = redis::cmd("MSET");
        for (k, v) in pairs {
            cmd.arg(k).arg(v);
        }
        let mut conn = self.pool.get().await.map_err(conn_err)?;
        cmd.query_async(&mut conn).await.map_err(op_err)
    }
}

fn conn_err(e: impl std::fmt::Display) -> KvError {
    KvError::ConnectionError(e.to_string())
}

fn op_err(e: impl std::fmt::Display) -> KvError {
    KvError::OperationFailed(e.to_string())
}

/// `INCR` for 1 (Redis's atomic increment), `INCRBY` otherwise.
fn incr_cmd(key: &str, amount: i64) -> redis::Cmd {
    let mut cmd = redis::cmd(if amount == 1 { "INCR" } else { "INCRBY" });
    cmd.arg(key);
    if amount != 1 {
        cmd.arg(amount);
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packed(cmd: &redis::Cmd) -> String {
        String::from_utf8_lossy(&cmd.get_packed_command()).into_owned()
    }

    #[test]
    fn incr_uses_incr_for_one_and_incrby_for_others() {
        let one = packed(&incr_cmd("visits", 1));
        assert!(one.contains("$4\r\nINCR\r\n"), "{one}");
        assert!(!one.contains("INCRBY"), "{one}");

        let many = packed(&incr_cmd("visits", 5));
        assert!(many.contains("$6\r\nINCRBY\r\n"), "{many}");
        assert!(many.contains("$1\r\n5\r\n"), "{many}");

        let zero = packed(&incr_cmd("visits", 0));
        assert!(zero.contains("$6\r\nINCRBY\r\n"), "{zero}");
    }

    #[test]
    fn packed_command_contains_key() {
        let cmd = packed(&incr_cmd("page/1", 1));
        assert!(cmd.contains("$6\r\npage/1\r\n"), "{cmd}");
    }
}
