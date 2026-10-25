<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API রেফারেন্স

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

ফিরে যান [README](README.bn.md)।

এই ডকুমেন্টটি Beerust ফ্রেমওয়ার্কের API রেফারেন্স: প্রতিটি মডিউলের ব্যবহার, কোড উদাহরণ ও ইন্টারফেসের ব্যাখ্যা।

## ওয়েব কোর (bee_router)

### কন্ট্রোলার সংজ্ঞায়িত করা

```rust
use async_trait::async_trait;  // Cargo.toml-এ: async-trait = "0.1"
use bee_rust::bee_router::context::RouterError;
use bee_rust::prelude::*;

// কন্ট্রোলার সংজ্ঞায়িত করা
struct UserController;

#[async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### রাউট রেজিস্ট্রেশন

```rust
// handler হলো axum handler — স্টেটলেস হলে সাধারণ async fn
async fn list_users() -> &'static str { "[]" }
async fn create_user() -> &'static str { "created" }

// রাউট রেজিস্ট্রেশন
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users", list_users)
          .post("/users", create_user);
    });
```

> পাথ সিনট্যাক্স axum 0.8-এর মতো: ডায়নামিক সেগমেন্ট `{name}` (যেমন `/users/{id}`); `:name` build-এ panic করে (পরীক্ষিত)। `ns` প্রিফিক্স গ্রুপ-পাথের সাথে সরাসরি যুক্ত হয়: `ns("/api/v1", …)` + `/users` → `/api/v1/users`; খালি প্রিফিক্স (`ns("", …)`) বৈধ — তখন গ্রুপ-পাথই সম্পূর্ণ পাথ।

### স্টেটযুক্ত handler

handler হলো axum handler: শেয়ার্ড স্টেট `State`-এ ইনজেক্ট হয় ও `Router::with_state`-এ জোড়া লাগে (`axum` সরাসরি নির্ভরতা হতে হবে — CLI স্ক্যাফোল্ডের Cargo.toml-এ তা আছে)। স্টেটলেস রুটে `build()`; handler `State` নিলে `axum::serve`-এর আগে `with_state` আবশ্যক। প্রচলিত `Pool + Cache` রূপ:

```rust
use std::sync::Arc;
use axum::extract::State;
use bee_rust::bee_cache::MemoryCache;
use bee_rust::bee_orm::pool::sqlite::Pool;
use bee_rust::prelude::*;

#[derive(Model)]
#[bee(crate = "bee_rust::bee_orm", table = "users")]  // শুধু bee_rust-এ নির্ভর করার রূপ
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

#[derive(Clone)]
struct AppState {
    pool: Pool,
    cache: Arc<dyn Cache>,
}

async fn list_users(State(state): State<AppState>) -> Result<String, String> {
    let users = User::query().all(&state.pool).await.map_err(|e| e.to_string())?;  // একটি কোয়েরি
    Ok(format!("{} users", users.len()))
}

let app = Router::new()
    .ns("/api/v1", |ns| ns.get("/users", list_users))
    .with_state(AppState { pool, cache: Arc::new(MemoryCache::new()) });
```

### Context API

**Context যা দেয়:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — রেসপন্স আউটপুট
- `ctx.redirect()` — রিডাইরেক্ট
- `ctx.abort()` — রিকোয়েস্ট বাধা
- `ctx.session` — সেশন অ্যাক্সেস
- `ctx.params` — পাথ প্যারামিটার

### লেটেন্সি বাজেট ও ক্যাশ ফলব্যাক

`Router::latency` হলো opt-in GET/HEAD লেটেন্সি বাজেট। ডিফল্ট মোট **6 ms**: handler-এর জন্য 4 ms এবং cache-fallback রিজার্ভের জন্য 2 ms, যা 10 ms P99 লক্ষ্যের নিচে scheduling হেডরুম রাখে; `.deadline(10ms)` দিয়ে ওভাররাইড করুন। সময় শেষ হলে এটি ক্যাশ পড়ে, `X-Bee-Cache: stale` / `Age` সহ একটি public রেসপন্স রিপ্লে করে, অথবা 504 `"request timed out"` + `X-Bee-Deadline: exceeded` ফেরত দেয়। write মেথড এই বাজেটের অধীনে নয় এবং বিদ্যমান 30 s/408 আচরণ বজায় রাখে।

```rust
use std::sync::Arc;
use axum::http::header;
use axum::response::IntoResponse;
use bee_cache::{Cache, MemoryCache};
use bee_router::{LatencyConfig, Router};

