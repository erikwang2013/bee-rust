# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

> সংস্করণায়ন: প্রকল্পটি 1.x থাকা অবস্থায়, মাইনর রিলিজে API-ভাঙা পরিবর্তন থাকতে পারে; আপনি যদি `OrmError` সম্পূর্ণভাবে match করেন তবে নির্দিষ্ট সংস্করণ পিন করুন। 1.3.0 এবং 1.6.0 দেখুন।

## [1.6.0] — 2026-10-25

### যা যোগ হয়েছে
- `bee_orm`: শার্ডিংয়ের সহজ ব্যবহার — `#[bee(shard_key)]` রাউটিং ফিল্ড চিহ্নিত করে (সর্বোচ্চ একটি; `ignore` করা ফিল্ড এবং ডেটাবেস-নির্ধারিত `auto` / `auto_now_add` / `auto_now` ফিল্ডে প্রত্যাখ্যাত, কারণ রাউটিং insert-এর আগে ঘটে), `Model::shard_key()` এটি উন্মুক্ত করে (`None` / `NULL` রাউটযোগ্য নয়), এবং `ShardedPools::for_model` কী পুনরাবৃত্তি ছাড়াই একটি ইনস্ট্যান্স রাউট করে। `ShardedPools::all_shards()` কনফিগার করা পুলগুলি পুনরাবৃত্তি করে, এবং `sync_all_parallel` প্রতিটি পুল × টেবিল শার্ড সমবর্তীভাবে মাইগ্রেট করে (ইতিমধ্যে প্রয়োগ করা DDL রোলব্যাক হয় না)।
- `bee_orm`: `OrmError::ShardError` শার্ডিংয়ের ভুল ব্যবহারকে কোয়েরি ব্যর্থতা থেকে আলাদা করে; `OrmError` এখন `#[non_exhaustive]` — বাহ্যিক match-এ wildcard arm প্রয়োজন।
- `bee_orm`: ক্রস-ব্যাকএন্ড `PoolStatus` সহ sqlite, postgres ও mysql পুলে `pool_status()` (mysql-এ `size` / `available` হলো `None`, কারণ এটি কোনো পরিসংখ্যান দেয় না), এবং ধীর কোয়েরি কলব্যাকের জন্য `TimedDb`। Criterion বেঞ্চমার্ক রাউটার হ্যাশিং ও QuerySet-এর SQL রেন্ডারিং কভার করে।
- `bee_kv` / `bee_cache`: `RedisLock::acquire(key, ttl, wait)` সীমিত অপেক্ষা শেষ হওয়া পর্যন্ত সর্বাধিক প্রতি 50ms-এ লক পুনরায় চেষ্টা করে; `RedisPoolStatus` / `pool_status()` crate-মালিকানাধীন পুল স্ন্যাপশট দেয়; `redis-tls` feature `rediss://` সংযোগ সক্রিয় করে এবং ACL ক্রেডেনশিয়াল URL-এ যায়।
- `bee_kv` / `bee_cache`: `redis-sentinel` / `redis-cluster` feature-এর পিছনে Sentinel স্টোর/ক্যাশ (`RedisSentinelStore`, `RedisSentinelCache`) এবং Cluster স্টোর/ক্যাশ (`RedisClusterStore`, `RedisClusterCache`); `CROSSSLOT` এড়াতে ক্লাস্টার `mget` / `mset` একক-কী কমান্ড হিসেবে চলে।
- `examples/sharding` ও `examples/redis-lock`; CI উভয় উদাহরণ ও বেঞ্চমার্ক কম্পাইল করে, MySQL 9.7.4 পিন করে এবং MariaDB job রাখে।
- `bee_router`: GET/HEAD-এর জন্য opt-in লেটেন্সি বাজেট, সাথে পুরোনো ক্যাশ ফলব্যাক — `Router::latency(LatencyConfig::new(cache))` ডিফল্টে মোট 6 ms ব্যবহার করে (4 ms handler + 2 ms রিজার্ভ), 64 KiB পর্যন্ত পাবলিক GET 200 রেসপন্স ক্যাশ করে, এবং টাইমআউটে সেগুলি পুনরায় পরিবেশন করে (`X-Bee-Cache: stale`, `Age`) বা 504 ফেরত দেয়। টাইমআউট হওয়া GET ক্যাশ গরম করতে একটি সীমিত ব্যাকগ্রাউন্ড টাস্কে চলতে থাকে, এবং `LatencyStats` কাউন্টার ও আনুমানিক P50/P99 প্রকাশ করে। রাইট মেথডগুলি 30 s/408 আচরণ ধরে রাখে।

