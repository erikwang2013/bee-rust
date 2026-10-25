<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API संदर्भ

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

वापस [README](README.hi.md) पर जाएँ।

यह दस्तावेज़ Beerust फ्रेमवर्क का API संदर्भ है: प्रत्येक मॉड्यूल का उपयोग, कोड उदाहरण और इंटरफ़ेस विवरण।

## Web कोर (bee_router)

### कंट्रोलर परिभाषित करना

```rust
use async_trait::async_trait;  // Cargo.toml में: async-trait = "0.1"
use bee_rust::bee_router::context::RouterError;
use bee_rust::prelude::*;

// कंट्रोलर परिभाषित करें
struct UserController;

#[async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### रूट रजिस्ट्रेशन

```rust
// handler ही axum handler हैं — स्टेटलेस में साधारण async fn
async fn list_users() -> &'static str { "[]" }
async fn create_user() -> &'static str { "created" }

// रूट रजिस्ट्रेशन
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users", list_users)
          .post("/users", create_user);
    });
```

> पाथ सिंटैक्स axum 0.8 जैसा: डायनैमिक सेगमेंट `{name}` (जैसे `/users/{id}`); `:name` build पर panic करता है (परीक्षित)। `ns` प्रीफ़िक्स समूह-पाथ से सीधे जुड़ता है: `ns("/api/v1", …)` + `/users` → `/api/v1/users`; खाली प्रीफ़िक्स (`ns("", …)`) भी वैध — तब समूह-पाथ ही पूरा पाथ है।

### स्टेट वाले handler

handler axum handler ही हैं: साझा स्टेट `State` से इंजेक्ट और `Router::with_state` से जुड़ती है (`axum` सीधी निर्भरता होनी चाहिए — CLI स्कैफ़ोल्ड की Cargo.toml में यह पहले से है)। स्टेटलेस रूट `build()`; handler जैसे ही `State` ले, `axum::serve` से पहले `with_state` ज़रूरी। ठेठ `Pool + Cache` रूप:

```rust
use std::sync::Arc;
use axum::extract::State;
use bee_rust::bee_cache::MemoryCache;
use bee_rust::bee_orm::pool::sqlite::Pool;
use bee_rust::prelude::*;

#[derive(Model)]
#[bee(crate = "bee_rust::bee_orm", table = "users")]  // केवल bee_rust पर निर्भरता की लिपि
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
    let users = User::query().all(&state.pool).await.map_err(|e| e.to_string())?;  // एक क्वेरी
    Ok(format!("{} users", users.len()))
}

let app = Router::new()
    .ns("/api/v1", |ns| ns.get("/users", list_users))
    .with_state(AppState { pool, cache: Arc::new(MemoryCache::new()) });
```

### Context API

**Context प्रदान करता है:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — प्रतिक्रिया आउटपुट
- `ctx.redirect()` — रीडायरेक्ट
- `ctx.abort()` — अनुरोध रोकना
- `ctx.session` — सत्र तक पहुँच
- `ctx.params` — पाथ पैरामीटर

### लेटेंसी बजट और कैश फ़ॉलबैक

`Router::latency` एक opt-in GET/HEAD लेटेंसी बजट है। डिफ़ॉल्ट कुल **6 ms** है: handler के लिए 4 ms और cache-fallback रिज़र्व के लिए 2 ms, जो 10 ms P99 लक्ष्य के नीचे scheduling हेडरूम छोड़ता है; `.deadline(10ms)` से ओवरराइड करें। समय समाप्त होने पर यह कैश पढ़ता है, `X-Bee-Cache: stale` / `Age` के साथ public प्रतिक्रिया फिर से देता है, या 504 `"request timed out"` + `X-Bee-Deadline: exceeded` लौटाता है। write मेथड इस बजट के अंतर्गत नहीं हैं और मौजूदा 30 s/408 व्यवहार बनाए रखते हैं।

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

कैश अनुबंध:
- केवल **GET 200** प्रतिक्रियाएँ संग्रहीत होती हैं; `Cache-Control` में `public` होना चाहिए और `private`/`no-store`/`no-cache` नहीं, `Set-Cookie` या `Vary` नहीं होना चाहिए, और body का `size_hint().exact()` ≤ 64 KiB होना चाहिए। HEAD GET कुंजी का पुन: उपयोग करता है पर कभी संग्रहीत नहीं करता।
- `Authorization` / `Range` / `Cache-Control: no-store` / `no-cache` वाले अनुरोध कैश को बायपास करते हैं और टाइमआउट पर 504 लौटाते हैं।
- TTL कॉन्फ़िगर किए गए मान और प्रतिक्रिया के `max-age` में से छोटा है; write कभी कैश अमान्य नहीं करते, इसलिए प्रतिक्रियाएँ एक TTL तक पुरानी हो सकती हैं।
- टाइमआउट हुआ GET अगले अनुरोध के लिए कैश गर्म करने हेतु एक **सीमित बैकग्राउंड टास्क** (डिफ़ॉल्ट 128; `background_limit(0)` इसे अक्षम करता है) में जारी रहता है।
- deadline प्रतिक्रिया को सीमित करती है, सर्वर-साइड कार्य को नहीं: SQLite/Postgres/MySQL/Redis कार्य रद्द नहीं होता। long-poll, SSE और डाउनलोड रूट को **`latency` के बिना** Router उपयोग करना चाहिए।
- `LatencyStats::snapshot()` अनुरोध / टाइमआउट / fallback हिट के साथ P50/P99 बकेट की ऊपरी सीमाएँ और सटीक `max_us` रिपोर्ट करता है।

## सुरक्षा जाँच (`security` feature)

[security-rust](https://crates.io/crates/security-rust) पर आधारित हमला-पता लगाने वाला फ़िल्टर, जो XSS, SQL इंजेक्शन, कमांड इंजेक्शन, SSRF आदि 27 प्रकार के हमलों को कवर करता है:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // सभी 27 डिटेक्टर चालू
```