let cache: Arc<dyn Cache> = Arc::new(MemoryCache::new());
let config = LatencyConfig::new(cache);   // optional .deadline(10ms) / .fallback_reserve(2ms)
let stats = config.stats();
let app = Router::new()
    .latency(config)
    .ns("/api", |ns| ns.get("/feed", feed));

async fn feed() -> impl IntoResponse {
    ([(header::CACHE_CONTROL, "public, max-age=30")], "payload")
}
```

ক্যাশ কন্ট্রাক্ট:
- শুধু **GET 200** রেসপন্স সংরক্ষিত হয়; `Cache-Control`-এ `public` থাকতে হবে এবং `private`/`no-store`/`no-cache` থাকতে পারবে না, `Set-Cookie` বা `Vary` থাকতে পারবে না, এবং body-র `size_hint().exact()` অবশ্যই ≤ 64 KiB হতে হবে। HEAD GET কী পুনরায় ব্যবহার করে কিন্তু কখনো সংরক্ষণ করে না।
- `Authorization` / `Range` / `Cache-Control: no-store` / `no-cache` সহ অনুরোধ ক্যাশ বাইপাস করে এবং টাইমআউটে 504 ফেরত দেয়।
- TTL হলো কনফিগার করা মান ও রেসপন্সের `max-age`-এর ছোটটি; write কখনো ক্যাশ অকার্যকর করে না, তাই রেসপন্স এক TTL পর্যন্ত পুরনো হতে পারে।
- টাইমআউট হওয়া GET পরের অনুরোধের জন্য ক্যাশ গরম করতে একটি **সীমিত ব্যাকগ্রাউন্ড টাস্কে** (ডিফল্ট 128; `background_limit(0)` নিষ্ক্রিয় করে) চলতে থাকে।
- deadline রেসপন্সকে সীমিত করে, সার্ভার-সাইড কাজকে নয়: SQLite/Postgres/MySQL/Redis কাজ বাতিল হয় না। long-poll, SSE ও ডাউনলোড রুটে **`latency` ছাড়া** Router ব্যবহার করা উচিত।
- `LatencyStats::snapshot()` অনুরোধ / টাইমআউট / fallback হিট এবং P50/P99 বাকেটের ঊর্ধ্বসীমা ও সঠিক `max_us` রিপোর্ট করে।

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

> শুধু `bee_rust`-এ নির্ভর করলে: মডেলে `#[bee(crate = "bee_rust::bee_orm")]` দিন (প্রথম পছন্দ), বা মডিউলের শীর্ষে `use bee_rust::bee_orm;` — এক্সপ্যানশন ডিফল্টে `bee_orm::…` পাথ লেখে, দুটোর যেকোনো একটি থাকলেই সেগুলো রিজলভ হয়; বাকি `bee_orm::` পাথ `bee_rust::bee_orm::` হয়ে যায় (নিচের উদাহরণ সরাসরি `bee_orm` নির্ভরতার জন্য)।

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

    // INSERT — insert() প্রভাবিত সারি ফেরত দেয়
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

> লেখার দুই রূপ: `insert()` প্রভাবিত সারি (`u64`) ফেরত দেয়; `create()` ইনসার্ট করে সারি আবার পড়ে `auto` প্রাইমারি কী / ডিফল্ট ভরা ইনস্ট্যান্স ফেরত দেয় (আবার পড়তে না পারলে `NotFound`) — কী দরকার হলে সরাসরি `create()` (নিচের m2m উদাহরণ)।

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

