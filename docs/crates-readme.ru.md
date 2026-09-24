<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

Продакшн-фреймворк для Rust. Философия дизайна заимствована у Beego (Go) и переосмыслена средствами Rust — трейтами, макросами и системой типов.

MVC-контроллеры · маршрутизация с пространствами имён · цепочки фильтров · ORM · единые трейт-абстракции над хранилищами

## Установка

```bash
cargo add bee_rust
```

Либо добавьте в `Cargo.toml`：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

Feature `full` по умолчанию включает `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`.

## Быстрый старт

```rust
async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> bee_rust::Result<()> {
    let _log_handle = bee_rust::init()?;

    let router = bee_rust::bee_router::Router::new()
        .ns("/api/v1", |ns| ns.get("/health", health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, router.build()).await?;
    Ok(())
}
```

```bash
cargo run
curl http://localhost:8080/api/v1/health    # OK
```

Полностью рабочий проект — в [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello) (с E2E-тестами).

## Feature-флаги

`search`, `graph` и `tsdb` **не входят** в `full` по умолчанию — включайте их явно.

| Feature | Что подключает | Описание |
|---------|-----------|-------|
| `full` *(по умолчанию)* | router, orm, kv, config, logs, cache, session, security, template | всё сразу |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Ядро Web: маршрутизация + контроллеры + цепочка фильтров |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | Абстракция KV |
| `cache` | bee_cache, bee_config | Абстракция кэша |
| `session` | bee_session, bee_cache | Сессии с несколькими бэкендами |
| `config` | bee_config | INI / YAML / ENV + горячая перезагрузка |
| `logs` | bee_logs | Уровневое логирование + tracing |
| `template` | bee_template | Шаблоны tera |
| `security` | bee_router/security | 27 фильтров обнаружения атак |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

Минимальное ядро (без ORM и драйверов БД)：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## Подкрейты

Фреймворк состоит из независимо используемых крейтов; `bee_rust` — единая точка входа, реэкспортирующая их по требованию.

| Crate | Назначение | Аналог в Beego |
|-------|--------------|-------------------|
| `bee_rust` | мета-крейт, единая точка входа | — |
| `bee_router` | маршрутизация + контроллеры + `Context` + фильтры | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + миграции | `client/orm` |
| `bee_config` | конфигурация + горячая перезагрузка | `client/config` |
| `bee_logs` | логирование | `logs` |
| `bee_cache` | абстракция кэша | `client/cache` |
| `bee_session` | управление сессиями | `server/web/session` |
| `bee_template` | рендеринг шаблонов | — (расширено) |
| `bee_kv` | абстракция KV/кэша | `client/cache` (расширено) |
| `bee_search` | поисковые / аналитические движки | — (новое) |
| `bee_graph` | графовые базы данных | — (новое) |
| `bee_tsdb` | базы данных временных рядов | — (новое) |
| `bee_cli` | скаффолдинг / генерация кода / dev-раннер | `bee` утилита |

Их можно подключать и по отдельности — например, только клиент Elasticsearch：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## Поддерживаемые базы данных

| Тип | Базы данных | Crate | Feature |
|------|-----------|-------|---------|
| Реляционные | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / кэш | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| Поиск / аналитика | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| Графовые | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| Временные ряды | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## Требования

- Rust 1.80+ (MSRV)
- edition 2024

## Ссылки

- **Документация API**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **Репозиторий**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **Полный справочник API (13 языков)**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## Лицензия

Apache-2.0
