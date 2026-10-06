<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API রেফারেন্স

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

ফিরে যান [README](README.bn.md)।

এই ডকুমেন্টটি Beerust ফ্রেমওয়ার্কের API রেফারেন্স: প্রতিটি মডিউলের ব্যবহার, কোড উদাহরণ ও ইন্টারফেসের ব্যাখ্যা।

## ওয়েব কোর (bee_router)

### কন্ট্রোলার সংজ্ঞায়িত করা

```rust
use bee_rust::prelude::*;

// কন্ট্রোলার সংজ্ঞায়িত করা
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### রাউট রেজিস্ট্রেশন

```rust
// রাউট রেজিস্ট্রেশন
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context যা দেয়:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — রেসপন্স আউটপুট
- `ctx.redirect()` — রিডাইরেক্ট
- `ctx.abort()` — রিকোয়েস্ট বাধা
- `ctx.session` — সেশন অ্যাক্সেস
- `ctx.params` — পাথ প্যারামিটার

## নিরাপত্তা সনাক্তকরণ (`security` feature)

[security-rust](https://crates.io/crates/security-rust)-ভিত্তিক আক্রমণ শনাক্তকরণ ফিল্টার, XSS, SQL ইনজেকশন, কমান্ড ইনজেকশন, SSRF সহ ২৭ ধরনের আক্রমণ কভার করে:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // ২৭টি ডিটেক্টর চালু
```

`Cargo.toml`-এ চালু করুন:
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
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql একই রূপ

    // স্কিমা — migrate::sync মডেল মেটাডেটা থেকে অবিনাশী DDL তৈরি করে
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — `auto` প্রাইমারি কী ডেটাবেস দেয়
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — চেইনড ফিল্টার + এক্সিকিউশন: all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // রিলেশন — has_many-এর জন্য children, প্যারেন্টের জন্য belongs_to; FK প্রয়োগে আগে চাইল্ড মুছুন
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete সফট ডিলিট (ফ্ল্যাগ উল্টায়); with_deleted() / hard_delete() তা এড়ায়
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

    // ট্রানজ্যাকশন — `get()` থেকে কানেকশন ধরে রাখুন; sqlite-এর `CheckedConn` সিঙ্ক্রোনাস
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> মাইগ্রেশন এখন উপলব্ধ: `bee_orm::migrate` (`create_table` / `add_missing_columns` / `sync`) ডায়ালেক্ট অনুযায়ী DDL তৈরি করে — টেবিল তৈরি ও অনুপস্থিত কলাম যোগ করে, কখনো মুছে বা বদলায় না। রিলেশন পড়া `belongs_to`, has_many (`children*`) ও many-to-many (`m2m`) সমর্থন করে। `#[bee(fk = …)]` বিদেশি-কী DDL তৈরি করে: postgres ও sqlite (bundled বিল্ড) এটি প্রয়োগ করে, আর mysql opt-in-এ টেবিল-স্তরের বিদেশি-কী চালু করতে পারে (নিচে)।

### অনেক-থেকে-অনেক (`m2m`)

স্ট্রাক্ট স্তরে `#[bee(m2m(Target))]` দিয়ে ঘোষণা; জয়েন টেবিল তৈরি করে `create_table` / `sync` (আগে লক্ষ্য মডেল, তারপর ঘোষক):

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // জয়েন টেবিল user_tag, কলাম user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // আগে লক্ষ্য মডেল
migrate::sync::<User, _>(&pool).await?;      // ঘোষক মডেল জয়েন টেবিল তৈরি করে

// `auto` কী ডেটাবেস নির্ধারণ করে: রিলেশন স্পর্শের আগে ইনস্ট্যান্স আবার পড়ুন
User { id: 0, name: "alice".into() }.insert(&pool).await?;
let user = User::query().filter_eq("name", "alice")?.one(&pool).await?.expect("inserted");
Tag { id: 0, name: "rust".into() }.insert(&pool).await?;
let tag = Tag::query().filter_eq("name", "rust")?.one(&pool).await?.expect("inserted");

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // ডুপ্লিকেট attach -> কম্পোজিট PK ত্রুটি
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // idempotent: দ্বিতীয় কলে 0 সারি
```

> নাম পরিবর্তন: `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`; জয়েন টেবিল কেবল `create_table` / `sync` দিয়ে তৈরি হয় — `add_missing_columns` সেগুলো স্পর্শ করে না।

### JSON কলাম

`serde_json::Value` ফিল্ড sqlite `TEXT` / postgres `JSONB` / mysql `JSON`-এ ম্যাপ হয়:

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // আবশ্যক
    extra: Option<serde_json::Value>,  // nullable
}
```

> NULL অর্থবোধ: SQL `NULL` ডিকোড হয় `None`-এ; সংরক্ষিত JSON `null` ডকুমেন্ট `Some(Json::Null)`-এ — দুটি আলাদা।

### MySQL টেবিল-স্তরের বিদেশি-কী (opt-in)

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // শুধু mysql
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> চালু করলে নতুন টেবিল ও যোগ করা কলাম টেবিল-স্তরের `FOREIGN KEY` বহন করে (constraint নাম `{table}_{column}_fk`); আগে থেকে থাকা অনাথ সারি `ADD CONSTRAINT` ব্যর্থ করে (নীরব এড়িয়ে যাওয়া নেই, বিদ্যমান কলামের retrofit নেই); pg / sqlite-এ অপশনটি নিষ্ক্রিয় (inline আগেই প্রয়োগ করে)।

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls বান্ডল করা webpki রুট ব্যবহার করে; কাস্টম rustls কনফিগের জন্য connect_tls_with:
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig (পুনঃরপ্তানি, crate-এর মতো সংস্করণ)
```

### তারিখ ও Decimal প্রকার

তারিখ ও Decimal ফিল্ড feature-নিয়ন্ত্রিত: `bee_orm`-এ `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) ও `rust_decimal` (`Decimal`) চালু করুন; `bee_rust` হয়ে গেলে ফরওয়ার্ডিং feature `orm-chrono` / `orm-rust_decimal`।

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date; DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal (তিন ব্যাকএন্ডেই একই)
}