### যা পরিবর্তন করা হয়েছে
- `OrmError` এখন `#[non_exhaustive]` এবং শার্ডিং ব্যর্থতা এখন `QueryError`-এর বদলে `ShardError` রিপোর্ট করে; ডাউনস্ট্রিম match-এ wildcard arm প্রয়োজন।

### সীমাবদ্ধতা
- Sentinel: পুল করা মাস্টার সংযোগ `PING` দিয়ে রিসাইকেল হয়, ভূমিকা-যাচাই হয় না, তাই failover-এর পরে পুনরায় পদাবনত মাস্টার যা পৌঁছানোযোগ্য থাকে সে পুরোনো সংযোগ পরিবেশন করতে পারে। `deadpool-redis` 0.18-এর `Config::from_urls`-এর মাধ্যমে মাস্টার সংযোগে ACL ক্রেডেনশিয়াল বা TLS-ও যায় না (Sentinel URL-এর ক্রেডেনশিয়াল/TLS শুধু Sentinel-এর জন্য প্রযোজ্য); ACL-সুরক্ষিত বা শুধু-TLS মাস্টারকে সরাসরি সংযোগ করতে হবে।
- Cluster: কমান্ডগুলি `RedisOptions::response_timeout` দ্বারা সীমাবদ্ধ নয় — `deadpool-redis` 0.18 কোনো setter দেয় না; checkout, create ও recycle তাদের 30s / 5s / 5s সীমা ধরে রাখে।
- লেটেন্সি বাজেট: শুধু GET/HEAD। সময়সীমা রেসপন্সকে সীমাবদ্ধ করে, সার্ভার-সাইড কাজকে নয় — SQLite-এর ব্লকিং কোয়েরি বাতিল করা যায় না এবং Postgres/MySQL/Redis কমান্ড চলতে থাকে। ক্যাশ করা রেসপন্স শুধু পাবলিক এবং TTL পর্যন্ত পুরোনো হতে পারে; রাইট ক্যাশকে কখনও অকার্যকর করে না; শুধু সঠিক আকারের ≤64 KiB বডি ক্যাশ হয়; `MemoryCache` অসীম। long-poll/streaming GET রুটের জন্য `latency` ছাড়া Router ব্যবহার করা উচিত।

## [1.5.0] — 2026-10-25

### যা যোগ হয়েছে
- `bee_orm`: MySQL 5.7 ও MariaDB 10.11+ সমর্থন — CI ম্যাট্রিক্স এখন 5.7 / 8.0 / 8.4 / 9.x প্লাস একটি MariaDB job কভার করে; ইন্টিগ্রেশন টেস্টগুলি সার্ভার ভেন্ডর সনাক্ত করে (MariaDB JSON-কে LONGTEXT হিসেবে রিপোর্ট করে এবং কলাম-স্তরের `REFERENCES` প্রয়োগ করে; MySQL 5.7/8.x এটি পার্স করে উপেক্ষা করে)।
- `bee_orm`: MySQL-এ `Model::create` এখন মডেলের `columns()` মেটাডেটা ব্যবহার করে প্রতিটি insert-এর `information_schema` প্রোব এড়িয়ে যায়; মেটাডেটা-হীন হাতে লেখা মডেল পুরোনো প্রোব ধরে রাখে। এই হিন্টটি কর্তৃত্বপূর্ণ — schema ও মডেলের ড্রিফট সমর্থিত নয়।
- `bee_orm::shard`: `ShardedPools::{pools, pool, len, is_empty}` কনফিগার করা শার্ডগুলি অপারেশনের জন্য উন্মুক্ত করে; `ShardHandle` অটো-ইনক্রিমেন্ট হিন্ট ফরওয়ার্ড করে।
- `bee_kv` / `bee_cache`: `status()` পুল মেট্রিক্স; `with_options` সহ পাবলিক `RedisOptions` (`max_size`, `response_timeout`); `RedisLock::try_lock_with_watchdog` release/drop পর্যন্ত TTL নবীকরণ করে (watchdog TTL কমপক্ষে 30ms হতে হবে)।
- অভ্যন্তরীণ: একটি ডকুমেন্টেশন-সামঞ্জস্য CI গেট (`scripts/check-docs.sh`), একটি সোশ্যাল-প্রিভিউ রেন্ডার স্ক্রিপ্ট, এবং `tests/mod.rs` থেকে আলাদা করা শেয়ার্ড টেস্ট ফিক্সচার।

