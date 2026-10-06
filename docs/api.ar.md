<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust مرجع API

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

العودة إلى [README](README.ar.md).

هذا المستند هو مرجع API لإطار Beerust: استخدام كل وحدة، وأمثلة الكود، وشرح الواجهات.

## نواة الويب (bee_router)

### تعريف متحكم

```rust
use bee_rust::prelude::*;

// تعريف متحكم
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### تسجيل المسارات

```rust
// تسجيل المسارات
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**يوفر Context:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — مخرجات الاستجابة
- `ctx.redirect()` — إعادة التوجيه
- `ctx.abort()` — مقاطعة الطلب
- `ctx.session` — الوصول إلى الجلسة
- `ctx.params` — معاملات المسار

## كشف الأمان (feature: `security`)

فلتر كشف الهجمات مبني على [security-rust](https://crates.io/crates/security-rust)، يغطي 27 نوعًا من الهجمات مثل XSS و SQL Injection و Command Injection و SSRF:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // تفعيل أدوات الكشف الـ 27 كلها
```

التمكين في `Cargo.toml`:
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
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql بالشكل نفسه

    // المخطط — يبني migrate::sync عبارات DDL غير مُدمِّرة من بيانات النموذج الوصفية
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — المفتاح الأساسي `auto` تُسنِده قاعدة البيانات
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — مرشّحات متسلسلة + تنفيذ: all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // العلاقات — children لـ has_many وbelongs_to للصف الأب؛ مع فرض FK احذف الابن قبل الأب
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — الحذف عبر delete منطقي (قلب العلامة)؛ with_deleted() / hard_delete() يتجاوزانه
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

    // المعاملات — احتفظ باتصال من `get()`؛ ‏`CheckedConn` في sqlite متزامن
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> الترحيلات متاحة الآن: تولّد `bee_orm::migrate` (`create_table` / `add_missing_columns` / `sync`) عبارات DDL حسب اللهجة — إنشاء الجداول وإضافة الأعمدة الناقصة فقط، دون حذف أو تعديل. قراءة العلاقات تشمل `belongs_to` وhas_many (`children*`) وmany-to-many (`m2m`). ينتج `#[bee(fk = …)]` تعريف مفتاح أجنبي: postgres وsqlite (بناء bundled) يفرضان المرجع، ويمكن لـmysql تفعيل مفاتيح أجنبية على مستوى الجدول عبر opt-in (أدناه).

### متعدد إلى متعدد (`m2m`)

يُعلَن على مستوى البنية بـ `#[bee(m2m(Target))]`؛ يُنشئ `create_table` / `sync` جدول الربط (زامن نموذج الهدف أولاً ثم المعلِن):

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // جدول الربط user_tag، الأعمدة user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // نموذج الهدف أولاً
migrate::sync::<User, _>(&pool).await?;      // النموذج المعلن ينشئ جدول الربط

// مفاتيح `auto` تعيّنها قاعدة البيانات: أعد قراءة النسخة قبل لمس العلاقات
User { id: 0, name: "alice".into() }.insert(&pool).await?;
let user = User::query().filter_eq("name", "alice")?.one(&pool).await?.expect("inserted");
Tag { id: 0, name: "rust".into() }.insert(&pool).await?;
let tag = Tag::query().filter_eq("name", "rust")?.one(&pool).await?.expect("inserted");

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // attach مكرر -> خطأ المفتاح المركب
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // عديم الأثر: النداء الثاني يعيد 0 صفوف
```

> إعادة التسمية: `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`؛ جداول الربط تُنشأ فقط عبر `create_table` / `sync` — و`add_missing_columns` لا يمسها.

### أعمدة JSON

تُعيَّن حقول `serde_json::Value` إلى sqlite `TEXT` / postgres `JSONB` / mysql `JSON`:

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // مطلوب
    extra: Option<serde_json::Value>,  // nullable
}
```

> دلالات NULL: يُفكَّر SQL `NULL` إلى `None`، أما مستند JSON `null` المخزَّن فيُفك إلى `Some(Json::Null)` — وهما مختلفان.

### مفاتيح أجنبية على مستوى الجدول في MySQL (opt-in)

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // mysql فقط
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> عند التفعيل تحمل الجداول الجديدة والأعمدة المضافة `FOREIGN KEY` على مستوى الجدول (اسم القيد `{table}_{column}_fk`)؛ الصفوف اليتيمة الموجودة مسبقًا تُفشل `ADD CONSTRAINT` (دون تخطٍّ صامت ودون retrofit للأعمدة الحالية)؛ في pg / sqlite هذا الخيار بلا أثر (inline يفرضه بالفعل).

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls يستخدم جذور webpki المضمّنة؛ لإعداد rustls مخصص استخدم connect_tls_with:
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig (معاد تصديره، نفس إصدار الـcrate)
```

### أنواع التاريخ و Decimal

حقول التاريخ و Decimal محكومة بـ feature: فعِّل `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) و`rust_decimal` (`Decimal`) في `bee_orm`؛ وعبر `bee_rust` فميزات التمرير هي `orm-chrono` / `orm-rust_decimal`.

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date و‏DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal (متطابق على الواجهات الثلاث)
}

