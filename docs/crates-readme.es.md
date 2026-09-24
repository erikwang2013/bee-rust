<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

Un framework web de nivel producción para Rust. Su filosofía de diseño proviene de Beego (Go), reexpresada con los traits, macros y el sistema de tipos de Rust.

Controladores MVC · enrutado por namespace · cadenas de filtros · ORM · abstracciones de trait unificadas sobre motores de almacenamiento

## Instalación

```bash
cargo add bee_rust
```

O añádelo al `Cargo.toml`：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

La feature `full` por defecto activa `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`.

## Inicio rápido

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

Hay un proyecto completamente ejecutable en [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello) (cubierto por tests E2E).

## Feature flags

`search`, `graph` y `tsdb` **no forman parte** del `full` por defecto: actívalos explícitamente.

| Feature | Lo que incluye | Descripción |
|---------|-----------|-------|
| `full` *(por defecto)* | router, orm, kv, config, logs, cache, session, security, template | todo en uno |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Núcleo Web: enrutado + controladores + cadena de filtros |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | Abstracción KV |
| `cache` | bee_cache, bee_config | Abstracción de caché |
| `session` | bee_session, bee_cache | sesiones multi-backend |
| `config` | bee_config | INI / YAML / ENV + recarga en caliente |
| `logs` | bee_logs | logging por niveles + tracing |
| `template` | bee_template | plantillas tera |
| `security` | bee_router/security | 27 filtros de detección de ataques |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

Núcleo mínimo (sin ORM ni drivers de base de datos)：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## Sub-crates

El framework es un conjunto de crates usables de forma independiente; `bee_rust` es el punto de entrada único que los reexporta según demanda.

| Crate | Función | Equivalente en Beego |
|-------|--------------|-------------------|
| `bee_rust` | meta-crate, punto de entrada único | — |
| `bee_router` | enrutado + controladores + `Context` + filtros | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + migraciones | `client/orm` |
| `bee_config` | configuración + recarga en caliente | `client/config` |
| `bee_logs` | logging | `logs` |
| `bee_cache` | abstracción de caché | `client/cache` |
| `bee_session` | gestión de sesiones | `server/web/session` |
| `bee_template` | renderizado de plantillas | — (ampliado) |
| `bee_kv` | abstracción KV/caché | `client/cache` (ampliado) |
| `bee_search` | motores de búsqueda / analítica | — (nuevo) |
| `bee_graph` | bases de datos de grafos | — (nuevo) |
| `bee_tsdb` | bases de datos de series temporales | — (nuevo) |
| `bee_cli` | scaffolding / generación de código / runner de desarrollo | `bee` herramienta |

También pueden usarse por separado; por ejemplo, solo un cliente de Elasticsearch：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## Bases de datos compatibles

| Tipo | Bases de datos | Crate | Feature |
|------|-----------|-------|---------|
| Relacionales | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / caché | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| Búsqueda / analítica | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| Grafos | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| Series temporales | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## Requisitos

- Rust 1.80+ (MSRV)
- edition 2024

## Enlaces

- **Documentación de la API**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **Repositorio**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **Referencia completa de la API (13 idiomas)**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## Licencia

Apache-2.0
