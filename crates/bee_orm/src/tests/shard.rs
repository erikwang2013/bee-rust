// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! Routing vectors, handle forwarding and shard-wide migration checks.

use super::*;
use crate::MigrateOptions;
use crate::shard::{HashRouter, ShardRouter, ShardedPools, key_hash};
use async_trait::async_trait;
use std::sync::Arc;

#[test]
fn fnv1a_matches_known_vectors() {
    assert_eq!(key_hash(&Value::Text("a".into())).unwrap(), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(key_hash(&Value::Text("foobar".into())).unwrap(), 0x8594_4171_f739_67e8);
}

#[test]
fn hash_router_routes_known_keys() {
    let router = HashRouter::new(2, 4).unwrap();
    assert_eq!(router.db_index(&Value::Int(1)).unwrap(), 1);
    assert_eq!(router.table_name("users", &Value::Int(1)).unwrap(), "users_0");
    assert_eq!(router.db_index(&Value::Int(2)).unwrap(), 1);
    assert_eq!(router.table_name("users", &Value::Int(2)).unwrap(), "users_3");
}

#[test]
fn mixed_shard_counts_reach_every_database_table_pair() {
    let router = HashRouter::new(2, 4).unwrap();
    let seen: std::collections::HashSet<(usize, String)> = (0..256)
        .map(|key| {
            let key = Value::Int(key);
            (router.db_index(&key).unwrap(), router.table_name("users", &key).unwrap())
        })
        .collect();
    assert_eq!(seen.len(), 8);
}

#[test]
fn one_table_shard_keeps_the_logical_name() {
    let router = HashRouter::new(2, 1).unwrap();
    assert_eq!(router.table_name("users", &Value::Int(2)).unwrap(), "users");
    assert_eq!(router.table_name_at("users", 0).unwrap(), "users");
}

#[test]
fn table_name_at_rejects_out_of_range_indices() {
    let router = HashRouter::new(1, 3).unwrap();
    assert_eq!(router.table_name_at("users", 2).unwrap(), "users_2");
    assert!(router.table_name_at("users", 3).is_err());
    let router = HashRouter::new(2, 1).unwrap();
    assert!(router.table_name_at("users", 1).is_err());
}

#[test]
fn unsupported_keys_are_rejected() {
    for key in [Value::Null, Value::Float(1.0), Value::Json(serde_json::json!({}))] {
        assert!(key_hash(&key).is_err(), "{key:?}");
    }
}

#[test]
fn zero_shards_are_rejected() {
    assert!(HashRouter::new(0, 1).is_err());
    assert!(HashRouter::new(1, 0).is_err());
}

#[tokio::test]
async fn for_key_selects_the_router_database() {
    let a = Mock::default();
    let b = Mock::default();
    let (a_probe, b_probe) = (a.clone(), b.clone());
    let shards = ShardedPools::new(vec![a, b], Arc::new(HashRouter::new(2, 1).unwrap())).unwrap();

    shards.for_key(Value::Int(1)).unwrap().execute("SELECT 1", &[]).await.unwrap();
    assert_eq!(a_probe.calls().len(), 1);
    assert_eq!(b_probe.calls().len(), 0);

    shards.for_key(Value::Int(2)).unwrap().execute("SELECT 1", &[]).await.unwrap();
    assert_eq!(b_probe.calls().len(), 1);
}

#[tokio::test]
async fn sync_all_creates_every_physical_table_in_every_pool() {
    let a = Mock::with_dialect(Dialect::Sqlite);
    let b = Mock::with_dialect(Dialect::Sqlite);
    let (a_probe, b_probe) = (a.clone(), b.clone());
    let shards = ShardedPools::new(vec![a, b], Arc::new(HashRouter::new(2, 2).unwrap())).unwrap();

    // Two columns per physical table, two tables per pool, two pools.
    assert_eq!(shards.sync_all::<Author>(MigrateOptions::default()).await.unwrap(), 8);

    let created = |probe: &Mock| -> Vec<String> {
        probe
            .calls()
            .into_iter()
            .map(|(sql, _)| sql)
            .filter(|sql| sql.starts_with("CREATE TABLE"))
            .collect()
    };
    let authors_0 = "CREATE TABLE IF NOT EXISTS authors_0 (id INTEGER PRIMARY KEY AUTOINCREMENT, \
                     deleted INTEGER NOT NULL DEFAULT 0)"
        .to_string();
    let authors_1 = authors_0.replace("authors_0", "authors_1");
    assert_eq!(created(&a_probe), vec![authors_0.clone(), authors_1.clone()]);
    assert_eq!(created(&b_probe), vec![authors_0, authors_1]);
}

#[test]
fn sharded_pools_rejects_pool_count_mismatch() {
    let result = ShardedPools::new(vec![Mock::default()], Arc::new(HashRouter::new(2, 1).unwrap()));
    assert!(result.is_err());
}

struct BadRouter;
impl ShardRouter for BadRouter {
    fn db_shards(&self) -> usize {
        1
    }
    fn table_shards(&self) -> usize {
        1
    }
    fn db_index(&self, _: &Value) -> Result<usize> {
        Ok(0)
    }
    fn table_name(&self, _: &str, _: &Value) -> Result<String> {
        Ok("users; DROP TABLE x".into())
    }
    fn table_name_at(&self, _: &str, _: usize) -> Result<String> {
        Ok("users; DROP TABLE x".into())
    }
}

#[tokio::test]
async fn invalid_physical_table_name_is_rejected() {
    let shards = ShardedPools::new(vec![Mock::default()], Arc::new(BadRouter)).unwrap();
    let handle = shards.for_key(Value::Int(1)).unwrap();
    assert!(handle.resolve_table("users").is_err());
}

struct OutOfRangeRouter;
impl ShardRouter for OutOfRangeRouter {
    fn db_shards(&self) -> usize {
        1
    }
    fn table_shards(&self) -> usize {
        1
    }
    fn db_index(&self, _: &Value) -> Result<usize> {
        Ok(1)
    }
    fn table_name(&self, logical: &str, _: &Value) -> Result<String> {
        Ok(logical.to_owned())
    }
    fn table_name_at(&self, logical: &str, _: usize) -> Result<String> {
        Ok(logical.to_owned())
    }
}

#[test]
fn for_key_rejects_out_of_range_router_indices() {
    let shards = ShardedPools::new(vec![Mock::default()], Arc::new(OutOfRangeRouter)).unwrap();
    assert!(shards.for_key(Value::Int(1)).is_err());
}

struct ReturningDb;
#[async_trait]
impl Db for ReturningDb {
    async fn query(&self, _: &str, _: &[Value]) -> Result<Vec<Row>> {
        Ok(Vec::new())
    }
    async fn execute(&self, _: &str, _: &[Value]) -> Result<u64> {
        Ok(1)
    }
    async fn insert_returning(
        &self,
        _: &str,
        _: &str,
        _: &str,
        _: &[Value],
    ) -> Result<Option<Row>> {
        Ok(Some(row_of(&[("id", serde_json::json!(1))])))
    }
}

#[tokio::test]
async fn insert_returning_delegates_to_the_selected_pool() {
    let shards =
        ShardedPools::new(vec![ReturningDb], Arc::new(HashRouter::new(1, 1).unwrap())).unwrap();
    let handle = shards.for_key(Value::Int(1)).unwrap();
    let row = Db::insert_returning(&handle, "users", "id", "INSERT ...", &[]).await.unwrap();
    assert!(row.is_some());
}

#[tokio::test]
async fn dialect_is_forwarded() {
    let shards = ShardedPools::new(
        vec![Mock::with_dialect(Dialect::Mysql)],
        Arc::new(HashRouter::new(1, 1).unwrap()),
    )
    .unwrap();
    assert_eq!(shards.for_key(Value::Int(1)).unwrap().dialect(), Some(Dialect::Mysql));
}
