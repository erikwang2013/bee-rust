# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
