<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

Ein produktionsreifes Web-Framework für Rust. Die Designphilosophie stammt von Go's Beego und ist mit Rust-Mitteln neu ausgedrückt — Traits, Makros und Typsystem.

MVC-Controller · Namespace-Routing · Filterketten · ORM · einheitliche Trait-Abstraktionen über Speicher-Engines

## Installation

```bash
cargo add bee_rust
```

Oder in die `Cargo.toml` eintragen：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

Das Standard-Feature `full` aktiviert `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`.

## Schnellstart

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

Ein vollständig lauffähiges Projekt liegt in [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello) (mit E2E-Tests).

## Feature-Flags

`search`, `graph` und `tsdb` sind **nicht** Teil des Standard-`full` — bitte explizit aktivieren.

| Feature | Zieht herein | Beschreibung |
|---------|-----------|-------|
| `full` *(Standard)* | router, orm, kv, config, logs, cache, session, security, template | alles auf einmal |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Web-Kern: Routing + Controller + Filterkette |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | KV-Abstraktion |
| `cache` | bee_cache, bee_config | Cache-Abstraktion |
| `session` | bee_session, bee_cache | Sessions mit mehreren Backends |
| `config` | bee_config | INI / YAML / ENV + Hot Reload |
| `logs` | bee_logs | Logging nach Stufen + tracing |
| `template` | bee_template | tera-Templates |
| `security` | bee_router/security | 27 Filter zur Angriffserkennung |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

Minimaler Kern (ohne ORM und Datenbanktreiber)：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## Sub-Crates

Das Framework besteht aus unabhängig nutzbaren Crates; `bee_rust` ist der eine Einstiegspunkt, der sie bei Bedarf re-exportiert.

| Crate | Funktion | Beego-Entsprechung |
|-------|--------------|-------------------|
| `bee_rust` | Meta-Crate, einzelner Einstiegspunkt | — |
| `bee_router` | Routing + Controller + `Context` + Filter | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migrationen | `client/orm` |
| `bee_config` | Konfiguration + Hot Reload | `client/config` |
| `bee_logs` | Logging | `logs` |
| `bee_cache` | Cache-Abstraktion | `client/cache` |
| `bee_session` | Session-Verwaltung | `server/web/session` |
| `bee_template` | Template-Rendering | — (erweitert) |
| `bee_kv` | KV-/Cache-Abstraktion | `client/cache` (erweitert) |
| `bee_search` | Such-/Analyse-Engines | — (neu) |
| `bee_graph` | Graphdatenbanken | — (neu) |
| `bee_tsdb` | Zeitreihendatenbanken | — (neu) |
| `bee_cli` | Scaffolding / Codegen / Dev-Runner | `bee` Werkzeug |

Sie lassen sich auch einzeln einbinden — etwa nur ein Elasticsearch-Client：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## Unterstützte Datenbanken

| Art | Datenbanken | Crate | Feature |
|------|-----------|-------|---------|
| Relational | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / Cache | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| Suche / Analyse | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| Graph | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| Zeitreihen | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## Voraussetzungen

- Rust 1.80+ (MSRV)
- edition 2024

## Links

- **API-Dokumentation**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **Repository**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **Vollständige API-Referenz (13 Sprachen)**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## Lizenz

Apache-2.0
