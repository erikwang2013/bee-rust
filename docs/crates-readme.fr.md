<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

Un framework web de niveau production pour Rust. Sa philosophie de conception vient de Beego (Go), réexprimée avec les traits, les macros et le système de types de Rust.

Contrôleurs MVC · routage par namespace · chaînes de filtres · ORM · abstractions de trait unifiées sur les moteurs de stockage

## Installation

```bash
cargo add bee_rust
```

Ou dans le `Cargo.toml`：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

La feature `full` par défaut active `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`.

## Démarrage rapide

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

Un projet entièrement exécutable se trouve dans [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello) (couvert par des tests E2E).

## Feature flags

`search`, `graph` et `tsdb` **ne font pas partie** du `full` par défaut — activez-les explicitement.

| Feature | Ce que ça tire | Description |
|---------|-----------|-------|
| `full` *(défaut)* | router, orm, kv, config, logs, cache, session, security, template | tout-en-un |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Cœur Web : routage + contrôleurs + chaîne de filtres |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | Abstraction KV |
| `cache` | bee_cache, bee_config | Abstraction de cache |
| `session` | bee_session, bee_cache | sessions multi-backends |
| `config` | bee_config | INI / YAML / ENV + rechargement à chaud |
| `logs` | bee_logs | journalisation par niveau + tracing |
| `template` | bee_template | templates tera |
| `security` | bee_router/security | 27 filtres de détection d'attaques |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

Cœur minimal (sans ORM ni pilotes de base de données)：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## Sous-crates

Le framework est un ensemble de crates utilisables indépendamment ; `bee_rust` est le point d'entrée unique qui les réexporte à la demande.

| Crate | Rôle | Équivalent Beego |
|-------|--------------|-------------------|
| `bee_rust` | méta-crate, point d'entrée unique | — |
| `bee_router` | routage + contrôleurs + `Context` + filtres | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + migrations | `client/orm` |
| `bee_config` | configuration + rechargement à chaud | `client/config` |
| `bee_logs` | journalisation | `logs` |
| `bee_cache` | abstraction de cache | `client/cache` |
| `bee_session` | gestion de session | `server/web/session` |
| `bee_template` | rendu de templates | — (étendu) |
| `bee_kv` | abstraction KV/cache | `client/cache` (étendu) |
| `bee_search` | moteurs de recherche / analyse | — (nouveau) |
| `bee_graph` | bases de données graphe | — (nouveau) |
| `bee_tsdb` | bases de données temporelles | — (nouveau) |
| `bee_cli` | scaffolding / génération de code / runner de dev | `bee` outil |

Ils peuvent aussi être dépendus séparément — par exemple, juste un client Elasticsearch：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## Bases de données prises en charge

| Type | Bases de données | Crate | Feature |
|------|-----------|-------|---------|
| Relationnelles | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / cache | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| Recherche / analyse | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| Graphe | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| Temporelles | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## Prérequis

- Rust 1.80+ (MSRV)
- edition 2024

## Liens

- **Documentation API**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **Dépôt**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **Référence API complète (13 langues)**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## Licence

Apache-2.0
