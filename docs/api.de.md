<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API-Referenz

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Zurück zur [README](README.de.md).

Dieses Dokument ist die API-Referenz des Beerust-Frameworks: Verwendung der einzelnen Module, Codebeispiele und Schnittstellenbeschreibungen.

## Web-Kern (bee_router)

### Controller definieren

```rust
use async_trait::async_trait;  // Cargo.toml: async-trait = "0.1"
use bee_rust::bee_router::context::RouterError;
use bee_rust::prelude::*;

// Controller definieren
struct UserController;

#[async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Routenregistrierung

```rust
// Handler sind axum-Handler — ohne State einfache async fns
async fn list_users() -> &'static str { "[]" }
async fn create_user() -> &'static str { "created" }

// Routenregistrierung
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users", list_users)
          .post("/users", create_user);
    });
```

> Pfadsyntax wie in axum 0.8: dynamische Segmente schreibt man `{name}` (z. B. `/users/{id}`); `:name` panikt beim Build (verifiziert). Das `ns`-Präfix wird direkt mit dem gruppeninternen Pfad verkettet: `ns("/api/v1", …)` + `/users` → `/api/v1/users`; ein leeres Präfix (`ns("", …)`) ist zulässig — der Pfad in der Gruppe ist dann der volle Pfad.

### Handler mit State

Handler sind axum-Handler: gemeinsamer State kommt über `State` und wird mit `Router::with_state` verdrahtet (`axum` muss eine direkte Abhängigkeit sein — die vom CLI-Scaffold erzeugte Cargo.toml enthält sie). Zustandslose Routen nutzen `build()`; sobald ein Handler `State` nimmt, braucht es `with_state` vor `axum::serve`. Die typische `Pool + Cache`-Form:

```rust
use std::sync::Arc;
use axum::extract::State;
use bee_rust::bee_cache::MemoryCache;
use bee_rust::bee_orm::pool::sqlite::Pool;
use bee_rust::prelude::*;

#[derive(Model)]
#[bee(crate = "bee_rust::bee_orm", table = "users")]  // Schreibweise mit nur bee_rust
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
    let users = User::query().all(&state.pool).await.map_err(|e| e.to_string())?;  // eine Abfrage
    Ok(format!("{} users", users.len()))
}

let app = Router::new()
    .ns("/api/v1", |ns| ns.get("/users", list_users))
    .with_state(AppState { pool, cache: Arc::new(MemoryCache::new()) });
```

### Context-API

**Context bietet:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — Antwortausgabe
- `ctx.redirect()` — Weiterleitung
- `ctx.abort()` — Anfrage abbrechen
- `ctx.session` — Session-Zugriff
- `ctx.params` — Pfadparameter

### Latenzbudget und Cache-Fallback

`Router::latency` ist ein Opt-in-Latenzbudget für GET/HEAD. Der Standardwert beträgt insgesamt **6 ms**: 4 ms für den Handler plus 2 ms Reserve für den Cache-Fallback, was Planungsspielraum unter dem P99-Ziel von 10 ms lässt; überschreibbar mit `.deadline(10ms)`. Bei Ablauf liest es den Cache, spielt eine öffentliche Antwort mit `X-Bee-Cache: stale` / `Age` erneut aus oder liefert 504 `"request timed out"` + `X-Bee-Deadline: exceeded`. Schreibmethoden fallen nicht unter dieses Budget und behalten das bestehende 30 s/408-Verhalten.

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

Cache-Vertrag:
- Nur **GET 200**-Antworten werden gespeichert; `Cache-Control` muss `public` enthalten und darf kein `private`/`no-store`/`no-cache` enthalten, es darf weder `Set-Cookie` noch `Vary` geben, und `size_hint().exact()` des Bodys muss ≤ 64 KiB sein. HEAD verwendet den GET-Schlüssel erneut, speichert aber nie.
- Anfragen mit `Authorization` / `Range` / `Cache-Control: no-store` / `no-cache` umgehen den Cache und liefern bei Timeout 504.
- Die TTL ist der kleinere Wert aus der Konfiguration und dem `max-age` der Antwort; Schreibvorgänge invalidieren den Cache nie, daher können Antworten bis zu eine TTL lang veraltet sein.
- Ein GET, dessen Deadline abläuft, läuft in einer **begrenzten Hintergrundaufgabe** weiter (standardmäßig 128; `background_limit(0)` deaktiviert sie), um den Cache für die nächste Anfrage zu wärmen.
- Die Deadline begrenzt die Antwort, nicht die serverseitige Arbeit: SQLite-/Postgres-/MySQL-/Redis-Arbeit wird nicht abgebrochen. Long-Poll-, SSE- und Download-Routen sollten einen Router **ohne** `latency` verwenden.
- `LatencyStats::snapshot()` meldet Anfragen / Timeouts / Fallback-Treffer sowie die Obergrenzen der P50/P99-Buckets und ein exaktes `max_us`.

## Sicherheitserkennung (`security`-Feature)

Ein auf [security-rust](https://crates.io/crates/security-rust) basierender Angriffserkennungs-Filter, der 27 Angriffstypen wie XSS, SQL-Injection, Command-Injection und SSRF abdeckt:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // alle 27 Detektoren aktiv
```

