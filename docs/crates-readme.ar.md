<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

إطار عمل ويب بمستوى الإنتاج للغة Rust. فلسفة تصميمه مأخوذة من Beego بلغة Go، وأُعيدت صياغتها بسمات Rust وماكرواتها ونظام أنواعها.

متحكّمات MVC · توجيه بالنطاقات · سلاسل مرشّحات · ORM · تجريدات trait موحّدة فوق محرّكات التخزين

## التثبيت

```bash
cargo add bee_rust
```

أو أضِفه إلى `Cargo.toml`：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

تُفعّل ميزة `full` الافتراضية كلا من `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`.

## البدء السريع

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

يوجد مشروع قابل للتشغيل بالكامل في [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello) (مغطّى باختبارات E2E).

## أعلام feature

‏`search` و`graph` و`tsdb` **ليست ضمن** `full` الافتراضية — فعّلها صراحةً.

| Feature | ما الذي يضمّه | الوصف |
|---------|-----------|-------|
| `full` *(افتراضي)* | router, orm, kv, config, logs, cache, session, security, template | كل شيء دفعة واحدة |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | نواة الويب: التوجيه + المتحكّمات + سلسلة المرشّحات |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | تجريد KV |
| `cache` | bee_cache, bee_config | تجريد التخزين المؤقت |
| `session` | bee_session, bee_cache | جلسات متعددة الخلفيات |
| `config` | bee_config | INI / YAML / ENV + إعادة تحميل فورية |
| `logs` | bee_logs | تسجيل متدرّج + tracing |
| `template` | bee_template | قوالب tera |
| `security` | bee_router/security | 27 مرشّحًا لكشف الهجمات |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

النواة الدنيا (بدون ORM أو مشغّلات قواعد البيانات)：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## الحزم الفرعية

يتكوّن الإطار من حزم قابلة للاستخدام بشكل مستقل؛ و`bee_rust` هو نقطة الدخول الوحيدة التي تعيد تصديرها حسب الحاجة.

| Crate | الوظيفة | المقابل في Beego |
|-------|--------------|-------------------|
| `bee_rust` | حزمة وصفية، نقطة الدخول الوحيدة | — |
| `bee_router` | التوجيه + المتحكّمات + `Context` + المرشّحات | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + الترحيلات | `client/orm` |
| `bee_config` | الإعدادات + إعادة التحميل الفورية | `client/config` |
| `bee_logs` | التسجيل | `logs` |
| `bee_cache` | تجريد التخزين المؤقت | `client/cache` |
| `bee_session` | إدارة الجلسات | `server/web/session` |
| `bee_template` | عرض القوالب | — (موسّع) |
| `bee_kv` | تجريد KV/التخزين المؤقت | `client/cache` (موسّع) |
| `bee_search` | محرّكات البحث / التحليل | — (جديد) |
| `bee_graph` | قواعد بيانات الرسوم البيانية | — (جديد) |
| `bee_tsdb` | قواعد بيانات السلاسل الزمنية | — (جديد) |
| `bee_cli` | الهيكلة / توليد الشيفرة / مشغّل التطوير | `bee` أداة |

يمكن الاعتماد عليها منفردة أيضًا — مثلًا عميل Elasticsearch فقط：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## قواعد البيانات المدعومة

| النوع | قواعد البيانات | Crate | Feature |
|------|-----------|-------|---------|
| علائقية | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / تخزين مؤقت | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| بحث / تحليل | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| رسوم بيانية | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| سلاسل زمنية | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## المتطلبات

- Rust 1.80+ (MSRV)
- edition 2024

## روابط

- **توثيق API**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **المستودع**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **مرجع API الكامل (13 لغة)**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## الترخيص

Apache-2.0
