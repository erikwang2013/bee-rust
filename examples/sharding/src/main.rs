//! Key-required sharding: two databases × two tables, routed on
//! `#[bee(shard_key)]`.
//!
//! Run with `cargo run` from this directory; it creates `sharding_0.db` and
//! `sharding_1.db` next to it.

use std::sync::Arc;

use bee_orm::pool::sqlite::Pool;
use bee_orm::shard::{HashRouter, ShardedPools};
use bee_orm::{Db, MigrateOptions, Model};

#[derive(Model, Debug, Clone)]
#[bee(table = "users")]
struct User {
    #[bee(pk, auto)]
    id: i64,
    #[bee(shard_key)]
    tenant_id: i64,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pools = (0..2)
        .map(|index| Pool::connect(&format!("sharding_{index}.db"), 2))
        .collect::<Result<Vec<_>, _>>()?;
    // 2 databases × 2 physical tables per logical table.
    let shards = ShardedPools::new(pools, Arc::new(HashRouter::new(2, 2)?))?;

    // Create `users_0` / `users_1` on both pools; 0 added once they exist.
    let added = shards.sync_all::<User>(MigrateOptions::default()).await?;
    println!("sync added {added} columns across {} shards", shards.all_shards().count());

    let user = User { id: 0, tenant_id: 42, name: "alice".into() };

    // `for_model` reads #[bee(shard_key)]; `for_key` takes the value directly.
    let handle = shards.for_model(&user)?;
    assert_eq!(
        handle.resolve_table(User::table_name())?,
        shards.for_key(user.tenant_id)?.resolve_table(User::table_name())?
    );

    user.insert(&handle).await?;
    let found = User::query().filter_eq("tenant_id", 42)?.all(&handle).await?;
    println!("{:?}", found);
    Ok(())
}
