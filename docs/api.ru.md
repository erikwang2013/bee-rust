<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Справочник API Beerust

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Назад к [README](README.ru.md).

Этот документ — справочник API фреймворка Beerust: использование модулей, примеры кода и описание интерфейсов.

## Веб-ядро (bee_router)

### Определение контроллера

```rust
use bee_rust::prelude::*;

// определяем контроллер
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Регистрация маршрутов

```rust
// регистрируем маршруты
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context предоставляет:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — вывод ответа
- `ctx.redirect()` — перенаправление
- `ctx.abort()` — прерывание запроса
- `ctx.session` — доступ к сессии
- `ctx.params` — параметры пути

## Обнаружение атак (feature `security`)

Фильтр обнаружения атак на основе [security-rust](https://crates.io/crates/security-rust), покрывает 27 типов атак: XSS, SQL-инъекции, инъекции команд, SSRF и другие:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // все 27 детекторов включены
```

Включите в `Cargo.toml`:
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
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql имеют одинаковую форму

    // Схема — migrate::sync генерирует неразрушающий DDL из метаданных модели
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — первичный ключ `auto` назначает база данных
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — цепочные фильтры + выполнение: all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // Связи — children для has_many, belongs_to для родителя; при включённом FK сначала удалите потомка, затем родителя
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete мягко удаляет (переключает флаг); with_deleted() / hard_delete() обходят это
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

    // Транзакции — держите соединение из `get()`; `CheckedConn` в sqlite синхронный
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> Миграции доступны: `bee_orm::migrate` (`create_table` / `add_missing_columns` / `sync`) генерирует DDL под каждый диалект — создаёт таблицы и добавляет недостающие колонки, никогда не удаляя и не изменяя их. Чтение связей покрывает `belongs_to`, has_many (`children*`) и many-to-many (`m2m`). `#[bee(fk = …)]` генерирует DDL внешнего ключа: postgres и sqlite (bundled-сборка) обеспечивают соблюдение ссылки, а mysql может включить внешние ключи на уровне таблицы через opt-in (ниже).

### Many-to-many (`m2m`)

Объявляется на уровне структуры: `#[bee(m2m(Target))]`; таблицу связей создаёт `create_table` / `sync` (сначала синхронизируйте целевую модель, затем объявляющую):

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // таблица связей user_tag, колонки user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // сначала целевая модель
migrate::sync::<User, _>(&pool).await?;      // объявляющая модель создаёт таблицу связей

// `auto`-ключи назначает база: перед работой со связями перечитайте экземпляр
User { id: 0, name: "alice".into() }.insert(&pool).await?;
let user = User::query().filter_eq("name", "alice")?.one(&pool).await?.expect("inserted");
Tag { id: 0, name: "rust".into() }.insert(&pool).await?;
let tag = Tag::query().filter_eq("name", "rust")?.one(&pool).await?.expect("inserted");

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // повторный attach -> ошибка составного PK
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // идемпотентно: второй вызов вернёт 0 строк
```

> Переименования: `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`; таблицы связей создаются только через `create_table` / `sync` — `add_missing_columns` их не трогает.

### JSON-колонки

Поля `serde_json::Value` отображаются в sqlite `TEXT` / postgres `JSONB` / mysql `JSON`:

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // обязательное
    extra: Option<serde_json::Value>,  // nullable
}
```

> Семантика NULL: SQL `NULL` декодируется в `None`; сохранённый JSON-документ `null` — в `Some(Json::Null)`; это разные вещи.

### Внешние ключи на уровне таблицы в MySQL (opt-in)

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // только mysql
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> При включении новые таблицы и добавленные колонки получают `FOREIGN KEY` уровня таблицы (имя ограничения `{table}_{column}_fk`); уже существующие «осиротевшие» строки приводят к ошибке `ADD CONSTRAINT` (без тихого пропуска и без retrofit существующих колонок); на pg / sqlite опция ничего не делает (inline уже обеспечивает).

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls использует встроенные корни webpki; для своей конфигурации rustls — connect_tls_with:
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig (реэкспорт, та же версия, что у crate)
```

### Типы даты и Decimal

Поля даты и Decimal включаются feature: включите `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) и `rust_decimal` (`Decimal`) у `bee_orm`; через `bee_rust` пробрасывающие feature — `orm-chrono` / `orm-rust_decimal`.

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date; DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal (одинаково на всех трёх бэкендах)
}

