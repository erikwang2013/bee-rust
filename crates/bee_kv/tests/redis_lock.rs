// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! Env-gated e2e for the Redis distributed lock.
//!
//! Gated by `BEE_KV_REDIS_DSN` like the other real-server suites: without it
//! the tests return immediately.

#![cfg(feature = "redis")]

use std::time::Duration;

use bee_kv::{RedisLock, RedisStore};

fn key(name: &str) -> String {
    format!("bee_kv:lock:test:{name}:{}", std::process::id())
}

#[tokio::test]
async fn try_lock_is_exclusive_and_release_frees_it() {
    let Some(dsn) = std::env::var("BEE_KV_REDIS_DSN").ok().filter(|s| !s.is_empty()) else {
        return;
    };
    let lock = RedisLock::connect(&dsn).await.unwrap();
    let k = key("exclusive");

    let guard = lock.try_lock(&k, Duration::from_secs(5)).await.unwrap().unwrap();
    assert!(lock.try_lock(&k, Duration::from_secs(5)).await.unwrap().is_none());
    assert!(guard.release().await.unwrap());
    let guard = lock.try_lock(&k, Duration::from_secs(5)).await.unwrap().unwrap();
    assert!(guard.release().await.unwrap());
}

#[tokio::test]
async fn ttl_expires_the_lock() {
    let Some(dsn) = std::env::var("BEE_KV_REDIS_DSN").ok().filter(|s| !s.is_empty()) else {
        return;
    };
    let lock = RedisLock::connect(&dsn).await.unwrap();
    let k = key("ttl");

    assert!(lock.try_lock(&k, Duration::from_millis(100)).await.unwrap().is_some());
    assert!(lock.try_lock(&k, Duration::from_secs(5)).await.unwrap().is_none());
    tokio::time::sleep(Duration::from_millis(300)).await;
    let guard = lock.try_lock(&k, Duration::from_secs(5)).await.unwrap().unwrap();
    assert!(guard.release().await.unwrap());
}

#[tokio::test]
async fn store_lock_shares_the_store_pool() {
    let Some(dsn) = std::env::var("BEE_KV_REDIS_DSN").ok().filter(|s| !s.is_empty()) else {
        return;
    };
    let store = RedisStore::new(&dsn).await.unwrap();
    let k = key("store");

    let guard = store.lock().try_lock(&k, Duration::from_secs(5)).await.unwrap().unwrap();
    assert!(guard.release().await.unwrap());
}

#[tokio::test]
async fn releasing_an_expired_guard_returns_false() {
    let Some(dsn) = std::env::var("BEE_KV_REDIS_DSN").ok().filter(|s| !s.is_empty()) else {
        return;
    };
    let lock = RedisLock::connect(&dsn).await.unwrap();
    let k = key("expired");

    let guard = lock.try_lock(&k, Duration::from_millis(100)).await.unwrap().unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!guard.release().await.unwrap());
}
