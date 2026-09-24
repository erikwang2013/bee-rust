<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

Rust-এর জন্য প্রোডাকশন-মানের ওয়েব ফ্রেমওয়ার্ক। এর ডিজাইন দর্শন Go-এর Beego থেকে নেওয়া, Rust-এর trait / macro / টাইপ সিস্টেমে নতুন করে প্রকাশ করা।

MVC কন্ট্রোলার · নেমস্পেস রাউটিং · ফিল্টার চেইন · ORM · স্টোরেজ ইঞ্জিনের উপর একীভূত trait বিমূর্তন

## ইনস্টল

```bash
cargo add bee_rust
```

অথবা `Cargo.toml`-এ লিখুন：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

ডিফল্ট `full` feature এগুলো চালু করে: `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`।

## দ্রুত শুরু

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

সম্পূর্ণ চালানোর যোগ্য প্রকল্প আছে [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello)-তে (E2E টেস্ট সহ)।

## feature ফ্ল্যাগ

`search`, `graph` ও `tsdb` ডিফল্ট `full`-এ **নেই** — এগুলো আলাদা করে চালু করুন।

| Feature | যা যুক্ত করে | বিবরণ |
|---------|-----------|-------|
| `full` *(ডিফল্ট)* | router, orm, kv, config, logs, cache, session, security, template | সব একসাথে |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Web কোর: রাউটিং + কন্ট্রোলার + ফিল্টার চেইন |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | KV বিমূর্তন |
| `cache` | bee_cache, bee_config | ক্যাশে বিমূর্তন |
| `session` | bee_session, bee_cache | মাল্টি-ব্যাকএন্ড সেশন |
| `config` | bee_config | INI / YAML / ENV + হট রিলোড |
| `logs` | bee_logs | স্তরভিত্তিক লগিং + tracing |
| `template` | bee_template | tera টেমপ্লেট |
| `security` | bee_router/security | ২৭ ধরনের আক্রমণ শনাক্তকরণ ফিল্টার |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

সর্বনিম্ন কোর (ORM ও ডেটাবেস ড্রাইভার ছাড়া)：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## সাব-ক্রেট

এই ফ্রেমওয়ার্ক স্বতন্ত্রভাবে ব্যবহারযোগ্য ক্রেটের সমষ্টি; `bee_rust` হলো একক প্রবেশবিন্দু যা প্রয়োজন অনুযায়ী এগুলো re-export করে।

| Crate | কাজ | Beego-তে সমতুল্য |
|-------|--------------|-------------------|
| `bee_rust` | মেটা ক্রেট, একক প্রবেশবিন্দু | — |
| `bee_router` | রাউটিং + কন্ট্রোলার + `Context` + ফিল্টার | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + মাইগ্রেশন | `client/orm` |
| `bee_config` | কনফিগ + হট রিলোড | `client/config` |
| `bee_logs` | লগিং | `logs` |
| `bee_cache` | ক্যাশে বিমূর্তন | `client/cache` |
| `bee_session` | সেশন ব্যবস্থাপনা | `server/web/session` |
| `bee_template` | টেমপ্লেট রেন্ডারিং | — (বর্ধিত) |
| `bee_kv` | KV/ক্যাশে বিমূর্তন | `client/cache` (বর্ধিত) |
| `bee_search` | সার্চ / অ্যানালিটিক্স ইঞ্জিন | — (নতুন) |
| `bee_graph` | গ্রাফ ডেটাবেস | — (নতুন) |
| `bee_tsdb` | টাইম-সিরিজ ডেটাবেস | — (নতুন) |
| `bee_cli` | স্ক্যাফোল্ডিং / কোড জেনারেশন / ডেভ রানার | `bee` টুল |

এগুলো আলাদাভাবেও নির্ভরতা করা যায় — যেমন শুধু একটি Elasticsearch ক্লায়েন্ট：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## সমর্থিত ডেটাবেস

| ধরন | ডেটাবেস | Crate | Feature |
|------|-----------|-------|---------|
| রিলেশনাল | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / ক্যাশে | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| সার্চ / অ্যানালিটিক্স | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| গ্রাফ | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| টাইম-সিরিজ | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## প্রয়োজনীয়তা

- Rust 1.80+ (MSRV)
- edition 2024

## লিঙ্ক

- **API ডকুমেন্টেশন**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **রিপোজিটরি**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **সম্পূর্ণ API রেফারেন্স (১৩ ভাষা)**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## লাইসেন্স

Apache-2.0
