<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API Reference

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Back to [README](README.en.md).

This document is the API reference for the Beerust framework: usage, code examples, and interface descriptions for each module.

## Web Core (bee_router)

### Defining a Controller

```rust
use bee_rust::prelude::*;

// Define a controller
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Route Registration

```rust
// Route registration
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context provides:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — response output
- `ctx.redirect()` — redirection
- `ctx.abort()` — request interruption
- `ctx.session` — session access
- `ctx.params` — path parameters

## Security Detection (`security` feature)

Attack detection powered by [security-rust](https://crates.io/crates/security-rust), covering XSS, SQL injection, command injection, SSRF, and 23 other attack types via 27 detectors:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // all 27 detectors enabled
```

Enable in `Cargo.toml`:
```toml
bee_rust = { features = ["security"] }
```

## ORM (bee_orm)

```rust
use bee_orm::pool::sqlite::Pool;
use bee_orm::{Model, Value, migrate, rel};

#[derive(Model)]
#[bee(table = "users")]
struct User {
    #[bee(pk, auto)]
    id: i64,
    #[bee(column = "user_name")]
    name: String,
    age: Option<i32>,
    active: bool,
    #[bee(auto_now_add)]
    created_at: i64,
    #[bee(soft_delete)]
    deleted: bool,
    #[bee(ignore)]
    cache: Vec<u8>,
}

#[derive(Model)]
#[bee(table = "posts")]
struct Post {
    #[bee(pk, auto)]
    id: i64,
    #[bee(fk = User)]
    user_id: Option<i64>,
    title: String,
    #[bee(soft_delete)]
    deleted: bool,
}

async fn demo() -> Result<(), bee_orm::OrmError> {
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql share this shape

    // Schema — migrate::sync renders non-destructive DDL from the model metadata
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — the `auto` primary key is assigned by the database
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — chained filters plus execution: all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // Relations — children for has_many, belongs_to for the parent; under FK enforcement delete the child before the parent
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete soft-deletes (flips the flag); with_deleted() / hard_delete() go around it
    Post::query().filter_eq("title", "hello")?.hard_delete(&pool).await?;
    if let Some(mut u) = author {
        u.age = Some(31);
        u.update(&pool).await?;
        u.delete(&pool).await?;
        let live = User::query().count(&pool).await?;
        let every = User::query().with_deleted().count(&pool).await?;
        u.hard_delete(&pool).await?;
    }
    User::query().filter_eq("user_name", "bob")?.update(&pool, &[("user_name", "bobby".into())]).await?;

    // Transactions — hold a connection from `get()`; sqlite's `CheckedConn` is synchronous
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> Migrations are available: `bee_orm::migrate` (`create_table` / `add_missing_columns` / `sync`) renders dialect-aware DDL, creating tables and adding missing columns — never dropping or altering. Relation reads cover `belongs_to`, has_many (`children*`), and many-to-many (`m2m`). `#[bee(fk = …)]` emits FK DDL: postgres and sqlite (bundled build) enforce the reference, mysql can opt into table-level foreign keys (below).

### Many-to-many (`m2m`)

Declared at the struct level with `#[bee(m2m(Target))]`; the join table is created by `create_table` / `sync` (sync the target model first, then the declaring model):

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // join table user_tag, columns user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // target model first
migrate::sync::<User, _>(&pool).await?;      // declaring model creates the join table

// `auto` ids are assigned by the database: read the instance back before touching relations
User { id: 0, name: "alice".into() }.insert(&pool).await?;
let user = User::query().filter_eq("name", "alice")?.one(&pool).await?.expect("inserted");
Tag { id: 0, name: "rust".into() }.insert(&pool).await?;
let tag = Tag::query().filter_eq("name", "rust")?.one(&pool).await?.expect("inserted");

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // duplicate attach -> composite-PK error
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // idempotent: a second call returns 0 rows
```

> Name overrides: `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`; join tables ride `create_table` / `sync` only — `add_missing_columns` never touches them.

### JSON columns

`serde_json::Value` fields map to sqlite `TEXT` / postgres `JSONB` / mysql `JSON`:

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // required
    extra: Option<serde_json::Value>,  // nullable
}
```

> NULL semantics: SQL `NULL` decodes to `None`; a stored JSON `null` document decodes to `Some(Json::Null)` — the two are different.

### MySQL table-level foreign keys (opt-in)

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // mysql only
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> With it on, fresh tables and newly added columns carry table-level `FOREIGN KEY` (constraint name `{table}_{column}_fk`); pre-existing orphan rows make `ADD CONSTRAINT` fail (no silent skip, and no retrofit of existing columns); on pg / sqlite the option is a no-op (inline already enforces).

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls uses the bundled webpki roots; for a custom rustls config use connect_tls_with:
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig (re-export, same version as the crate)
```

### Date and Decimal types

Date and Decimal fields are feature-gated: enable `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) and `rust_decimal` (`Decimal`) on `bee_orm`; through `bee_rust` the forwarding features are `orm-chrono` / `orm-rust_decimal`.

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date; DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal (same on all three backends)
}