// create() ইনসার্ট করে ইনস্ট্যান্স আবার পড়ে (auto pk ভরা) — সরাসরি রিলেশনের জন্য প্রস্তুত
let user = User { id: 0, name: "alice".into() }.create(&pool).await?;
let tag = Tag { id: 0, name: "rust".into() }.create(&pool).await?;

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

> MySQL 9 ও MariaDB 10.11+ কলাম-স্তরের `REFERENCES` প্রয়োগ করে; MySQL 5.7/8.x এটি পার্স করে উপেক্ষা করে। প্রথমে রেফারেন্সকৃত টেবিল তৈরি করুন।

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

### শার্ডিং (bee_orm)

key-required ডেটাবেস/টেবিল শার্ডিং। `#[bee(shard_key)]` রাউটিং ফিল্ড চিহ্নিত করে এবং derive `Model::shard_key()` তৈরি করে; `ShardedPools::for_model` সরাসরি একটি ইনস্ট্যান্স রাউট করে, আর `for_key` এখনও মান নেয়। কোয়েরিতে শার্ড কী থাকতে হবে। রাউটিংয়ের আগে শার্ড কী কলারকেই সরবরাহ করতে হবে: ম্যাক্রো `auto` / `auto_now_add` / `auto_now` ফিল্ড প্রত্যাখ্যান করে, এবং `None` / `NULL` কে কী নেই বলে ধরা হয় (`for_model` `ShardError` ফেরত দেয়)। রাউটিং একটি composite ইনডেক্স `hash % (db_shards * table_shards)` ব্যবহার করে, তাই প্রতিটি ডেটাবেস/টেবিল জোড়া পৌঁছানো যায়; ক্রস-শার্ড কোয়েরি, JOIN বা ট্রানজেকশন সমর্থিত নয়। SQL-এ বসানোর আগে physical টেবিল নাম `[A-Za-z_][A-Za-z0-9_]*` দিয়ে যাচাই করা হয়। `ShardedPools::all_shards()` / `pools()` / `pool(index)` অপারেশন ও একই-শার্ড লেনদেনের জন্য অন্তর্নিহিত পুল উন্মুক্ত করে; রাউটিং এখনও `for_key` / `for_model` ছাড়া সম্ভব নয়। `sync_all` প্রতিটি pool × table shard ক্রমানুসারে মাইগ্রেট করে; `sync_all_parallel` সেই লক্ষ্যগুলো সমবর্তীভাবে চালায় (আগেই প্রয়োগ করা DDL ফিরিয়ে নেওয়া হয় না)। শার্ডিং অপব্যবহার `OrmError::ShardError` ফেরত দেয়, এবং `OrmError` হলো `#[non_exhaustive]`।

```rust
use std::sync::Arc;
use bee_orm::shard::{HashRouter, ShardedPools};

#[derive(Model)]
#[bee(table = "users")]
struct User {
    #[bee(pk, auto)]
    id: i64,
    #[bee(shard_key)]
    tenant_id: i64,
    name: String,
}

let shards = ShardedPools::new(vec![pool0, pool1], Arc::new(HashRouter::new(2, 4)?))?;
let user = User { id: 0, tenant_id: 42, name: "alice".into() };
let db = shards.for_model(&user)?;                  // routed by #[bee(shard_key)]
let users = User::query().filter_eq("tenant_id", 42)?.all(&db).await?;
shards.sync_all::<User>(Default::default()).await?; // create every physical table
```

### পুল স্ট্যাটাস ও ধীর কোয়েরি

তিনটি ব্যাকএন্ডই `PoolStatus { max_size, size, available }` ফেরত দেয়; mysql কোনো পরিসংখ্যান দেয় না, তাই শেষ দুটি `None`। `TimedDb` যেকোনো `Db` মোড়ক করে এবং যখন `query` / `execute` বা `insert_returning` / `insert_returning_auto` read-back অন্তত থ্রেশহোল্ড সময় নেয় তখন callback ডাকে; read-back মান অবিকৃতভাবে পাস হয় এবং শুধু সময় মাপা হয়।

```rust
use std::time::Duration;
use bee_orm::{Db, TimedDb};

let db = TimedDb::new(pool, Duration::from_millis(200), |sql, elapsed| {
    eprintln!("slow ({elapsed:?}): {sql}");
});
let rows = db.query("SELECT ...", &[]).await?;
```

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

