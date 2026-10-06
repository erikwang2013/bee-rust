<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Referencia de la API de Beerust

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Volver al [README](README.es.md).

Este documento es la referencia de la API del framework Beerust: el uso de cada módulo, los ejemplos de código y la descripción de las interfaces.

## Núcleo web (bee_router)

### Definir un controlador

```rust
use async_trait::async_trait;  // Cargo.toml: async-trait = "0.1"
use bee_rust::bee_router::context::RouterError;
use bee_rust::prelude::*;

// Definir el controlador
struct UserController;

#[async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Registro de rutas

```rust
// los handlers son handlers de axum — async fns simples sin estado
async fn list_users() -> &'static str { "[]" }
async fn create_user() -> &'static str { "created" }

// Registro de rutas
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users", list_users)
          .post("/users", create_user);
    });
```

> Sintaxis de rutas igual que axum 0.8: los segmentos dinámicos se escriben `{name}` (p. ej. `/users/{id}`); `:name` entra en pánico al construir el router (verificado). El prefijo de `ns` se concatena tal cual con la ruta del grupo: `ns("/api/v1", …)` + `/users` → `/api/v1/users`; un prefijo vacío (`ns("", …)`) es válido — la ruta del grupo es entonces la ruta completa.

### Handlers con estado

Los handlers son handlers de axum: el estado compartido se inyecta con `State` y se conecta con `Router::with_state` (`axum` debe ser dependencia directa — el Cargo.toml que genera el scaffold del CLI ya la incluye). Las rutas sin estado usan `build()`; en cuanto un handler toma `State`, hace falta `with_state` antes de `axum::serve`. La forma típica `Pool + Cache`:

```rust
use std::sync::Arc;
use axum::extract::State;
use bee_rust::bee_cache::MemoryCache;
use bee_rust::bee_orm::pool::sqlite::Pool;
use bee_rust::prelude::*;

#[derive(Model)]
#[bee(crate = "bee_rust::bee_orm", table = "users")]  // escritura con solo bee_rust
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
    let users = User::query().all(&state.pool).await.map_err(|e| e.to_string())?;  // una consulta
    Ok(format!("{} users", users.len()))
}

let app = Router::new()
    .ns("/api/v1", |ns| ns.get("/users", list_users))
    .with_state(AppState { pool, cache: Arc::new(MemoryCache::new()) });
```

### API de Context

**`Context` proporciona:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — salida de respuestas
- `ctx.redirect()` — redirección
- `ctx.abort()` — interrumpe la petición
- `ctx.session` — acceso a la sesión
- `ctx.params` — parámetros de ruta

## Detección de seguridad (feature `security`)

Filtro de detección de ataques basado en [security-rust](https://crates.io/crates/security-rust) que cubre 27 tipos de ataques, como XSS, inyección SQL, inyección de comandos y SSRF:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // Los 27 detectores activados
```

Para activarlo en `Cargo.toml`:
```toml
bee_rust = { features = ["security"] }
```

## ORM (bee_orm)

> Con solo `bee_rust`: añade `#[bee(crate = "bee_rust::bee_orm")]` al modelo (preferido), o `use bee_rust::bee_orm;` al inicio del módulo — la expansión emite rutas `bee_orm::…` y cualquiera de las dos las resuelve; el resto de rutas `bee_orm::` pasan a `bee_rust::bee_orm::` (los ejemplos de abajo están escritos para dependencia directa de `bee_orm`).

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
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql comparten la misma forma

    // Esquema — migrate::sync genera DDL no destructivo a partir de los metadatos del modelo
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — insert() devuelve las filas afectadas
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — filtros encadenados + ejecución: all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // Relaciones — children para has_many, belongs_to para el padre; con FK activo, borra el hijo antes que el padre
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete es lógico (cambia la marca); with_deleted() / hard_delete() lo eluden
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

    // Transacciones — retén una conexión de `get()`; el `CheckedConn` de sqlite es síncrono
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> Dos formas de escritura: `insert()` devuelve filas afectadas (`u64`); `create()` inserta y relee la fila, devolviendo la instancia con la clave `auto` / los valores por defecto ya rellenos (`NotFound` si no se puede releer) — usa `create()` siempre que necesites la clave (ver el ejemplo m2m abajo).

> Las migraciones ya están disponibles: `bee_orm::migrate` (`create_table` / `add_missing_columns` / `sync`) genera el DDL según el dialecto — crea tablas y añade columnas faltantes, nunca elimina ni modifica. Las lecturas de relaciones cubren `belongs_to`, has_many (`children*`) y many-to-many (`m2m`). `#[bee(fk = …)]` emite DDL de clave foránea: postgres y sqlite (build bundled) hacen cumplir la referencia, y mysql puede activar claves foráneas a nivel de tabla con opt-in (ver abajo).

### Many-to-many (`m2m`)

Se declara a nivel de struct con `#[bee(m2m(Target))]`; la tabla puente la crea `create_table` / `sync` (sincroniza primero el modelo destino y después el declarante):

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // tabla puente user_tag, columnas user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // primero el modelo destino
migrate::sync::<User, _>(&pool).await?;      // el declarante crea la tabla puente

// create() inserta y relee la instancia (pk auto rellena), lista para relaciones
let user = User { id: 0, name: "alice".into() }.create(&pool).await?;
let tag = Tag { id: 0, name: "rust".into() }.create(&pool).await?;

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // attach duplicado -> error de PK compuesta
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // idempotente: una segunda llamada devuelve 0 filas
```

> Renombrados: `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`; las tablas puente solo se crean con `create_table` / `sync` — `add_missing_columns` nunca las toca.

### Columnas JSON

Los campos `serde_json::Value` se mapean a sqlite `TEXT` / postgres `JSONB` / mysql `JSON`:

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // obligatorio
    extra: Option<serde_json::Value>,  // nullable
}
```

