// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! A distributed mutual-exclusion lock on Redis.
//!
//! [`RedisLock::try_lock`] is one `SET NX PX`, so at most one caller holds a
//! key. There is no watchdog or TTL renewal, no reentrancy, and no blocking
//! wait: a lost race returns `None` immediately. The TTL is the safety net —
//! a guard whose owner dies is reclaimed when it runs out, which is also why
//! [`LockGuard::release`] is best-effort. Release explicitly to free the key
//! early; dropping a guard does not release it.
//!
//! Every command is bounded at 30 seconds by the same pooled checkout that
//! [`crate::RedisStore`] uses.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::KvError;
use crate::redis_store::{checkout_pool, connect_pool, op_err};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A Redis-backed lock, one key per [`RedisLock::try_lock`] call.
pub struct RedisLock {
    pool: deadpool_redis::Pool,
}

impl RedisLock {
    /// Connect to `addr` (e.g. `"redis://127.0.0.1:6379"`) with at most 16
    /// pooled connections.
    ///
    /// Connects eagerly, like [`crate::RedisStore::new`].
    pub async fn connect(addr: &str) -> Result<Self, KvError> {
        Self::with_pool_size(addr, 16).await
    }

    /// Connect to `addr` with a pool of at most `max_size` connections.
    pub async fn with_pool_size(addr: &str, max_size: usize) -> Result<Self, KvError> {
        Ok(Self { pool: connect_pool(addr, max_size).await? })
    }

    pub(crate) fn from_pool(pool: deadpool_redis::Pool) -> Self {
        Self { pool }
    }

    /// Try to take the lock at `key` for `ttl`, returning `None` when another
    /// caller holds it. Never waits.
    ///
    /// An empty `key` or a `ttl` below 1 ms is rejected with
    /// [`KvError::OperationFailed`].
    pub async fn try_lock(&self, key: &str, ttl: Duration) -> Result<Option<LockGuard>, KvError> {
        let ttl_ms = ttl.as_millis();
        if key.is_empty() || ttl_ms == 0 {
            return Err(KvError::OperationFailed(
                "lock: key must be non-empty and ttl at least 1ms".into(),
            ));
        }
        let ttl_ms = i64::try_from(ttl_ms).unwrap_or(i64::MAX);
        let token = new_token();

        let mut conn = checkout_pool(&self.pool).await?;
        let acquired = redis::cmd("SET")
            .arg(key)
            .arg(&token)
            .arg("NX")
            .arg("PX")
            .arg(ttl_ms)
            .query_async::<Option<String>>(&mut conn)
            .await
            .map_err(op_err)?;

        Ok(acquired.map(|_| LockGuard { pool: self.pool.clone(), key: key.to_owned(), token }))
    }
}

/// A held lock.
///
/// Call [`LockGuard::release`] to free the key early. Dropping the guard does
/// not release it — the TTL reclaims the key instead.
pub struct LockGuard {
    pool: deadpool_redis::Pool,
    key: String,
    token: String,
}

impl LockGuard {
    /// Delete the key only if it still holds this guard's token.
    ///
    /// Returns `false` when the TTL already expired and the lock was lost,
    /// `true` when this call freed it. Consumes the guard, so a lock cannot be
    /// released twice.
    pub async fn release(self) -> Result<bool, KvError> {
        const LUA: &str = r#"
if redis.call("GET", KEYS[1]) == ARGV[1] then
    return redis.call("DEL", KEYS[1])
else
    return 0
end
"#;
        let mut conn = checkout_pool(&self.pool).await?;
        let deleted: i64 = redis::Script::new(LUA)
            .key(&self.key)
            .arg(&self.token)
            .invoke_async(&mut conn)
            .await
            .map_err(op_err)?;
        Ok(deleted != 0)
    }
}

/// Unique per process and call: pid, wall-clock nanos, and a counter.
fn new_token() -> String {
    let nanos =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_nanos());
    format!("{}-{nanos}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pool that never connects: the validation below runs before checkout.
    fn offline_lock() -> RedisLock {
        let pool = deadpool_redis::Config::from_url("redis://127.0.0.1:1")
            .builder()
            .unwrap()
            .max_size(1)
            .runtime(deadpool_redis::Runtime::Tokio1)
            .build()
            .unwrap();
        RedisLock::from_pool(pool)
    }

    #[tokio::test]
    async fn rejects_an_empty_key_or_a_sub_millisecond_ttl() {
        let lock = offline_lock();
        for (key, ttl) in [("", Duration::from_secs(1)), ("k", Duration::from_micros(999))] {
            let Err(err) = lock.try_lock(key, ttl).await else {
                panic!("expected an error for key {key:?}, ttl {ttl:?}");
            };
            assert!(matches!(err, KvError::OperationFailed(_)), "{err:?}");
        }
    }
}