`Cargo.toml` में सक्षम करें:
```toml
bee_rust = { features = ["security"] }
```

## ORM (bee_orm)

> केवल `bee_rust` पर निर्भर हों: मॉडल पर `#[bee(crate = "bee_rust::bee_orm")]` लगाएँ (पसंदीदा), या मॉड्यूल के शीर्ष पर `use bee_rust::bee_orm;` — विस्तार डिफ़ॉल्ट रूप से `bee_orm::…` पथ लिखता है, दोनों में से कोई एक उन्हें हल कर देता है; बाकी `bee_orm::` पथ `bee_rust::bee_orm::` हो जाते हैं (नीचे के उदाहरण सीधी `bee_orm` निर्भरता के हैं)।

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
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql एक ही रूप

    // स्कीमा — migrate::sync मॉडल मेटाडेटा से गैर-विनाशकारी DDL बनाता है
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — insert() प्रभावित पंक्तियाँ लौटाता है
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — चेन फ़िल्टर + निष्पादन: all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // संबंध — has_many के लिए children, पैरेंट के लिए belongs_to; FK लागू होने पर पहले चाइल्ड हटाएँ
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete सॉफ़्ट डिलीट है (फ़्लैग पलटता है); with_deleted() / hard_delete() बायपास करते हैं
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

    // ट्रांज़ैक्शन — `get()` से कनेक्शन रखें; sqlite का `CheckedConn` सिंक्रोनस है
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> लिखने के दो रूप: `insert()` प्रभावित पंक्तियाँ (`u64`) लौटाता है; `create()` सम्मिलित करके पंक्ति दोबारा पढ़ता है और `auto` प्राइमरी कुंजी / डिफ़ॉल्ट भरा इंस्टेंस लौटाता है (दोबारा न पढ़ पाने पर `NotFound`) — कुंजी चाहिए तो सीधे `create()` (नीचे m2m उदाहरण)।

> माइग्रेशन अब उपलब्ध हैं: `bee_orm::migrate` (`create_table` / `add_missing_columns` / `sync`) डायलेक्ट के अनुसार DDL बनाता है — तालिकाएँ बनाता और छूटे कॉलम जोड़ता है, कभी हटाता या बदलता नहीं। संबंध पढ़ने के लिए `belongs_to`, has_many (`children*`) और many-to-many (`m2m`) उपलब्ध हैं। `#[bee(fk = …)]` FK DDL बनाता है: postgres और sqlite (bundled बिल्ड) इसे लागू करते हैं, और mysql टेबल-स्तरीय फ़ॉरेन की opt-in से चालू कर सकता है (नीचे)।

### अनेक-से-अनेक (`m2m`)

स्ट्रक्चर स्तर पर `#[bee(m2m(Target))]` से घोषित; join टेबल `create_table` / `sync` बनाता है (पहले लक्ष्य मॉडल, फिर घोषक):

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // join टेबल user_tag, कॉलम user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // पहले लक्ष्य मॉडल
migrate::sync::<User, _>(&pool).await?;      // घोषक मॉडल join टेबल बनाता है