// লেখা: ফিল্ড সংশ্লিষ্ট Value ভ্যারিয়েন্টে যায়; পড়া: সেল serde-তে ডিকোড হয়
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // "1.50" সমান মানে ফেরে
```

> সমর্থিত: `NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`. পরিধির বাইরে — ম্যাক্রো SQL ম্যাপিং দেয় না, `#[bee(sql_type = "…")]` এস্কেপ হ্যাচ: `NaiveTime` / `DateTime<Local>` / `FixedOffset` / খালি `DateTime`, এবং ন্যানোসেকেন্ড নির্ভুলতা।
> সংরক্ষণ: sqlite সর্বদা TEXT (`typeof(col)` = `text`; Decimal-ও — DECIMAL ঘোষণা NUMERIC অ্যাফিনিটি দিয়ে "1.50"-কে REAL 1.5 বানিয়ে ফেলে); mysql `datetime(6)` / `timestamp(6)` মাইক্রোসেকেন্ডসহ — TIMESTAMP UTC হিসেবে পড়া হয় (RFC3339-এ `Z`; কঠোর রাউন্ড-ট্রিপে সেশনের টাইমজোন UTC চাই), DATETIME naive প্রত্যয় ছাড়া, কলাম-টাইপ অনুযায়ী বিভাজন; pg `timestamptz` UTC-তে স্বাভাবিক।
> নির্ভুলতার সীমা: pg-তে `numeric` ২৯ তাৎপর্যপূর্ণ অঙ্কের বেশি হলে `NULL` পড়া যায় (নেটিভ FromSql প্রত্যাখ্যান করে); mysql টেক্সট পথে, বাড়তি নির্ভুলতা ডিকোডের সময় আপনার serde-তে ব্যর্থ; টাকার জন্য `#[bee(sql_type = Raw("decimal(12,2)"))]` শ্রেয়।
> কম্পাইল-টাইম চুক্তি: feature বন্ধ থাকলে তারিখ ফিল্ডের মডেল E0277 (`Value: From<NaiveDate>` অসম্পূর্ণ) দিয়ে ব্যর্থ — কখনও নীরবে ভুল কলাম নয়।