## [1.4.0] — 2026-10-25

### যা যোগ হয়েছে
- `bee_orm`: MySQL 8.0 / 8.4 / 9.x সামঞ্জস্য, CI ম্যাট্রিক্সে যাচাইকৃত। MySQL 9 কলাম-স্তরের `REFERENCES` প্রয়োগ করে (8.0/8.4 এটি পার্স করে উপেক্ষা করে); প্রথমে রেফারেন্সকৃত টেবিল তৈরি করুন।
- `bee_orm`: `shard::{ShardedPools, HashRouter, ShardRouter}` — ডেটাবেস ও টেবিলের জন্য key-required হ্যাশ শার্ডিং; ক্রস-শার্ড কোয়েরি বা ট্রানজেকশন নেই।
- `bee_kv` / `bee_cache`: প্রকৃত Redis কানেকশন পুল (`deadpool-redis`) এবং `bee_kv::RedisLock` (`SET NX PX` + Lua রিলিজ)।

## [1.3.0] — 2026-10-13

### যা ঠিক করা হয়েছে
- `bee_tsdb` / `bee_graph`: একটিমাত্র backend ফিচার আলাদা করে চালু করলে কম্পাইল হত না — শেয়ার করা `http_client` হেল্পারটি ফিচার-গেটেড `influxdb` / `neo4j` মডিউলের ভিতরে থাকত, তাই শুধু `iotdb` / `questdb` / `arangodb` / `nebulagraph` নিয়ে বিল্ড unresolved import-এ ব্যর্থ হত; এখন এটি প্রতিটি crate-এর নিজের `crate::http`-এ (যে backend গুলো এটি ব্যবহার করে ঠিক সেগুলোর উপর cfg গেট), আর প্রতিটি একক-backend ফিচার স্বতন্ত্রভাবে কম্পাইল হয় — ফিচার ম্যাট্রিক্স একে একে আবার চালানো হয়েছে
- `bee_router`: ঐচ্ছিক `security` ফিচার এখন **security-rust 4.0.0**-এ — `Scanner` / `DetectionResult` / `AttackCategory` / `Severity`-এর আকার 1.x-এর মতোই, bee_router-এর সোর্সে কোনো পরিবর্তন লাগেনি; সক্ষমতা বেড়েছে: অন্তর্নির্মিত ডিটেক্টর 32 (নতুন: `format_string`, `log4shell`, HPP, `formula_injection`, reDoS), ডিটেক্টরগুলো এখন শক্ত/দুর্বল স্তরে (দুর্বল সিগন্যাল `Severity::Low`-এ স্থির), এবং নতুন `Scanner::assess`। ফিল্টার আগের মতোই "কোনো হিট পেলেই আটকাও" নীতিতে চলে, তাই **দুর্বল সিগন্যালও (যেমন পুনরাবৃত্ত query প্যারামিটার) এখন আটকানো হবে** — কাস্টম scanner ঢোকানোর আর পথ নেই; এগুলো ছাড় দিতে চাইলে `SecurityFilter::scanner()` ঘিরে নিজের `Filter` বানান। নির্ভরতা-গ্রাফে "1.x + 4.x" এর বদলে এখন একটিই security-rust 4.0.0; ডকুমেন্টেশনও হালনাগাদ
- পুনরাবৃত্তি একত্রীকরণ: `bee_tsdb` / `bee_graph` / `bee_search`-এ সংযোগ ও error-mapping হেল্পারের একটিমাত্র কপি রইল (backend-এর নাম এখন কলার পাঠায়, ত্রুটির বার্তা অপরিবর্তিত; `aggregation_fn` গেছে `bee_tsdb`-এর lib-এ), `bee_graph`-এর `backtick` / `esc` তার `http` মডিউলে, `bee_orm` নিষ্কাশন করেছে `dedup_pk_values` / `index_rows_by` ও `migrate`-এর foreign-key ধারা, `bee_session::refresh` এখন `load` পুনর্ব্যবহার করে, আর `bee_config`-এর `watch_path` বাকি থাকা `wait_for_change` কল-পথটি শুষে নিয়েছে; `bee_search`-এর ত্রুটি-বার্তায় backend-এর নাম ফিরেছে, এবং `bee_kv`-র টেস্ট stub এখন কেবল তার টেস্টেই সীমিত
- পাবলিক API অপসারণ (তাই মাইনর সংস্করণ): `Context::set_params` / `param`, `SecurityFilter::with_scanner`, `Traversal::edge_labels` → `edge_label: Option<String>` (তিনটি backend-ই আগে থেকে কেবল প্রথম লেবেল নিত — আচরণ অপরিবর্তিত), `GraphError::EdgeNotFound`, `ConfigError::ParseError` / `TypeMismatch`, `bee_config::watch_path_timeout`, এবং `bee_logs::Logger::async_` (এমন সুইচ যা `init()` কখনও পড়ত না) — সবারই শূন্য কলার
- মৃত নির্ভরতা অপসারণ: `bee_router` (`tracing` / `http` / `bee_config` / `bee_logs`), `bee_session` (`async-trait`), `bee_config_macro` (`proc-macro2`), এবং `bee_tsdb` / `bee_graph` / `bee_search`-এ [dependencies]-এর সাথে পুনরাবৃত্ত dev `serde`