In `Cargo.toml` aktivieren:
```toml
bee_rust = { features = ["security"] }
```

## ORM (bee_orm)

> Mit nur `bee_rust`: entweder `#[bee(crate = "bee_rust::bee_orm")]` am Modell (bevorzugt) oder `use bee_rust::bee_orm;` oben im Modul — die Expansion erzeugt `bee_orm::…`-Pfade, eines von beidem lässt sie auflösen; übrige `bee_orm::`-Pfade werden zu `bee_rust::bee_orm::` (die Beispiele unten sind für eine direkte `bee_orm`-Abhängigkeit geschrieben).

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
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql haben dieselbe Form

    // Schema — migrate::sync erzeugt nicht-destruktives DDL aus den Modell-Metadaten
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — insert() liefert die betroffenen Zeilen
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — Kettenfilter plus Ausführung: all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // Beziehungen — children für has_many, belongs_to für die Elternzeile; bei erzwungenem FK erst das Kind, dann die Elternzeile löschen
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete löscht soft (setzt die Markierung); with_deleted() / hard_delete() umgehen das
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

    // Transaktionen — Verbindung über `get()` halten; `CheckedConn` ist bei sqlite synchron
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> Zwei Schreibformen: `insert()` liefert die betroffenen Zeilen (`u64`); `create()` fügt ein und liest die Zeile zurück — die Instanz kommt mit gefülltem `auto`-Primärschlüssel / Defaults zurück (`NotFound`, wenn nicht rücklesbar) — für alles, was den Schlüssel braucht, direkt `create()` nehmen (siehe m2m-Beispiel unten).

> Migrationen sind verfügbar: `bee_orm::migrate` (`create_table` / `add_missing_columns` / `sync`) erzeugt dialektgerechtes DDL — Tabellen anlegen und fehlende Spalten ergänzen, niemals löschen oder ändern. Beziehungs-Leseabfragen umfassen `belongs_to`, has_many (`children*`) und many-to-many (`m2m`). `#[bee(fk = …)]` erzeugt FK-DDL: postgres und sqlite (bundled-Build) erzwingen den Fremdschlüssel, mysql kann Tabellen-Fremdschlüssel per Opt-in aktivieren (siehe unten).

### Many-to-many (`m2m`)

Strukturebene `#[bee(m2m(Target))]`; die Join-Tabelle entsteht über `create_table` / `sync` (zuerst das Zielmodell synchronisieren, dann das deklarierende):

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // Join-Tabelle user_tag, Spalten user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // Zielmodell zuerst
migrate::sync::<User, _>(&pool).await?;      // deklarierendes Modell erstellt die Join-Tabelle