// create() सम्मिलित कर इंस्टेंस दोबारा पढ़ता है (auto pk भरी) — सीधे रिलेशन के लिए तैयार
let user = User { id: 0, name: "alice".into() }.create(&pool).await?;
let tag = Tag { id: 0, name: "rust".into() }.create(&pool).await?;

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // दोहरा attach -> समग्र PK त्रुटि
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // idempotent: दूसरा कॉल 0 पंक्तियाँ लौटाता है
```

> नाम बदलाव: `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`; join टेबल केवल `create_table` / `sync` से बनते हैं — `add_missing_columns` उन्हें नहीं छूता।

### JSON कॉलम

`serde_json::Value` फ़ील्ड sqlite `TEXT` / postgres `JSONB` / mysql `JSON` में मैप होते हैं:

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // आवश्यक
    extra: Option<serde_json::Value>,  // nullable
}
```

> NULL अर्थशास्त्र: SQL `NULL` डिकोड होकर `None`; सहेजा गया JSON `null` दस्तावेज़ `Some(Json::Null)` — दोनों भिन्न हैं।

### MySQL टेबल-स्तरीय फ़ॉरेन की (opt-in)

> MySQL 9 और MariaDB 10.11+ कॉलम-स्तरीय `REFERENCES` लागू करते हैं; MySQL 5.7/8.x इसे पार्स करके अनदेखा करते हैं। पहले संदर्भित टेबल बनाएं।

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // केवल mysql
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> चालू करने पर नई टेबलें और जोड़े गए कॉलम टेबल-स्तरीय `FOREIGN KEY` लेते हैं (constraint नाम `{table}_{column}_fk`); पहले से मौजूद अनाथ पंक्तियाँ `ADD CONSTRAINT` को विफल करती हैं (चुपचाप छोड़ना नहीं, मौजूदा कॉलम का retrofit नहीं); pg / sqlite पर यह विकल्प निष्क्रिय है (inline पहले से लागू करता है)।

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls बंडल किए webpki रूट उपयोग करता है; कस्टम rustls कॉन्फ़िग के लिए connect_tls_with:
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig (पुनः-निर्यात, crate जैसा संस्करण)
```

### दिनांक और Decimal प्रकार

दिनांक और Decimal फ़ील्ड feature-नियंत्रित हैं: `bee_orm` पर `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) और `rust_decimal` (`Decimal`) चालू करें; `bee_rust` से होकर जाने पर अग्रेषण feature `orm-chrono` / `orm-rust_decimal` हैं।

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date; DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal (तीनों बैकएंड पर समान)
}

// लेखन: फ़ील्ड संगत Value वेरिएंट में बदलता है; पठन: सेल serde से डिकोड होता है
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // "1.50" बराबर मान में लौटता है
```

> समर्थित: `NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`. दायरे से बाहर — मैक्रो SQL मैपिंग नहीं देता, `#[bee(sql_type = "…")]` एस्केप हैच है: `NaiveTime` / `DateTime<Local>` / `FixedOffset` / नंगा `DateTime`, और नैनोसेकंड परिशुद्धता।
> भंडारण: sqlite सदैव TEXT (`typeof(col)` = `text`; Decimal भी — DECIMAL घोषित करने पर NUMERIC एफ़िनिटी मिलती और "1.50" REAL 1.5 बन जाता); mysql में `datetime(6)` / `timestamp(6)` माइक्रोसेकंड सहित — TIMESTAMP UTC के रूप में पढ़ा जाता है (RFC3339 में `Z`; कड़े राउंड-ट्रिप के लिए सत्र टाइमज़ोन UTC चाहिए), DATETIME naive बिना प्रत्यय, कॉलम-प्रकार से वितरण; pg `timestamptz` UTC में सामान्यीकृत।
> परिशुद्धता की सीमा: pg में `numeric` 29 सार्थक अंकों से ऊपर `NULL` पढ़ा जाता है (मूल FromSql अस्वीकार करता है); mysql टेक्स्ट से जाता है, अतिरिक्त परिशुद्धता डिकोड के समय आपके serde में विफल होती है; पैसे के लिए `#[bee(sql_type = Raw("decimal(12,2)"))]` श्रेयस्कर।
> कंपाइल-टाइम अनुबंध: feature बंद होने पर दिनांक फ़ील्ड वाला मॉडल E0277 (`Value: From<NaiveDate>` असंतुष्ट) से विफल — चुपचाप गलत कॉलम कभी नहीं।