> Semántica de NULL: SQL `NULL` se decodifica como `None`; un documento JSON `null` almacenado como `Some(Json::Null)` — no son lo mismo.

### Claves foráneas a nivel de tabla en MySQL (opt-in)

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // solo mysql
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> Activado, las tablas nuevas y las columnas añadidas llevan `FOREIGN KEY` a nivel de tabla (nombre de constraint `{table}_{column}_fk`); filas huérfanas preexistentes hacen fallar `ADD CONSTRAINT` (sin omisión silenciosa y sin retrofit de columnas existentes); en pg / sqlite la opción no hace nada (inline ya la aplica).

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls usa las raíces webpki incluidas; para una configuración rustls propia, connect_tls_with:
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig (reexportado, misma versión que la crate)
```

### Tipos fecha y Decimal

Los campos de fecha y Decimal están gateados por feature: activa `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) y `rust_decimal` (`Decimal`) en `bee_orm`; a través de `bee_rust` los features de reenvío son `orm-chrono` / `orm-rust_decimal`.

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date; DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal (igual en los tres backends)
}

// escritura: el campo se vuelve la variante Value correspondiente; lectura: la celda decodifica vía serde
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // "1.50" vuelve con igual valor
```

> Soportado: `NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`. Fuera de alcance — la macro no emite mapeo SQL, usa la vía de escape `#[bee(sql_type = "…")]`: `NaiveTime` / `DateTime<Local>` / `FixedOffset` / `DateTime` desnudo, y precisión de nanosegundos.
> Almacenamiento: sqlite siempre TEXT (`typeof(col)` es `text`; Decimal también — declarar DECIMAL daría afinidad NUMERIC y se tragaría "1.50" como REAL 1.5); mysql usa `datetime(6)` / `timestamp(6)` con microsegundos — TIMESTAMP se lee como UTC (RFC3339 con `Z`; el viaje estricto exige zona horaria de sesión UTC), DATETIME sigue naive sin sufijo, despachados por tipo de columna; pg `timestamptz` normaliza a UTC.
> Techo de precisión: en pg, `numeric` con más de 29 dígitos significativos se lee como `NULL` (el FromSql nativo lo rechaza); en mysql va por texto, la precisión sobrante falla recién al decodificar en tu serde; para dinero mejor `#[bee(sql_type = Raw("decimal(12,2)"))]`.
> Contrato en compilación: con el feature apagado, un modelo con campo de fecha falla con E0277 (`Value: From<NaiveDate>` no satisfecho) — nunca una columna silenciosamente incorrecta.

## Gestión de configuración (bee_config)

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

## Motores de almacenamiento

**KV/Caché:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// backend memcached (feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> `RedisStore` usa un `ConnectionManager`: una conexión caída se reconecta sola (bajo demanda — sin hilo en segundo plano — con backoff exponencial y jitter) — el comando que topa con la caída falla y el siguiente espera la conexión nueva. memcached: `incr` crea primero el contador (0 si no existe) y luego aplica el delta; los contadores no tienen signo, así que decrementar se queda en 0 (Redis baja a negativo); `expire(≤0)` equivale a borrar.

**Backends de caché (bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache` (feature `memcache`) tiene la misma interfaz; el TTL `Some(0)` pasa por `DEL` en el backend Redis (`SET … EX 0` se rechaza) y también se trata como borrado en memcached (allí 0 significa no expirar nunca); `incr` sobre un valor no numérico es un error de serialización.

**Motor de búsqueda (implementado, opt-in):**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // requiere feature `elasticsearch`
let engine = Elasticsearch::new("http://localhost:9200");   // constructor síncrono (sin `?`)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> Los demás drivers funcionan igual y también son opt-in: `opensearch`, `clickhouse` (features del mismo nombre).

**Base de datos de grafos (implementada, opt-in):**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // requiere feature `neo4j`
let db = Neo4j::new("http://localhost:7474");   // constructor síncrono (sin `?`)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> Los demás drivers funcionan igual y también son opt-in: `nebulagraph`, `arangodb` (features del mismo nombre).

**Base de datos de series temporales (implementada, opt-in):**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // requiere feature `influxdb`
let tsdb = InfluxDB::new("http://localhost:8086");   // constructor síncrono (sin `?`)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> Los demás drivers funcionan igual y también son opt-in: `iotdb`, `questdb` (features del mismo nombre).

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> El backend es cualquier caché que implemente `bee_cache::Cache`: `MemoryCache` / `RedisCache` / `MemcacheCache`.

## Registros

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## Plantillas

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## Herramientas CLI

```bash
# Crea un proyecto (genera un andamiaje ejecutable: Cargo.toml + src/main.rs)
bee-rust new my-app

# Genera código
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Ejecución en desarrollo (--watch observa src/ y reinicia automáticamente ante cambios)
bee-rust run
bee-rust run --watch

# Empaquetado y despliegue (cargo build --release + copia a dist/)
bee-rust pack

# Migración de base de datos (genera el punto de entrada en tu crate y lo ejecuta)
bee-rust migrate init    # crea src/bin/bee_migrate.rs (se niega a sobrescribir un archivo existente)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> Nota: el parámetro `pack --target` es una interfaz reservada; el proceso de empaquetado actual no distingue plataformas de destino.
