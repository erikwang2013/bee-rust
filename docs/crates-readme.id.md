<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

Framework web kelas produksi untuk Rust. Filosofi desainnya diambil dari Beego (Go), diekspresikan ulang lewat trait, macro, dan sistem tipe Rust.

Controller MVC · routing namespace · rantai filter · ORM · abstraksi trait terpadu atas engine penyimpanan

## Instalasi

```bash
cargo add bee_rust
```

Atau tuliskan di `Cargo.toml`：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

Feature `full` bawaan mengaktifkan `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`.

## Mulai cepat

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

Proyek yang benar-benar bisa dijalankan ada di [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello) (dengan tes E2E).

## Feature flag

`search`, `graph`, dan `tsdb` **tidak termasuk** dalam `full` bawaan — aktifkan secara eksplisit.

| Feature | Yang disertakan | Keterangan |
|---------|-----------|-------|
| `full` *(bawaan)* | router, orm, kv, config, logs, cache, session, security, template | semuanya sekaligus |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Inti Web: routing + controller + rantai filter |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | Abstraksi KV |
| `cache` | bee_cache, bee_config | Abstraksi cache |
| `session` | bee_session, bee_cache | sesi multi-backend |
| `config` | bee_config | INI / YAML / ENV + hot reload |
| `logs` | bee_logs | logging bertingkat + tracing |
| `template` | bee_template | template tera |
| `security` | bee_router/security | 27 filter pendeteksi serangan |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

Inti minimal (tanpa ORM dan driver basis data)：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## Sub-crate

Framework ini terdiri dari crate yang bisa dipakai secara mandiri; `bee_rust` adalah satu-satunya pintu masuk yang mengekspor ulang sesuai kebutuhan.

| Crate | Fungsi | Padanan di Beego |
|-------|--------------|-------------------|
| `bee_rust` | meta crate, pintu masuk tunggal | — |
| `bee_router` | routing + controller + `Context` + filter | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + migrasi | `client/orm` |
| `bee_config` | konfigurasi + hot reload | `client/config` |
| `bee_logs` | logging | `logs` |
| `bee_cache` | abstraksi cache | `client/cache` |
| `bee_session` | manajemen sesi | `server/web/session` |
| `bee_template` | render template | — (diperluas) |
| `bee_kv` | abstraksi KV/cache | `client/cache` (diperluas) |
| `bee_search` | engine pencarian / analitik | — (baru) |
| `bee_graph` | basis data graf | — (baru) |
| `bee_tsdb` | basis data time-series | — (baru) |
| `bee_cli` | scaffolding / codegen / dev runner | `bee` alat |

Mereka juga bisa dijadikan dependensi terpisah — misalnya hanya klien Elasticsearch：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## Basis data yang didukung

| Jenis | Basis data | Crate | Feature |
|------|-----------|-------|---------|
| Relasional | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / cache | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| Pencarian / analitik | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| Graf | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| Time-series | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## Persyaratan

- Rust 1.80+ (MSRV)
- edition 2024

## Tautan

- **Dokumentasi API**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **Repositori**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **Referensi API lengkap (13 bahasa)**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## Lisensi

Apache-2.0