### शार्डिंग (bee_orm)

Key-required डेटाबेस/टेबल शार्डिंग। `#[bee(shard_key)]` रूटिंग फ़ील्ड को चिह्नित करता है और derive `Model::shard_key()` उत्पन्न करता है; `ShardedPools::for_model` किसी इंस्टेंस को सीधे रूट करता है, जबकि `for_key` अभी भी मान लेता है। क्वेरी में शार्ड कुंजी होनी चाहिए। रूटिंग से पहले शार्ड कुंजी कॉलर को ही देनी होगी: मैक्रो `auto` / `auto_now_add` / `auto_now` फ़ील्ड अस्वीकार करता है, और `None` / `NULL` को कुंजी न होना माना जाता है (`for_model` `ShardError` लौटाता है)। रूटिंग एक composite इंडेक्स `hash % (db_shards * table_shards)` का उपयोग करती है, इसलिए हर डेटाबेस/टेबल जोड़ी पहुंच योग्य है; क्रॉस-शार्ड क्वेरी, JOIN और ट्रांजैक्शन समर्थित नहीं हैं। SQL में डालने से पहले physical टेबल नाम `[A-Za-z_][A-Za-z0-9_]*` से मान्य किए जाते हैं। `ShardedPools::all_shards()` / `pools()` / `pool(index)` संचालन और समान-शार्ड लेनदेन के लिए अंतर्निहित पूल उजागर करते हैं; रूटिंग अभी भी `for_key` / `for_model` मांगती है। `sync_all` हर pool × table shard को क्रमिक रूप से माइग्रेट करता है; `sync_all_parallel` उन लक्ष्यों को समवर्ती रूप से चलाता है (पहले से लागू DDL वापस नहीं लिया जाता)। शार्डिंग का दुरुपयोग `OrmError::ShardError` लौटाता है, और `OrmError` `#[non_exhaustive]` है।

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

### पूल स्थिति और धीमी क्वेरी

तीनों बैकएंड `PoolStatus { max_size, size, available }` लौटाते हैं; mysql कोई आँकड़े उजागर नहीं करता, इसलिए अंतिम दो `None` हैं। `TimedDb` किसी भी `Db` को लपेटता है और callback तब कॉल करता है जब `query` / `execute` या `insert_returning` / `insert_returning_auto` read-back कम से कम सीमा तक समय लेता है; read-back मान बिना बदलाव के आगे भेजा जाता है और केवल उसका समय मापा जाता है।

```rust
use std::time::Duration;
use bee_orm::{Db, TimedDb};

let db = TimedDb::new(pool, Duration::from_millis(200), |sql, elapsed| {
    eprintln!("slow ({elapsed:?}): {sql}");
});
let rows = db.query("SELECT ...", &[]).await?;
```

## कॉन्फ़िगरेशन प्रबंधन (bee_config)

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

## स्टोरेज इंजन

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// सीमित-प्रतीक्षा वाला वितरित लॉक; watchdog संस्करण TTL का नवीनीकरण करता है
let lock = kv.lock();
if let Some(guard) = lock.acquire("job", Duration::from_secs(10), Duration::from_secs(2)).await? {
    // ... work ...
    guard.release().await?;
}

// TLS / ACL: क्रेडेंशियल और स्कीम URL में (feature `redis-tls`)
let kv = RedisStore::new("rediss://user:pass@redis.example:6380").await?;

// Sentinel / Cluster (features `redis-sentinel` / `redis-cluster`)
use bee_kv::{RedisSentinelStore, RedisClusterStore};
let kv = RedisSentinelStore::new("mymaster", &["redis://127.0.0.1:26379"], 16).await?;
let kv = RedisClusterStore::new(&["redis://127.0.0.1:7000"], 16).await?;