// الكتابة: يتحول الحقل إلى متغير Value المناسب؛ القراءة: تُفكَّك الخلية عبر serde
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // "1.50" يعود بقيمة مساوية
```

> المدعوم: `NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`. خارج النطاق — الماكرو لا يولّد تعيين SQL، استخدم `#[bee(sql_type = "…")]`: `NaiveTime` / `DateTime<Local>` / `FixedOffset` / `DateTime` المجرد، ودقة النانوثانية.
> التخزين: sqlite دائمًا TEXT (`typeof(col)` تساوي `text`؛ وDecimal كذلك — إعلان DECIMAL يمنح ألفة NUMERIC فيبتلع "1.50" ليصبح REAL 1.5)؛ mysql يستخدم `datetime(6)` / `timestamp(6)` بميكروثوانٍ — TIMESTAMP يُقرأ كـ UTC (RFC3339 مع `Z`؛ والذهاب والإياب الصارم يتطلب منطقة زمنية للجلسة UTC)، وDATETIME يبقى naive بلا لاحقة، والتوزيع حسب نوع العمود؛ وpg `timestamptz` يُطبَّع إلى UTC.
> سقف الدقة: في pg يُقرأ `numeric` فوق 29 رقمًا معنويًا كـ `NULL` (يرفضه FromSql الأصلي)؛ وفي mysql يمر النص، والزيادة تفشل عند فك الترميز في serde لديك؛ للمبالغ يُفضَّل `#[bee(sql_type = Raw("decimal(12,2)"))]`.
> عقد الترجمة: مع إيقاف feature يفشل نموذج يحمل حقل تاريخ بـ E0277 (`Value: From<NaiveDate>` غير متحقق) — لا عمود خاطئ بصمت.

## إدارة التكوين (bee_config)

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

## محركات التخزين

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// خلفية memcached (feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> داخل `RedisStore` يوجد `ConnectionManager`: الاتصال المقطوع يُعاد تلقائيًا (عند الطلب — دون خيط خلفي — مع تراجع أُسّي واهتزاز عشوائي) — الأمر الذي يصادف القطع يفشل، والتالي ينتظر الاتصال الجديد. memcached: ينشئ `incr` العدّاد أولاً (0 إن لم يوجد) ثم يطبّق الفرق؛ العدّادات غير موقّعة لذا يتوقف التنقيص عند 0 (في Redis ينزل إلى السالب)؛ `expire(≤0)` يعادل الحذف.

**خلفيات التخزين المؤقت (bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache` (feature `memcache`) له الواجهة نفسها؛ TTL `Some(0)` يمر عبر `DEL` في خلفية Redis (رفض `SET … EX 0`)، ويُعامل أيضًا كحذف في memcached (حيث 0 تعني عدم الانتهاء أبدًا)؛ و`incr` على قيمة غير رقمية خطأ تسلسل.

**محرك البحث (مُنفَّذ، opt-in):**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // يتطلب feature `elasticsearch`
let engine = Elasticsearch::new("http://localhost:9200");   // إنشاء متزامن (بدون `?`)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> بقية المشغلات تعمل بالطريقة نفسها وهي opt-in أيضًا: `opensearch`, `clickhouse` (feature بالأسماء نفسها).

**قاعدة البيانات الرسومية (مُنفَّذة، opt-in):**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // يتطلب feature `neo4j`
let db = Neo4j::new("http://localhost:7474");   // إنشاء متزامن (بدون `?`)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> بقية المشغلات تعمل بالطريقة نفسها وهي opt-in أيضًا: `nebulagraph`, `arangodb` (feature بالأسماء نفسها).

**قاعدة بيانات السلاسل الزمنية (مُنفَّذة، opt-in):**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // يتطلب feature `influxdb`
let tsdb = InfluxDB::new("http://localhost:8086");   // إنشاء متزامن (بدون `?`)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> بقية المشغلات تعمل بالطريقة نفسها وهي opt-in أيضًا: `iotdb`, `questdb` (feature بالأسماء نفسها).

## الجلسات

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> الخلفية أي ذاكرة مؤقتة تُنفِّذ `bee_cache::Cache`: `MemoryCache` / `RedisCache` / `MemcacheCache`.

## السجلات

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## القوالب

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## أدوات CLI

```bash
# إنشاء مشروع (يولد سقالة قابلة للتشغيل: Cargo.toml + src/main.rs)
bee-rust new my-app

# توليد الكود
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# تشغيل تطويري (--watch يراقب تغييرات src/ ويعيد التشغيل تلقائيًا)
bee-rust run
bee-rust run --watch

# تغليف ونشر (cargo build --release + نسخ إلى dist/)
bee-rust pack

# ترحيل قاعدة البيانات (يولّد نقطة الدخول في الـcrate ثم يشغّلها)
bee-rust migrate init    # ينشئ src/bin/bee_migrate.rs (لا يستبدل ملفًا موجودًا)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> ملاحظة: معامل `pack --target` واجهة محجوزة، وعملية التغليف الحالية لا تفرق منصات الهدف.