// write: the field turns into the matching Value variant; read: the cell decodes back via serde
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // "1.50" round-trips to an equal value
```

> Supported: `NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`. Out of scope — the macro emits no SQL mapping, use the `#[bee(sql_type = "…")]` escape hatch: `NaiveTime` / `DateTime<Local>` / `FixedOffset` / bare `DateTime`, and nanosecond precision.
> Storage: sqlite is always TEXT (`typeof(col)` is `text`; Decimal too — a DECIMAL declaration gains NUMERIC affinity and swallows "1.50" into REAL 1.5); mysql uses `datetime(6)` / `timestamp(6)` with microseconds — TIMESTAMP reads back as UTC (RFC3339 with `Z`; a strict round trip requires the session time zone to be UTC), DATETIME stays naive without a suffix, dispatched by column type; pg `timestamptz` normalises to UTC.
> Precision ceiling: pg `numeric` above 29 significant digits reads back as `NULL` (the native FromSql rejects it); mysql goes through text, so excess precision errors only at decode time in your serde; for money prefer `#[bee(sql_type = Raw("decimal(12,2)"))]`.
> Compile-time contract: with the feature off, a model carrying a date field fails with E0277 (`Value: From<NaiveDate>` not satisfied) — never a silently wrong column.

## Config (bee_config)

```rust
#[derive(Config)]
#[config(file = "conf/app.conf")]
struct AppConfig {
    app_name: String,
    http_port: u16,
    run_mode: String,
}

let cfg = AppConfig::load("conf/app.conf")?;
```

## Storage Engines

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// memcached backend (feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> `RedisStore` holds a `ConnectionManager`: a dropped connection reconnects automatically (on demand — no background thread — with exponential backoff and jitter) — the command that hits the break errors, the next one waits for the fresh connection. memcached: `incr` creates the counter first (0 when absent) and then applies the delta; counters are unsigned, so decrementing floors at 0 (Redis goes negative); `expire(≤0)` deletes the key.

**Cache backends (bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache` (feature `memcache`) has the same interface; TTL `Some(0)` goes through `DEL` on the Redis backend (`SET … EX 0` is rejected), and is treated as a delete on memcached too (there 0 means never expire); `incr` on a non-numeric value is a serialization error.

**Search Engine (implemented, opt-in):**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // requires feature `elasticsearch`
let engine = Elasticsearch::new("http://localhost:9200");   // sync constructor (no `?`)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> The other drivers work the same way and are opt-in, too: `opensearch`, `clickhouse` (features of the same name).

**Graph Database (implemented, opt-in):**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // requires feature `neo4j`
let db = Neo4j::new("http://localhost:7474");   // sync constructor (no `?`)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> The other drivers work the same way and are opt-in, too: `nebulagraph`, `arangodb` (features of the same name).

**Time Series Database (implemented, opt-in):**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // requires feature `influxdb`
let tsdb = InfluxDB::new("http://localhost:8086");   // sync constructor (no `?`)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> The other drivers work the same way and are opt-in, too: `iotdb`, `questdb` (features of the same name).

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> The backend is any cache implementing `bee_cache::Cache`: `MemoryCache` / `RedisCache` / `MemcacheCache`.

## Logging

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## Templates

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI Tool

```bash
# Scaffold a runnable project (Cargo.toml + src/main.rs)
bee-rust new my-app

# Code generation
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Dev server (--watch restarts on src/ changes)
bee-rust run
bee-rust run --watch

# Packaging (cargo build --release + copy to dist/)
bee-rust pack

# Database migrations (scaffold the entrypoint in your crate and run it)
bee-rust migrate init    # creates src/bin/bee_migrate.rs (refuses to overwrite an existing file)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> Note: `pack --target` is a reserved argument; packaging does not currently
> distinguish target platforms.