// সীমিত-অপেক্ষার বিতরণকৃত লক; watchdog ভ্যারিয়েন্ট TTL নবীন করে
let lock = kv.lock();
if let Some(guard) = lock.acquire("job", Duration::from_secs(10), Duration::from_secs(2)).await? {
    // ... work ...
    guard.release().await?;
}

// TLS / ACL: ক্রেডেনশিয়াল ও স্কিম URL-এ (feature `redis-tls`)
let kv = RedisStore::new("rediss://user:pass@redis.example:6380").await?;

// Sentinel / Cluster (features `redis-sentinel` / `redis-cluster`)
use bee_kv::{RedisSentinelStore, RedisClusterStore};
let kv = RedisSentinelStore::new("mymaster", &["redis://127.0.0.1:26379"], 16).await?;
let kv = RedisClusterStore::new(&["redis://127.0.0.1:7000"], 16).await?;

// memcached ব্যাকএন্ড (feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> `RedisStore` একটি `deadpool-redis` পুল ধরে: নিহত কানেকশন পরের checkout-এ প্রতিস্থাপিত হয় (recycle PING); প্রতিটি কমান্ড 30 সেকেন্ডে সীমিত, এবং exhausted পুল ব্যর্থ হওয়ার আগে সর্বোচ্চ 30 সেকেন্ড অপেক্ষা করে। `bee_kv::RedisLock` `SET NX PX` অধিগ্রহণ ও Lua compare-and-delete রিলিজ দেয়; `acquire(key, ttl, wait)` সীমিত অপেক্ষা শেষ হওয়া পর্যন্ত সর্বোচ্চ প্রতি 50ms-এ পুনরায় চেষ্টা করে, এবং `try_lock_with_watchdog` ব্যাকগ্রাউন্ডে প্রতি TTL/3-এ TTL নবীন করে। `RedisOptions` পুল আকার ও রেসপন্স টাইমআউট সেট করে, এবং `status()` / `RedisPoolStatus` পুল স্ন্যাপশট পড়ে। `redis-tls` URL-এ ACL ক্রেডেনশিয়ালসহ `rediss://` সক্ষম করে (`redis://user:pass@host`)।
>
> Sentinel / Cluster: `redis-sentinel` `RedisSentinelStore` / `RedisSentinelCache` দেয়; `redis-cluster` `RedisClusterStore` / `RedisClusterCache` দেয়। Sentinel URL ক্রেডেনশিয়াল/TLS শুধু Sentinel-গুলোর জন্য প্রযোজ্য; master কানেকশনে কোনোটিই থাকে না, এবং পুল করা কানেকশন role-check ছাড়া PING-recycle হয়, তাই failover-এর পরে এখনও পৌঁছানো যায় এমন পুরনো master সেবা দিতে থাকতে পারে (`deadpool-redis` 0.18 সীমাবদ্ধতা)। Cluster-এ `mget` / `mset` অনুক্রমিক একক-কী কমান্ড (`CROSSSLOT` এড়াতে), এবং cluster কমান্ড `response_timeout` দ্বারা সীমিত নয়।
>
> memcached: `incr` আগে কাউন্টার তৈরি করে (না থাকলে 0) তারপর পার্থক্য প্রয়োগ করে; কাউন্টার unsigned, তাই কমানো 0-তে থামে (Redis ঋণাত্মক হয়); `expire(≤0)` মুছে ফেলার সমতুল্য।

**ক্যাশ ব্যাকএন্ড (bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache` (feature `memcache`)-এর একই ইন্টারফেস; TTL `Some(0)` Redis ব্যাকএন্ডে `DEL` দিয়ে যায় (`SET … EX 0` প্রত্যাখ্যাত হয়), memcached-এও মুছে ফেলা হিসেবে ধরা হয় (সেখানে 0 মানে কখনো মেয়াদ শেষ হয় না); অ-সংখ্যাসূচক মানে `incr` সিরিয়ালাইজেশন ত্রুটি।

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