## [1.2.3] — 2026-10-06

### যা ঠিক করা হয়েছে
- `bee_cli`: স্ক্যাফোল্ড আর কম্পাইল-অযোগ্য প্রজেক্ট বানায় না — `new` টেমপ্লেটে প্যাকেজের ভুল নাম ছিল (`bee-rust`, যা রিজলভ হয় না; প্রকাশিত নাম `bee_rust`), আর কন্ট্রোলার টেমপ্লেটে `#[async_trait]` ছিল না এবং প্রাইভেট `RouterError` রুট পাথ ব্যবহৃত হচ্ছিল; নতুন `BEE_CLI_E2E`-গেটেড টেস্ট প্রজেক্ট বানিয়ে তাতে `cargo check` চালিয়ে পুরো পথ যাচাই করে
- ডকুমেন্টেশন: api-এর কন্ট্রোলার উদাহরণ এখন কম্পাইলযোগ্য রূপে (নিজস্ব `async-trait` নির্ভরতা, `bee_rust::bee_router::context::RouterError`); কেবল `bee_rust`-নির্ভর প্রজেক্টের `#[bee(crate = "bee_rust::bee_orm")]` লেখা, `ns`-এর খালি-প্রিফিক্স নিয়মসহ `{name}` রুট সিনট্যাক্স এবং `create()`-এর ব্যবহার লিপিবদ্ধ; বাংলা api ডকে ঢুকে পড়া একটি কোরিয়ান অংশ ঠিক করা হয়েছে
- `examples/shortlink`: অভ্যন্তরীণ ত্রুটির বিবরণ এখন লগে যায় (প্রতিক্রিয়ায় সাধারণ বার্তা), URL যাচাই খালি হোস্ট ও কন্ট্রোল অক্ষর প্রত্যাখ্যান করে, এবং `create_link` / `add_tag` এখন `Model::create()` ব্যবহার করে (302 হট পাথের `Click` ইনসার্ট `insert()`-ই থাকে)

## [1.2.2] — 2026-10-06

### যা যোগ হয়েছে
- `bee_orm`: `Model::create() -> Result<Self>` — ইনসার্ট করে সম্পূর্ণ ইনস্ট্যান্স ফেরত দেয়, ডেটাবেস-নির্ধারিত প্রাইমারি কী সহ (sqlite / postgres-এ `INSERT … RETURNING *`, mysql-এ `LAST_INSERT_ID()` + পুনঃপাঠ); `insert()` আগের মতোই, প্রভাবিত সারি ফেরত দেয়
- `bee_orm`: `#[bee(crate = "…")]` অ্যাট্রিবিউট — derive-এর ORM ক্রেট পাথ নির্দিষ্ট করে (শুধু `bee_rust` মেটা-ক্রেটে নির্ভর করলে `#[bee(crate = "bee_rust::bee_orm")]`); বাদ দিলে এক্সপ্যানশন বাইট-ধরে অপরিবর্তিত
- feature-নিয়ন্ত্রিত পাবলিক মডিউলওয়ালা আটটি ক্রেটে docs.rs বিল্ড কনফিগ পূরণ (`all-features = true`) — ব্যাকএন্ড feature-এর ডক পেজ ও derive অ্যাট্রিবিউট টেবিল docs.rs-এ আগে দেখা যেত না