// create() fügt ein und liest die Instanz zurück (auto-PK gefüllt), bereit für Relationen
let user = User { id: 0, name: "alice".into() }.create(&pool).await?;
let tag = Tag { id: 0, name: "rust".into() }.create(&pool).await?;

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // doppeltes attach -> Composite-PK-Fehler
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // idempotent: zweiter Aufruf liefert 0 Zeilen
```

> Namens-Overrides: `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`; Join-Tabellen entstehen nur über `create_table` / `sync` — `add_missing_columns` rührt sie nie an.

### JSON-Spalten

`serde_json::Value`-Felder werden zu sqlite `TEXT` / postgres `JSONB` / mysql `JSON`:

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // Pflichtfeld
    extra: Option<serde_json::Value>,  // nullable
}
```

> NULL-Semantik: SQL `NULL` dekodiert zu `None`; ein gespeichertes JSON-`null`-Dokument zu `Some(Json::Null)` — beides ist verschieden.

### MySQL-Tabellen-Fremdschlüssel (Opt-in)

> MySQL 9 und MariaDB 10.11+ erzwingen spaltenweises `REFERENCES`; MySQL 5.7/8.x parsen und ignorieren es. Referenzierte Tabellen zuerst anlegen.

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // nur mysql
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> Aktiviert tragen frische Tabellen und neu hinzugefügte Spalten einen Tabellen-`FOREIGN KEY` (Constraint-Name `{table}_{column}_fk`); vorhandene verwaiste Zeilen lassen `ADD CONSTRAINT` scheitern (kein stilles Überspringen, kein Retrofit bestehender Spalten); unter pg / sqlite ist die Option wirkungslos (inline erzwingt bereits).

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls nutzt die gebündelten webpki-Roots; für eigene rustls-Konfiguration connect_tls_with:
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig (Re-Export, gleiche Version wie die Crate)
```

### Datums- und Decimal-Typen

Datums- und Decimal-Felder sind feature-gesteuert: `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) und `rust_decimal` (`Decimal`) in `bee_orm` aktivieren; über `bee_rust` heißen die Weiterleitungs-Features `orm-chrono` / `orm-rust_decimal`.

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date; DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal (auf allen drei Backends gleich)
}

// Schreiben: das Feld wird zur passenden Value-Variante; Lesen: die Zelle dekodiert per serde zurück
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // "1.50" kommt numerisch gleich zurück
```

> Unterstützt: `NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`. Außerhalb des Umfangs — das Makro liefert keine SQL-Zuordnung, `#[bee(sql_type = "…")]` ist der Ausweg: `NaiveTime` / `DateTime<Local>` / `FixedOffset` / nacktes `DateTime` sowie Nanosekunden.
> Speicherung: sqlite immer TEXT (`typeof(col)` ist `text`; auch Decimal — eine DECIMAL-Deklaration bekäme NUMERIC-Affinität und verschluckt "1.50" zu REAL 1.5); mysql nutzt `datetime(6)` / `timestamp(6)` mit Mikrosekunden — TIMESTAMP liest als UTC zurück (RFC3339 mit `Z`; strikter Round-Trip erfordert Sitzungszeitzone UTC), DATETIME bleibt naiv ohne Suffix, verteilt nach Spaltentyp; pg `timestamptz` normalisiert auf UTC.
> Präzisionsgrenze: pg `numeric` über 29 signifikante Stellen liest als `NULL` zurück (natives FromSql lehnt ab); mysql läuft über Text, zu viel Präzision fällt erst beim Decode in deinem serde auf; für Geld besser `#[bee(sql_type = Raw("decimal(12,2)"))]`.
> Compile-Zeit-Vertrag: ohne Feature scheitert ein Modell mit Datumsfeld mit E0277 (`Value: From<NaiveDate>` nicht erfüllt) — nie eine still falsche Spalte.

### Sharding (bee_orm)