// memcached बैकएंड (feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> `RedisStore` एक `deadpool-redis` पूल रखता है: मारी गई कनेक्शन अगले checkout पर बदल दी जाती है (recycle PING); प्रत्येक कमांड 30 सेकंड तक सीमित है, और exhausted पूल विफल होने से पहले 30 सेकंड तक प्रतीक्षा करता है। `bee_kv::RedisLock` `SET NX PX` अधिग्रहण और Lua compare-and-delete रिलीज़ देता है; `acquire(key, ttl, wait)` सीमित प्रतीक्षा समाप्त होने तक अधिकतम हर 50ms पर पुनः प्रयास करता है, और `try_lock_with_watchdog` पृष्ठभूमि में हर TTL/3 पर TTL का नवीनीकरण करता है। `RedisOptions` पूल आकार और प्रतिक्रिया टाइमआउट सेट करता है, और `status()` / `RedisPoolStatus` पूल स्नैपशॉट पढ़ते हैं। `redis-tls` URL में ACL क्रेडेंशियल के साथ `rediss://` सक्षम करता है (`redis://user:pass@host`)।
>
> Sentinel / Cluster: `redis-sentinel` `RedisSentinelStore` / `RedisSentinelCache` प्रदान करता है; `redis-cluster` `RedisClusterStore` / `RedisClusterCache` प्रदान करता है। Sentinel URL क्रेडेंशियल/TLS केवल Sentinels पर लागू होते हैं; master कनेक्शन में इनमें से कोई नहीं होता, और पूल की कनेक्शन role-check के बिना PING-recycle होती हैं, इसलिए failover के बाद अभी भी पहुँच योग्य पुराना master सेवा देता रह सकता है (`deadpool-redis` 0.18 सीमा)। Cluster में `mget` / `mset` अनुक्रमिक एकल-कुंजी कमांड हैं (`CROSSSLOT` से बचने के लिए), और cluster कमांड `response_timeout` से सीमित नहीं हैं।
>
> memcached: `incr` पहले काउंटर बनाता है (न हो तो 0) फिर अंतर लागू करता है; काउंटर अहस्ताक्षरित हैं इसलिए घटाना 0 पर रुकता है (Redis ऋणात्मक जाता है); `expire(≤0)` हटाने के बराबर है।

**कैश बैकएंड (bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache` (feature `memcache`) का वही इंटरफ़ेस है; TTL `Some(0)` Redis बैकएंड में `DEL` से जाता है (`SET … EX 0` अस्वीकृत होता है), और memcached में भी हटाना माना जाता है (वहाँ 0 का अर्थ कभी समाप्त न होना है); गैर-संख्यात्मक मान पर `incr` सीरियलाइज़ेशन त्रुटि है।

**खोज इंजन (कार्यान्वित, opt-in):**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // feature `elasticsearch` आवश्यक
let engine = Elasticsearch::new("http://localhost:9200");   // सिंक्रोनस कंस्ट्रक्टर (`?` के बिना)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> बाकी ड्राइवर भी इसी तरह काम करते हैं और opt-in हैं: `opensearch`, `clickhouse` (समान नाम वाले feature)।

**ग्राफ़ डेटाबेस (कार्यान्वित, opt-in):**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // feature `neo4j` आवश्यक
let db = Neo4j::new("http://localhost:7474");   // सिंक्रोनस कंस्ट्रक्टर (`?` के बिना)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> बाकी ड्राइवर भी इसी तरह काम करते हैं और opt-in हैं: `nebulagraph`, `arangodb` (समान नाम वाले feature)।

**टाइम-सीरीज़ डेटाबेस (कार्यान्वित, opt-in):**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // feature `influxdb` आवश्यक
let tsdb = InfluxDB::new("http://localhost:8086");   // सिंक्रोनस कंस्ट्रक्टर (`?` के बिना)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> बाकी ड्राइवर भी इसी तरह काम करते हैं और opt-in हैं: `iotdb`, `questdb` (समान नाम वाले feature)।

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> बैकएंड कोई भी कैश जो `bee_cache::Cache` लागू करता हो: `MemoryCache` / `RedisCache` / `MemcacheCache`।

## लॉगिंग

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .init()?;
```

## टेम्पलेट

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI उपकरण

```bash
# प्रोजेक्ट बनाएँ (चलने योग्य स्कैफोल्ड उत्पन्न करता है: Cargo.toml + src/main.rs)
bee-rust new my-app

# कोड जनरेट करें
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# डेवलपमेंट रन (--watch src/ में बदलाव देखकर स्वतः रीस्टार्ट करता है)
bee-rust run
bee-rust run --watch

# पैकेज और डिप्लॉय (cargo build --release + dist/ में कॉपी)
bee-rust pack

# डेटाबेस माइग्रेशन (आपके crate में एंट्रीपॉइंट बनाकर चलाता है)
bee-rust migrate init    # src/bin/bee_migrate.rs बनाता है (मौजूद फ़ाइल को ओवरराइट नहीं करता)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> नोट: `pack --target` पैरामीटर एक आरक्षित इंटरफ़ेस है; वर्तमान पैकेजिंग प्रक्रिया लक्ष्य प्लेटफ़ॉर्म में भेद नहीं करती।
