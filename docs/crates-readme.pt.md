<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

Um framework web de nível de produção para Rust. Sua filosofia de design vem do Beego (Go), reexpressa com os traits, macros e o sistema de tipos do Rust.

Controladores MVC · roteamento por namespace · cadeias de filtros · ORM · abstrações de trait unificadas sobre engines de armazenamento

## Instalação

```bash
cargo add bee_rust
```

Ou adicione ao `Cargo.toml`：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

A feature `full` padrão ativa `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`.

## Início rápido

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

Há um projeto totalmente executável em [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello) (coberto por testes E2E).

## Feature flags

`search`, `graph` e `tsdb` **não fazem parte** do `full` padrão — ative-os explicitamente.

| Feature | O que inclui | Descrição |
|---------|-----------|-------|
| `full` *(padrão)* | router, orm, kv, config, logs, cache, session, security, template | tudo em um |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Núcleo Web: roteamento + controladores + cadeia de filtros |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | Abstração de KV |
| `cache` | bee_cache, bee_config | Abstração de cache |
| `session` | bee_session, bee_cache | sessões com múltiplos backends |
| `config` | bee_config | INI / YAML / ENV + recarga a quente |
| `logs` | bee_logs | logs por nível + tracing |
| `template` | bee_template | templates tera |
| `security` | bee_router/security | 27 filtros de detecção de ataques |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

Núcleo mínimo (sem ORM nem drivers de banco)：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## Sub-crates

O framework é um conjunto de crates usáveis de forma independente; `bee_rust` é o ponto de entrada único que os reexporta sob demanda.

| Crate | Função | Equivalente no Beego |
|-------|--------------|-------------------|
| `bee_rust` | meta-crate, ponto de entrada único | — |
| `bee_router` | roteamento + controladores + `Context` + filtros | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + migrações | `client/orm` |
| `bee_config` | configuração + recarga a quente | `client/config` |
| `bee_logs` | logs | `logs` |
| `bee_cache` | abstração de cache | `client/cache` |
| `bee_session` | gerenciamento de sessão | `server/web/session` |
| `bee_template` | renderização de templates | — (estendido) |
| `bee_kv` | abstração de KV/cache | `client/cache` (estendido) |
| `bee_search` | engines de busca / análise | — (novo) |
| `bee_graph` | bancos de dados de grafo | — (novo) |
| `bee_tsdb` | bancos de dados de séries temporais | — (novo) |
| `bee_cli` | scaffolding / geração de código / runner de dev | `bee` ferramenta |

Eles também podem ser usados separadamente — por exemplo, apenas um cliente Elasticsearch：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## Bancos de dados suportados

| Tipo | Bancos de dados | Crate | Feature |
|------|-----------|-------|---------|
| Relacionais | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / cache | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| Busca / análise | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| Grafo | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| Séries temporais | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## Requisitos

- Rust 1.80+ (MSRV)
- edition 2024

## Links

- **Documentação da API**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **Repositório**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **Referência completa da API (13 idiomas)**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## Licença

Apache-2.0