## [1.2.1] — 2026-10-06

### যা যোগ হয়েছে
- `bee_orm`: opt-in features-এর পিছনে টাইপড তারিখ / Decimal ফিল্ড — `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) ও `rust_decimal` (`Decimal`) তিন ব্যাকএন্ডেই `Date` / `DateTime` / `DateTimeTz` / `Decimal` SQL টাইপে ম্যাপ হয়; `bee_rust` এগুলো `orm-chrono` / `orm-rust_decimal` হিসেবে ফরওয়ার্ড করে (`full`-এ নেই)
- `bee_orm`: এসব টাইপের সৎ সীমা — sqlite TEXT-এ রাখে (`typeof` = `text`; DECIMAL ঘোষণায় NUMERIC অ্যাফিনিটি মিলে `"1.50"` নীরবে REAL 1.5 হয়ে যায়), mysql `datetime(6)` / `timestamp(6)`-এর মাইক্রোসেকেন্ড রাখে (TIMESTAMP UTC হিসেবে পড়া হয়, DATETIME naive থাকে), আর pg `numeric` ২৯ তাৎপর্যপূর্ণ অঙ্কের বেশি হলে `NULL` পড়া যায়; `NaiveTime` / `DateTime<Local>` / `FixedOffset` / ন্যানোসেকেন্ড পরিধির বাইরে, `#[bee(sql_type = "…")]` লাগে, আর feature বন্ধ থাকলে মডেল কম্পাইল হয় না (E0277)

### যা ঠিক করা হয়েছে
- `bee_orm`: m2m সম্পর্ক-অনুপস্থিতির ইঙ্গিত এখন টেবিলের নামের বদলে টাইপ ident (`#[bee(m2m(Author))]`) লেখে, তাই নাম বদলানো টেবিলেও সাজেশনটি কম্পাইল হয়
- CI: `rust-toolchain`-এ স্পষ্ট `toolchain: stable` ইনপুট যোগ হয়েছে; feature ছাড়া `clippy --all-targets` ভাঙা দুটি টেস্ট ফাইলে ফাইল-স্তরের cfg গেট (bee_cache / bee_kv) এবং `bee_orm`-এর m2m doctest ব্যাকএন্ড-নিরপেক্ষ হলো; CLI `migrate`-এর এন্ড-টু-এন্ড টেস্ট এখন scratch crate অনলাইনে বানায় (অফলাইন কেবল গরম লোকাল ক্যাশে চলত) এবং `BEE_CLI_E2E=1` দিয়ে CI-তে যুক্ত

## [1.2.0] — 2026-10-06

