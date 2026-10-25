// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! A distributed lock on a real Redis.
//!
//! Run with `BEE_KV_REDIS_DSN=redis://127.0.0.1:6379 cargo run`.

use std::time::Duration;

use bee_kv::{KvStore, RedisStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dsn =
        std::env::var("BEE_KV_REDIS_DSN").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let store = RedisStore::new(&dsn).await?;
    println!("pool: {:?}", store.pool_status());

    // The ordinary key-value surface.
    store.set("example:redis-lock:value", "hello").await?;
    println!("value: {:?}", store.get("example:redis-lock:value").await?);

    let lock = store.lock();
    let key = "example:redis-lock:job";

    // Wait up to two seconds for a contended lock; `None` means the wait ran
    // out without acquiring it.
    if let Some(guard) = lock.acquire(key, Duration::from_secs(10), Duration::from_secs(2)).await? {
        println!("acquired after waiting");
        println!("released: {}", guard.release().await?);
    } else {
        println!("gave up waiting");
    }

    // The watchdog renews the TTL every ttl/3 until the guard is released or
    // dropped, so work may outlive the initial TTL.
    if let Some(guard) = lock.try_lock_with_watchdog(key, Duration::from_millis(300)).await? {
        println!("acquired with watchdog");
        println!("released: {}", guard.release().await?);
    }

    // A non-blocking attempt: `None` when someone else holds the key now.
    if let Some(guard) = lock.try_lock(key, Duration::from_secs(10)).await? {
        println!("released: {}", guard.release().await?);
    }

    Ok(())
}
