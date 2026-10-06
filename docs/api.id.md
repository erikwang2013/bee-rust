<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Referensi API Beerust

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Kembali ke [README](README.id.md).

Dokumen ini adalah referensi API framework Beerust: penggunaan, contoh kode, dan penjelasan antarmuka setiap modul.

## Web Inti (bee_router)

### Mendefinisikan Controller

```rust
use bee_rust::prelude::*;

// Mendefinisikan controller
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Registrasi Rute

```rust
// Registrasi rute
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context menyediakan:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — output respons
- `ctx.redirect()` — pengalihan
- `ctx.abort()` — membatalkan permintaan
- `ctx.session` — akses sesi
- `ctx.params` — parameter jalur

## Deteksi Keamanan (fitur `security`)

Filter deteksi serangan berbasis [security-rust](https://crates.io/crates/security-rust), mencakup 27 jenis serangan seperti XSS, injeksi SQL, injeksi perintah, SSRF:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // 27 detektor semuanya aktif
```

Aktifkan di `Cargo.toml`:
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
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql berbagi bentuk yang sama

    // Skema — migrate::sync menghasilkan DDL non-destruktif dari metadata model
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — kunci primer `auto` ditetapkan basis data
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — filter berantai + eksekusi: all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // Relasi — children untuk has_many, belongs_to untuk induk; dengan FK aktif, hapus anak sebelum induk
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete adalah soft delete (membalik flag); with_deleted() / hard_delete() melewatinya
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

    // Transaksi — pegang koneksi dari `get()`; `CheckedConn` sqlite bersifat sinkron
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> Migrasi kini tersedia: `bee_orm::migrate` (`create_table` / `add_missing_columns` / `sync`) menghasilkan DDL sesuai dialek — membuat tabel dan menambah kolom yang belum ada, tidak pernah menghapus atau mengubah. Pembacaan relasi mencakup `belongs_to`, has_many (`children*`), dan many-to-many (`m2m`). `#[bee(fk = …)]` menghasilkan DDL kunci asing: postgres dan sqlite (build bundled) menegakkan referensi, dan mysql dapat mengaktifkan kunci asing tingkat tabel lewat opt-in (di bawah).

### Banyak-ke-banyak (`m2m`)

Dideklarasikan di tingkat struct dengan `#[bee(m2m(Target))]`; tabel join dibuat oleh `create_table` / `sync` (sinkronkan model target dulu, lalu model pendeklarasi):

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // tabel join user_tag, kolom user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // model target dulu
migrate::sync::<User, _>(&pool).await?;      // model pendeklarasi membuat tabel join

// kunci `auto` ditetapkan basis data: baca ulang instance sebelum menyentuh relasi
User { id: 0, name: "alice".into() }.insert(&pool).await?;
let user = User::query().filter_eq("name", "alice")?.one(&pool).await?.expect("inserted");
Tag { id: 0, name: "rust".into() }.insert(&pool).await?;
let tag = Tag::query().filter_eq("name", "rust")?.one(&pool).await?.expect("inserted");

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // attach duplikat -> error PK komposit
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // idempoten: panggilan kedua mengembalikan 0 baris
```

> Penggantian nama: `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`; tabel join hanya dibuat oleh `create_table` / `sync` — `add_missing_columns` tidak menyentuhnya.

### Kolom JSON

Field `serde_json::Value` dipetakan ke sqlite `TEXT` / postgres `JSONB` / mysql `JSON`:

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // wajib
    extra: Option<serde_json::Value>,  // nullable
}
```

> Semantik NULL: SQL `NULL` didekode menjadi `None`; dokumen JSON `null` yang tersimpan menjadi `Some(Json::Null)` — keduanya berbeda.

### Kunci asing tingkat tabel MySQL (opt-in)

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // hanya mysql
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> Jika aktif, tabel baru dan kolom yang ditambahkan membawa `FOREIGN KEY` tingkat tabel (nama constraint `{table}_{column}_fk`); baris yatim yang sudah ada membuat `ADD CONSTRAINT` gagal (tanpa lewati senyap, tanpa retrofit kolom yang ada); di pg / sqlite opsi ini tanpa efek (inline sudah menegakkan).

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls memakai root webpki bawaan; untuk konfigurasi rustls kustom gunakan connect_tls_with:
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig (reekspor, versi sama dengan crate)
```

## Manajemen Konfigurasi (bee_config)

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

## Mesin Penyimpanan

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// backend memcached (feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> Di dalam `RedisStore` ada `ConnectionManager`: koneksi terputus tersambung ulang otomatis (sesuai permintaan — tanpa thread latar belakang — dengan backoff eksponensial dan jitter) — perintah yang kena putus akan error, perintah berikutnya menunggu koneksi baru. memcached: `incr` membuat counter dulu (0 jika belum ada) lalu menerapkan delta; counter tak bertanda, jadi pengurangan berhenti di 0 (Redis turun ke negatif); `expire(≤0)` sama dengan menghapus.

**Backend cache (bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache` (feature `memcache`) berbagi antarmuka yang sama; TTL `Some(0)` lewat `DEL` di backend Redis (`SET … EX 0` ditolak), dan diperlakukan sebagai hapus juga di memcached (di sana 0 berarti tidak pernah kedaluwarsa); `incr` pada nilai non-numerik adalah error serialisasi.

**Mesin pencarian (terimplementasi, opt-in):**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // memerlukan feature `elasticsearch`
let engine = Elasticsearch::new("http://localhost:9200");   // konstruktor sinkron (tanpa `?`)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> Driver lain bekerja dengan cara yang sama dan juga opt-in: `opensearch`, `clickhouse` (feature dengan nama sama).

**Basis data graf (terimplementasi, opt-in):**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // memerlukan feature `neo4j`
let db = Neo4j::new("http://localhost:7474");   // konstruktor sinkron (tanpa `?`)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> Driver lain bekerja dengan cara yang sama dan juga opt-in: `nebulagraph`, `arangodb` (feature dengan nama sama).

**Basis data deret waktu (terimplementasi, opt-in):**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // memerlukan feature `influxdb`
let tsdb = InfluxDB::new("http://localhost:8086");   // konstruktor sinkron (tanpa `?`)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> Driver lain bekerja dengan cara yang sama dan juga opt-in: `iotdb`, `questdb` (feature dengan nama sama).

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> Backend-nya adalah cache apa pun yang mengimplementasikan `bee_cache::Cache`: `MemoryCache` / `RedisCache` / `MemcacheCache`.

## Logging

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## Template

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## Alat CLI

```bash
# Buat proyek (menghasilkan scaffolding yang dapat dijalankan: Cargo.toml + src/main.rs)
bee-rust new my-app

# Generasi kode
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Run development (--watch memantau perubahan di src/ dan restart otomatis)
bee-rust run
bee-rust run --watch

# Paket untuk deployment (cargo build --release + salin ke dist/)
bee-rust pack

# Migrasi basis data (membuat titik masuk di crate Anda lalu menjalankannya)
bee-rust migrate init    # membuat src/bin/bee_migrate.rs (menolak menimpa file yang ada)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> Catatan: parameter `pack --target` adalah antarmuka cadangan; proses pengemasan saat ini tidak membedakan platform target.