Schlüsselpflichtiges Datenbank-/Tabellen-Sharding. `#[bee(shard_key)]` markiert das Routing-Feld und die Ableitung erzeugt `Model::shard_key()`; `ShardedPools::for_model` routet eine Instanz direkt, während `for_key` weiterhin den Wert entgegennimmt. Abfragen müssen den Shard-Schlüssel tragen. Der Shard-Schlüssel muss vor dem Routing vom Aufrufer geliefert werden: `auto` / `auto_now_add` / `auto_now`-Felder werden vom Makro abgelehnt, und `None` / `NULL` gilt als kein Schlüssel (`for_model` liefert `ShardError`). Das Routing nutzt einen zusammengesetzten Index `hash % (db_shards * table_shards)`, sodass jedes Datenbank-/Tabellenpaar erreichbar ist; Abfragen, JOINs und Transaktionen über mehrere Shards werden nicht unterstützt. Physische Tabellennamen werden gegen `[A-Za-z_][A-Za-z0-9_]*` geprüft, bevor sie in SQL eingesetzt werden. `ShardedPools::all_shards()` / `pools()` / `pool(index)` legen die zugrunde liegenden Pools für den Betrieb und Shard-interne Transaktionen offen; das Routing erfordert weiterhin `for_key` / `for_model`. `sync_all` migriert jeden Pool × Tabellen-Shard seriell; `sync_all_parallel` führt diese Ziele nebenläufig aus (bereits angewendetes DDL wird nicht zurückgerollt). Sharding-Missbrauch liefert `OrmError::ShardError`, und `OrmError` ist `#[non_exhaustive]`.

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

### Pool-Status und langsame Abfragen

Alle drei Backends liefern `PoolStatus { max_size, size, available }`; mysql legt keine Statistiken offen, daher sind die letzten beiden `None`. `TimedDb` umhüllt jedes `Db` und ruft einen Callback auf, wenn `query` / `execute` oder ein `insert_returning` / `insert_returning_auto`-Read-Back mindestens die Schwelle erreicht; der Read-Back-Wert wird unverändert durchgereicht und nur gemessen.

```rust
use std::time::Duration;
use bee_orm::{Db, TimedDb};

let db = TimedDb::new(pool, Duration::from_millis(200), |sql, elapsed| {
    eprintln!("slow ({elapsed:?}): {sql}");
});
let rows = db.query("SELECT ...", &[]).await?;
```

## Konfigurationsverwaltung (bee_config)

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

## Speicher-Engines

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// verteiltes Lock mit begrenzter Wartezeit; die Watchdog-Variante erneuert die TTL
let lock = kv.lock();
if let Some(guard) = lock.acquire("job", Duration::from_secs(10), Duration::from_secs(2)).await? {
    // ... work ...
    guard.release().await?;
}

// TLS / ACL: Zugangsdaten und Schema stehen in der URL (Feature `redis-tls`)
let kv = RedisStore::new("rediss://user:pass@redis.example:6380").await?;

// Sentinel / Cluster (Features `redis-sentinel` / `redis-cluster`)
use bee_kv::{RedisSentinelStore, RedisClusterStore};
let kv = RedisSentinelStore::new("mymaster", &["redis://127.0.0.1:26379"], 16).await?;
let kv = RedisClusterStore::new(&["redis://127.0.0.1:7000"], 16).await?;