// запись: поле превращается в нужный вариант Value; чтение: ячейка декодируется через serde
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // "1.50" возвращается численно равным
```

> Поддержка: `NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`. Вне области — макрос не выдаёт SQL-отображение, используйте `#[bee(sql_type = "…")]`: `NaiveTime` / `DateTime<Local>` / `FixedOffset` / голый `DateTime` и наносекундная точность.
> Хранение: sqlite всегда TEXT (`typeof(col)` = `text`; Decimal тоже — объявление DECIMAL дало бы NUMERIC-аффинность и превратило "1.50" в REAL 1.5); mysql — `datetime(6)` / `timestamp(6)` с микросекундами: TIMESTAMP читается как UTC (RFC3339 с `Z`; строгий round trip требует сессионной зоны UTC), DATETIME остаётся naive без суффикса, разбор по типу колонки; pg `timestamptz` нормализуется в UTC.
> Потолок точности: в pg `numeric` свыше 29 значащих цифр читается как `NULL` (нативный FromSql отклоняет); в mysql всё идёт текстом — избыточная точность падает только на декодировании в вашем serde; для денег лучше `#[bee(sql_type = Raw("decimal(12,2)"))]`.
> Контракт компиляции: с выключенным feature модель с полем даты падает с E0277 (`Value: From<NaiveDate>` не выполнен) — никакой тихо неверной колонки.

## Управление конфигурацией (bee_config)

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

## Движки хранения

**KV/Кэш:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// бэкенд memcached (feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> Внутри `RedisStore` — `ConnectionManager`: разорванное соединение восстанавливается само (по требованию — без фонового потока — с экспоненциальным backoff и джиттером) — команда, попавшая на разрыв, вернёт ошибку, следующая дождётся нового соединения. memcached: `incr` сначала создаёт счётчик (0, если его нет), затем применяет дельту; счётчики беззнаковые, поэтому декремент упирается в 0 (в Redis уходит в минус); `expire(≤0)` равносилен удалению.

**Бэкенды кэша (bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> У `MemcacheCache` (feature `memcache`) тот же интерфейс; TTL `Some(0)` в Redis-бэкенде идёт через `DEL` (`SET … EX 0` сервер отвергает), а в memcached трактуется как удаление (там 0 означает «никогда не истекает»); `incr` по нечисловому значению — ошибка сериализации.

**Поисковый движок (реализован, opt-in):**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // требует feature `elasticsearch`
let engine = Elasticsearch::new("http://localhost:9200");   // синхронный конструктор (без `?`)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> Остальные драйверы работают так же и тоже opt-in: `opensearch`, `clickhouse` (одноимённые feature).

**Графовая БД (реализована, opt-in):**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // требует feature `neo4j`
let db = Neo4j::new("http://localhost:7474");   // синхронный конструктор (без `?`)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> Остальные драйверы работают так же и тоже opt-in: `nebulagraph`, `arangodb` (одноимённые feature).

**БД временных рядов (реализована, opt-in):**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // требует feature `influxdb`
let tsdb = InfluxDB::new("http://localhost:8086");   // синхронный конструктор (без `?`)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> Остальные драйверы работают так же и тоже opt-in: `iotdb`, `questdb` (одноимённые feature).

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> Бэкенд — любой кэш, реализующий `bee_cache::Cache`: `MemoryCache` / `RedisCache` / `MemcacheCache`.

## Логирование

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## Шаблоны

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## Инструменты CLI

```bash
# создание проекта (генерирует рабочий скаффолд: Cargo.toml + src/main.rs)
bee-rust new my-app

# генерация кода
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# запуск в разработке (--watch следит за изменениями в src/ и перезапускает автоматически)
bee-rust run
bee-rust run --watch

# упаковка для развёртывания (cargo build --release + копирование в dist/)
bee-rust pack

# Миграции базы данных (генерирует точку входа в вашем crate и запускает её)
bee-rust migrate init    # создаёт src/bin/bee_migrate.rs (существующий файл не перезаписывается)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> Примечание: параметр `pack --target` — зарезервированный интерфейс; сейчас процесс упаковки не различает целевые платформы.