## কনফিগ ম্যানেজমেন্ট (bee_config)

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

## স্টোরেজ ইঞ্জিন

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// memcached ব্যাকএন্ড (feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> `RedisStore`-এর ভিতরে `ConnectionManager`: কাটা সংযোগ নিজে থেকে পুনঃসংযুক্ত হয় (চাহিদামতো — পটভূমি থ্রেড ছাড়াই — সূচকীয় ব্যাকঅফ ও জিটারসহ) — কাটার সাথে ধাক্কা খাওয়া কমান্ডটি ব্যর্থ হয়, পরেরটি নতুন সংযোগের অপেক্ষা করে। memcached: `incr` আগে কাউন্টার তৈরি করে (না থাকলে 0) তারপর পার্থক্য প্রয়োগ করে; কাউন্টার unsigned, তাই কমানো 0-তে থামে (Redis ঋণাত্মক হয়); `expire(≤0)` মুছে ফেলার সমতুল্য।

**ক্যাশ ব্যাকএন্ড (bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache` (feature `memcache`)-এর একই ইন্টারফেস; TTL `Some(0)` Redis ব্যাকএন্ডে `DEL` দিয়ে যায় (`SET … EX 0` প্রত্যাখ্যাত হয়), memcached-এও মুছে ফেলা হিসেবে ধরা হয় (সেখানে 0 মানে কখনো 만료되지 않음); অ-সংখ্যাসূচক মানে `incr` সিরিয়ালাইজেশন ত্রুটি।

**সার্চ ইঞ্জিন (বাস্তবায়িত, opt-in):**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // feature `elasticsearch` প্রয়োজন
let engine = Elasticsearch::new("http://localhost:9200");   // সিঙ্ক্রোনাস কনস্ট্রাক্টর (`?` ছাড়া)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> বাকি ড্রাইভারগুলোও একইভাবে কাজ করে এবং opt-in: `opensearch`, `clickhouse` (একই নামের feature)।

**গ্রাফ ডেটাবেস (বাস্তবায়িত, opt-in):**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // feature `neo4j` প্রয়োজন
let db = Neo4j::new("http://localhost:7474");   // সিঙ্ক্রোনাস কনস্ট্রাক্টর (`?` ছাড়া)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> বাকি ড্রাইভারগুলোও একইভাবে কাজ করে এবং opt-in: `nebulagraph`, `arangodb` (একই নামের feature)।

**টাইম-সিরিজ ডেটাবেস (বাস্তবায়িত, opt-in):**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // feature `influxdb` প্রয়োজন
let tsdb = InfluxDB::new("http://localhost:8086");   // সিঙ্ক্রোনাস কনস্ট্রাক্টর (`?` ছাড়া)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> বাকি ড্রাইভারগুলোও একইভাবে কাজ করে এবং opt-in: `iotdb`, `questdb` (একই নামের feature)।

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> ব্যাকএন্ড হলো যেকোনো ক্যাশ যা `bee_cache::Cache` বাস্তবায়ন করে: `MemoryCache` / `RedisCache` / `MemcacheCache`।

## লগিং

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## টেমপ্লেট

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI টুল

```bash
# প্রজেক্ট তৈরি (রানযোগ্য স্ক্যাফোল্ড তৈরি: Cargo.toml + src/main.rs)
bee-rust new my-app

# কোড জেনারেট করুন
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# ডেভ রান (--watch src/ ফোল্ডারের পরিবর্তন দেখে অটো-রিস্টার্ট)
bee-rust run
bee-rust run --watch

# প্যাকেজিং ও ডিপ্লয় (cargo build --release + dist/-তে কপি)
bee-rust pack

# ডেটাবেস মাইগ্রেশন (আপনার crate-এ এন্ট্রিপয়েন্ট তৈরি করে চালায়)
bee-rust migrate init    # src/bin/bee_migrate.rs তৈরি করে (বিদ্যমান ফাইল ওভাররাইট করে না)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> নোট: `pack --target` প্যারামিটারটি রিজার্ভড ইন্টারফেস; বর্তমান প্যাকেজিং প্রক্রিয়া টার্গেট প্ল্যাটফর্ম আলাদা করে না।