// memcached-Backend (Feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> `RedisStore` hält einen `deadpool-redis`-Pool: Eine getötete Verbindung wird beim nächsten Checkout ersetzt (Recycle-PING); jeder Befehl ist auf 30 Sekunden begrenzt, und ein erschöpfter Pool wartet bis zu 30 Sekunden, bevor er fehlschlägt. `bee_kv::RedisLock` bietet `SET NX PX`-Erwerb und Lua-Compare-and-Delete-Freigabe; `acquire(key, ttl, wait)` wiederholt höchstens alle 50 ms, bis die begrenzte Wartezeit abläuft, und `try_lock_with_watchdog` erneuert die TTL im Hintergrund alle TTL/3. `RedisOptions` setzt Poolgröße und Antwort-Timeout, und `status()` / `RedisPoolStatus` lesen den Pool-Snapshot. `redis-tls` aktiviert `rediss://` mit ACL-Zugangsdaten in der URL (`redis://user:pass@host`).
>
> Sentinel / Cluster: `redis-sentinel` bietet `RedisSentinelStore` / `RedisSentinelCache`; `redis-cluster` bietet `RedisClusterStore` / `RedisClusterCache`. URL-Zugangsdaten/TLS für Sentinel gelten nur für die Sentinels; Master-Verbindungen tragen weder noch, und gepoolte Verbindungen werden ohne Rollenprüfung per PING recycelt, sodass nach einem Failover ein noch erreichbarer alter Master weiter bedienen kann (eine Einschränkung von `deadpool-redis` 0.18). Cluster-`mget` / `mset` sind sequenzielle Einzelschlüssel-Befehle (zur Vermeidung von `CROSSSLOT`), und Cluster-Befehle sind nicht durch `response_timeout` begrenzt.
>
> memcached: `incr` legt den Zähler zuerst an (0, wenn nicht vorhanden) und wendet dann das Delta an; Zähler sind vorzeichenlos, Dekrementieren bleibt bei 0 stehen (Redis wird negativ); `expire(≤0)` löscht den Schlüssel.

**Cache-Backends (bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // Feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache` (Feature `memcache`) hat dieselbe Schnittstelle; TTL `Some(0)` läuft im Redis-Backend über `DEL` (`SET … EX 0` wird abgelehnt) und gilt auch bei memcached als Löschen (dort bedeutet 0 nie ablaufen); `incr` auf einen nicht-numerischen Wert ist ein Serialisierungsfehler.

**Suchmaschine (implementiert, opt-in):**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // benötigt Feature `elasticsearch`
let engine = Elasticsearch::new("http://localhost:9200");   // synchroner Konstruktor (kein `?`)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> Die übrigen Treiber funktionieren genauso und sind ebenfalls opt-in: `opensearch`, `clickhouse` (gleichnamige Features).

**Graphdatenbank (implementiert, opt-in):**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // benötigt Feature `neo4j`
let db = Neo4j::new("http://localhost:7474");   // synchroner Konstruktor (kein `?`)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> Die übrigen Treiber funktionieren genauso und sind ebenfalls opt-in: `nebulagraph`, `arangodb` (gleichnamige Features).

**Zeitreihendatenbank (implementiert, opt-in):**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // benötigt Feature `influxdb`
let tsdb = InfluxDB::new("http://localhost:8086");   // synchroner Konstruktor (kein `?`)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> Die übrigen Treiber funktionieren genauso und sind ebenfalls opt-in: `iotdb`, `questdb` (gleichnamige Features).

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> Das Backend ist jeder Cache, der `bee_cache::Cache` implementiert: `MemoryCache` / `RedisCache` / `MemcacheCache`.

## Logging

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .init()?;
```

## Templates

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI-Werkzeuge

```bash
# Projekt erstellen (erzeugt lauffähiges Scaffolding: Cargo.toml + src/main.rs)
bee-rust new my-app

# Code generieren
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Entwicklungsbetrieb (--watch überwacht src/ und startet bei Änderungen automatisch neu)
bee-rust run
bee-rust run --watch

# Build und Paketierung (cargo build --release + Kopieren nach dist/)
bee-rust pack

# Datenbankmigration (Einstiegspunkt in der eigenen Crate erzeugen und ausführen)
bee-rust migrate init    # erzeugt src/bin/bee_migrate.rs (überschreibt eine vorhandene Datei nicht)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> Hinweis: Der Parameter `pack --target` ist eine reservierte Schnittstelle; der aktuelle Paketierungsprozess unterscheidet nicht zwischen Zielplattformen.
