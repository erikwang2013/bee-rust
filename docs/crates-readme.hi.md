<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

Rust के लिए प्रोडक्शन-ग्रेड वेब फ़्रेमवर्क। इसका डिज़ाइन दर्शन Go के Beego से लिया गया है और Rust के trait / macro / टाइप सिस्टम में दोबारा अभिव्यक्त किया गया है।

MVC कंट्रोलर · नेमस्पेस राउटिंग · फ़िल्टर चेन · ORM · स्टोरेज इंजनों पर एकीकृत trait अमूर्तन

## इंस्टॉल

```bash
cargo add bee_rust
```

या `Cargo.toml` में लिखें：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

डिफ़ॉल्ट `full` feature इन्हें सक्रिय करता है: `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`।

## त्वरित शुरुआत

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

पूरी तरह चलने योग्य प्रोजेक्ट [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello) में है (E2E टेस्ट सहित)।

## Feature फ़्लैग

`search`, `graph` और `tsdb` डिफ़ॉल्ट `full` में **शामिल नहीं** हैं — इन्हें अलग से सक्रिय करें।

| Feature | क्या जोड़ता है | विवरण |
|---------|-----------|-------|
| `full` *(डिफ़ॉल्ट)* | router, orm, kv, config, logs, cache, session, security, template | सब कुछ एक साथ |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Web कोर: राउटिंग + कंट्रोलर + फ़िल्टर चेन |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | KV अमूर्तन |
| `cache` | bee_cache, bee_config | कैश अमूर्तन |
| `session` | bee_session, bee_cache | मल्टी-बैकएंड सेशन |
| `config` | bee_config | INI / YAML / ENV + हॉट रीलोड |
| `logs` | bee_logs | स्तर-आधारित लॉगिंग + tracing |
| `template` | bee_template | tera टेम्पलेट |
| `security` | bee_router/security | 27 प्रकार के आक्रमण-पहचान फ़िल्टर |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

न्यूनतम कोर (ORM और डेटाबेस ड्राइवर के बिना)：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## सब-क्रेट

यह फ़्रेमवर्क स्वतंत्र रूप से प्रयोग होने योग्य क्रेट्स का समूह है; `bee_rust` वह एकल प्रवेश बिंदु है जो इन्हें आवश्यकता अनुसार re-export करता है।

| Crate | कार्य | Beego में समतुल्य |
|-------|--------------|-------------------|
| `bee_rust` | मेटा क्रेट, एकल प्रवेश बिंदु | — |
| `bee_router` | राउटिंग + कंट्रोलर + `Context` + फ़िल्टर | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + माइग्रेशन | `client/orm` |
| `bee_config` | कॉन्फ़िग + हॉट रीलोड | `client/config` |
| `bee_logs` | लॉगिंग | `logs` |
| `bee_cache` | कैश अमूर्तन | `client/cache` |
| `bee_session` | सेशन प्रबंधन | `server/web/session` |
| `bee_template` | टेम्पलेट रेंडरिंग | — (विस्तारित) |
| `bee_kv` | KV/कैश अमूर्तन | `client/cache` (विस्तारित) |
| `bee_search` | खोज / विश्लेषण इंजन | — (नया) |
| `bee_graph` | ग्राफ़ डेटाबेस | — (नया) |
| `bee_tsdb` | टाइम-सीरीज़ डेटाबेस | — (नया) |
| `bee_cli` | स्कैफ़ोल्डिंग / कोड जनरेशन / डेव रनर | `bee` टूल |

इन्हें अलग-अलग भी निर्भरता बनाया जा सकता है — जैसे केवल Elasticsearch क्लाइंट：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## समर्थित डेटाबेस

| प्रकार | डेटाबेस | Crate | Feature |
|------|-----------|-------|---------|
| रिलेशनल | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / कैश | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| खोज / विश्लेषण | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| ग्राफ़ | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| टाइम-सीरीज़ | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## आवश्यकताएँ

- Rust 1.80+ (MSRV)
- edition 2024

## लिंक

- **API दस्तावेज़**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **रिपॉज़िटरी**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **पूर्ण API संदर्भ (13 भाषाएँ)**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## लाइसेंस

Apache-2.0