### যা যোগ হয়েছে
- `bee_orm`: ORM এখন শুরু থেকে শেষ পর্যন্ত চলে — টাইপযুক্ত `Value` প্যারামিটার (Null / Bool / Int / Float / Text / Bytes), `#[bee(table / column / pk / auto / ignore)]`-সহ `#[derive(Model)]`, মডেলে `insert` / `update` / `delete`, এবং `QuerySet` এক্সিকিউশন (`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm`: তিন বেকএন্ডের জন্যই কানেকশন পুল (`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)`, `get()` → `CheckedConn`, `query` / `execute`, `status()`, এবং `begin` / `commit` / `rollback` দিয়ে ট্রানজ্যাকশন; ট্রানজ্যাকশনের মাঝপথে ফেলে দেওয়া কানেকশন sqlite ও postgres-এ রোলব্যাক হয়
- `bee_orm`: নতুন `postgres-tls` ফিচার (TLS-এ PostgreSQL, অন্তর্ভুক্ত Mozilla রুট) ও পুল সুদৃঢ়করণ — ৩০ সেকেন্ড অপেক্ষা / ১০ সেকেন্ড তৈরি টাইমআউট এবং postgres পুলে প্রিপেয়ার্ড-স্টেটমেন্ট ক্যাশে
- `bee_orm`: মডেল লাইফসাইকল — `#[bee(auto_now_add)]` / `#[bee(auto_now)]` স্বয়ংক্রিয় টাইমস্ট্যাম্প, `#[bee(soft_delete)]` সফট ডিলিট (delete ফ্ল্যাগ উল্টে দেয়; `with_deleted()` / `hard_delete()` তা এড়ায়), insert/update/delete হুক, `Model::insert_many` (৯৯৯ প্যারামিটারের ভাগে, ট্রানজ্যাকশনহীন), এবং `QuerySet::filter_in` ও `sum` / `avg` / `min` / `max` অ্যাগ্রিগেট
- `bee_orm`: অবিনাশী মাইগ্রেশন — `migrate::{create_table, add_missing_columns, sync}` ডায়ালেক্ট অনুযায়ী DDL তৈরি করে (sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`): টেবিল তৈরি ও অনুপস্থিত কলাম যোগ করে, কখনো মুছে বা বদলায় না
- `bee_orm`: বিদেশি-কী রিলেশন — `#[bee(fk = Target)]` (`#[bee(sql_type = "…")]`-সহ) FK DDL তৈরি করে, আর `rel::{fk_column_to, belongs_to, children, children_for}` তা পড়ে; প্রয়োগ নির্ভর করে বেকএন্ডের উপর: postgres ও sqlite (bundled বিল্ড) প্রয়োগ করে, mysql inline `REFERENCES` উপেক্ষা করে (round 5+ এর ফাঁক)
- `bee_orm`: অনেক-থেকে-অনেক রিলেশন — `#[bee(m2m(Target))]` রিলেশন ঘোষণা করে, `m2m::{attach, detach, related, related_for, related_ids}` পড়ে-লেখে, আর জয়েন টেবিল তৈরি করে `create_table` / `sync` (`add_missing_columns` তা স্পর্শ করে না)
- `bee_orm`: JSON কলাম — `serde_json::Value` ফিল্ড ব্যাকএন্ড অনুযায়ী `TEXT` / `JSONB` / `JSON`-এ ম্যাপ হয়, সৎ NULL অর্থবোধসহ: SQL `NULL` → `None`, সংরক্ষিত JSON `null` ডকুমেন্ট → `Some(Json::Null)`
- `bee_orm`: MySQL টেবিল-স্তরের বিদেশি-কী (opt-in) — `MigrateOptions { table_level_fk }`-সহ `sync_with` / `create_table_with` / `add_missing_columns_with`; ডিফল্টে বন্ধ, আর আগে থেকে থাকা অনাথ সারি নীরবে এড়িয়ে না গিয়ে `ADD CONSTRAINT` ব্যর্থ করে
- `bee_orm`: কাস্টম `rustls::ClientConfig`-এর জন্য `Pool::connect_tls_with`, সঙ্গে `bee_orm::rustls` পুনঃরপ্তানি যাতে সংস্করণ সবসময় মেলে; `connect_tls` বান্ডল করা webpki রুট রাখে
- `bee_rust`: চারটি ফরওয়ার্ডিং feature — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` `bee_rust`-এর মাধ্যমে `bee_orm` ব্যাকএন্ড ফরওয়ার্ড করে (কোনোটিই `full`-এ নেই)
- `bee_cli`: `bee-rust migrate init` `src/bin/bee_migrate.rs` তৈরি করে (বিদ্যমান ফাইল ওভাররাইট করে না), `bee-rust migrate run` তা `cargo run --bin bee_migrate` দিয়ে চালায়
- `bee_kv` / `bee_cache`: Redis ব্যাকএন্ড কাটা সংযোগ থেকে সেরে ওঠে — অন্তর্নিহিত `ConnectionManager` চাহিদা অনুযায়ী (পটভূমি থ্রেড ছাড়া) সূচকীয় ব্যাকঅফ ও জিটারে পুনঃসংযুক্ত হয় (কাটায় ধাক্কা খাওয়া কমান্ড ব্যর্থ, পরেরটি নতুন সংযোগের অপেক্ষা করে)
- `bee_kv` / `bee_cache`: memcached ব্যাকএন্ড (bee_kv-তে `MemcacheStore`, bee_cache-এ `MemcacheCache`); `incr` ডেল্টার আগে কাউন্টার বানায়, unsigned কাউন্টার 0-তে থামে, আর TTL 0 কী মুছে দেয়

### যা পরিবর্তন করা হয়েছে
- ডকুমেন্টে bee_orm-এর মাইগ্রেশন, রিলেশন (many-to-many সহ), `bee-rust migrate` সাবকমান্ড, Redis / Memcached ব্যাকএন্ড ও `bee_rust`-এর ORM ফরওয়ার্ডিং feature আর পরিকল্পিত নয়
- ডকুমেন্টে search / graph / tsdb ড্রাইভার আর পরিকল্পিত বা trait-stub হিসেবে নয়, বাস্তবায়িত (opt-in feature) হিসেবে উল্লেখ করা হয়েছে, এবং পুরোনো উদাহরণের টাইপ নাম ও সংযোগ পদ্ধতি সংশোধন করা হয়েছে (অনুপস্থিত `ElasticsearchEngine` / `Neo4jDB`, ভুল `bolt://`)

## [1.1.5] — 2026-09-25

### যা যোগ হয়েছে
- আরও ১১টি ভাষার সংক্ষিপ্ত crates.io README: `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`। crates.io পৃষ্ঠার ভাষা সুইচার এখন রিপোজিটরির পূর্ণ README-এর বদলে এই ফাইলগুলো নির্দেশ করে, ফলে প্রতিটি ভাষা একই সংক্ষিপ্ত ডেভেলপার-কেন্দ্রিক ডকুমেন্ট পায়।

## [1.1.4] — 2026-09-25

### যা পরিবর্তন করা হয়েছে
- crates.io পৃষ্ঠা (`docs/crates-readme.md`) এখন দ্বিভাষিক: উপরে ইংরেজি, নিচে চীনা। crates.io প্রতি ক্রেটে কেবল একটি README রেন্ডার করে এবং পৃষ্ঠার ভিতরে ভাষা পরিবর্তনের সুবিধা দেয় না, তাই ইংরেজি প্রথম স্ক্রিনে এসেছে এবং চীনা লেখা একই পৃষ্ঠায় রয়েছে।

## [1.1.3] — 2026-09-25

### যা যোগ হয়েছে
- crates.io পৃষ্ঠায় (`docs/crates-readme.md`) এখন 13 ভাষার সুইচার যোগ করা হয়েছে, যা রিপোজিটরির README-গুলোর সঙ্গে যুক্ত

### যা ঠিক করা হয়েছে
- `docs/api.*`: 11টি অনুবাদে চীনা মূল (`api.md`) এর লিঙ্ক অনুপস্থিত ছিল
- `docs/CHANGELOG.zh.md` ও `docs/CONTRIBUTING.zh.md` ছিল নিজ নিজ গ্রুপে একমাত্র ফাইল যেগুলোতে স্ব-লিঙ্ক ছিল না

## [1.1.2] — 2026-09-24

### যা যোগ হয়েছে
- `docs/crates-readme.md`: crates.io পৃষ্ঠাগুলোর জন্য আলাদা README — ইনস্টল, চালানোর যোগ্য উদাহরণ (`examples/hello` থেকে নেওয়া, যার E2E টেস্ট এটি কভার করে), feature ফ্ল্যাগ টেবিল এবং সাব-ক্রেট তালিকা
- `scripts/publish.sh`: ওয়ার্কস্পেসের ক্রেটগুলো এক এক করে প্রকাশ করে, আগেই আপলোড হওয়া ক্রেট এড়িয়ে যায়, এবং রেট লিমিট হলে crates.io যে পুনঃপ্রচেষ্টার সময় ফেরত দেয় সেটি পর্যন্ত অপেক্ষা করে

### যা ঠিক করা হয়েছে
- টেস্ট স্যুট কম্পাইল হত না (main-এ `cargo test --workspace` ব্যর্থ): `bee_router`-এর ইন্টিগ্রেশন টেস্টে `Submit`-এ `Serialize` অনুপস্থিত ছিল (এটি `Json` রেসপন্স হিসেবে ফেরত দেওয়া হয়), দুটি হেল্পারে `Router::build()` অনুপস্থিত ছিল, এবং আটটি `status_line` কলে `&` অনুপস্থিত ছিল
- `hello` উদাহরণের টেস্ট ভুল `CARGO_BIN_EXE` নাম উল্লেখ করছিল — বাইনারি টার্গেট তার হাইফেন ধরে রাখে (`CARGO_BIN_EXE_hello-bee`)
- `hello`-এর টেমপ্লেট অটোএস্কেপ টেস্ট `&#39;` যাচাই করছিল, অথচ tera `&#x27;` আউটপুট দেয়; এস্কেপিং আচরণ নিজেই সঠিক ছিল
- `bee_cli`-তে Clippy `manual_split_once` ও `manual_range_contains` লিন্ট; `bee_router`-এর ইন্টিগ্রেশন টেস্টে অব্যবহৃত import

### যা পরিবর্তন করা হয়েছে
- সব ক্রেটে এখন `readme` ও `repository` মেটাডেটা আছে — আগে কোনো crates.io পৃষ্ঠাতেই README রেন্ডার হত না, এবং ছয়টি ক্রেটে `repository` একেবারেই ছিল না
- `examples/hello` কে `publish = false` চিহ্নিত করা হয়েছে: এটি E2E টেস্ট হার্নেস হিসেবে ওয়ার্কস্পেসে থাকে, তবে crates.io-তে আর প্রকাশিত হয় না

## [1.0.6] — 2026-08-07

### যা যোগ হয়েছে
- `bee_cli`-এর বাস্তব বাস্তবায়ন: `new` (প্রজেক্ট স্ক্যাফোল্ডিং), `generate controller/model`, `--watch` হট-রিলোডসহ `run`, `pack` (release বিল্ড + `dist/`-তে কপি)
- স্ক্যাফোল্ডিং ও কোড জেনারেশনের জন্য CLI ইউনিট টেস্ট (৭টি নতুন টেস্ট)

### যা ঠিক করা হয়েছে
- `bee_rust::init()` এখন `logs` feature-এর পেছনে গেট করা হয়েছে — কমানো feature বিল্ড (যেমন `--no-default-features --features kv`) আবার কম্পাইল হয়
- `bee_kv::InMemoryKvStore::exists`-এ Clippy `unnecessary_map_or` lint
- `rustfmt.toml` থেকে নাইটলি-অনলি অপশন সরানো হয়েছে যেগুলো stable-এ নীরবে উপেক্ষা করা হতো; ওয়ার্কস্পেস এখন `cargo fmt --all --check` পাস করে
- `bee_cli` বাইনারিতে `doc = false` যাতে `bee_rust`-এর সাথে rustdoc আউটপুট ফাইলনেম সংঘর্ষ না হয়
- `hello` উদাহরণের পোর্ট এখন `PORT` env ভেরিয়েবলের মাধ্যমে কনফিগারযোগ্য

### যা পরিবর্তন করা হয়েছে
- `bee-rust migrate` "not implemented" রিপোর্ট করে এবং নন-জিরো দিয়ে বেরিয়ে যায় (পরিকল্পনাধীন)
- README / README.en আপডেট করে প্রকৃত CLI আচরণ বর্ণনা করা হয়েছে

## [1.0.4] — 2026-07-29

### যা যোগ হয়েছে
- `security-rust`-এর মাধ্যমে নিরাপত্তা আক্রমণ শনাক্তকরণ ফিল্টার (২৭টি ডিটেক্টর)
- XSS, SQL ইনজেকশন, কমান্ড ইনজেকশন, পাথ ট্রাভার্সাল কভারেজসহ `SecurityFilter`
- `bee_rust` ও `bee_router`-এ `security` feature flag

### যা পরিবর্তন করা হয়েছে
- README আপডেট করে নিরাপত্তা ফিচারের ডকুমেন্টেশন যোগ করা হয়েছে
- README আপডেট করে পেমেন্ট সাপোর্ট সেকশন যোগ করা হয়েছে (WeChat Pay / Alipay)

### যা ঠিক করা হয়েছে
- Rust 2024 এডিশনের জন্য `bee_template`-এ Tera র-আইডেন্টিফায়ার সিনট্যাক্স

## [1.0.3] — 2026-07-29

### যা যোগ হয়েছে
- ১৩টি crate-সহ প্রাথমিক ওয়ার্কস্পেস কাঠামো
- `Controller` trait ও `Router`-সহ MVC রাউটিং
- `QuerySet` বিল্ডার ও `Model` ডেরাইভ ম্যাক্রোসহ ORM
- Redis ও Memory ব্যাকএন্ডসহ KV/Cache trait অ্যাবস্ট্রাকশন
- Memory/Redis ব্যাকএন্ডসহ সেশন ম্যানেজমেন্ট
- INI/YAML/ENV সাপোর্ট ও হট-রিলোডসহ কনফিগ ম্যানেজমেন্ট
- Tera-র মাধ্যমে টেমপ্লেট রেন্ডারিং
- tracing ইন্টিগ্রেশনসহ লগিং
- CLI স্ক্যাফোল্ডিং ও কোড জেনারেশন
- সার্চ, গ্রাফ, টাইম-সিরিজ ইঞ্জিন trait stub (ড্রাইভার পরিকল্পনাধীন)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
