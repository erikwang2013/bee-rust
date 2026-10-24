// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! Env-gated e2e: a pool far smaller than the concurrent workload still
//! serves every request.
//!
//! Gated by `BEE_KV_REDIS_DSN` like the other real-server suites: without it
//! the test returns immediately. Thirty-two concurrent `GET`s share two
//! connections — callers queue on `wait` instead of failing.

#![cfg(feature = "redis")]

use std::sync::Arc;

use bee_kv::{KvStore, RedisStore};

#[tokio::test]
async fn concurrent_gets_with_a_small_pool_all_succeed() {
    let Some(dsn) = std::env::var("BEE_KV_REDIS_DSN").ok().filter(|s| !s.is_empty()) else {
        return;
    };
    let store = Arc::new(RedisStore::with_pool_size(&dsn, 2).await.unwrap());
    store.set("bee_kv:pool:probe", "ok").await.unwrap();

    let mut set = tokio::task::JoinSet::new();
    for _ in 0..32 {
        let store = Arc::clone(&store);
        set.spawn(async move { store.get("bee_kv:pool:probe").await });
    }
    while let Some(result) = set.join_next().await {
        assert_eq!(result.unwrap().unwrap().as_deref(), Some("ok"));
    }
    store.del("bee_kv:pool:probe").await.unwrap();
}
