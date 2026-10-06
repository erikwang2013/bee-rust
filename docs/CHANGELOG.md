# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.2.2] — 2026-10-06

### Added
- `bee_orm`: `Model::create() -> Result<Self>` — insert and get the full instance back, database-assigned primary key included (`INSERT … RETURNING *` on sqlite / postgres, `LAST_INSERT_ID()` + read-back on mysql); `insert()` is unchanged and still returns affected rows
- `bee_orm`: the `#[bee(crate = "…")]` attribute — point the derive at the ORM crate path (e.g. `#[bee(crate = "bee_rust::bee_orm")]` when only the `bee_rust` meta crate is a dependency); omitting it keeps the expansion byte-identical
- docs.rs build config completed for the eight crates with feature-gated public modules (`all-features = true`) — backend feature docs and the derive attribute table were previously invisible on docs.rs

## [1.2.1] — 2026-10-06

### Added
- `bee_orm`: typed date/time and decimal fields behind opt-in features — `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) and `rust_decimal` (`Decimal`) map to the `Date` / `DateTime` / `DateTimeTz` / `Decimal` SQL types on all three backends; `bee_rust` forwards them as `orm-chrono` / `orm-rust_decimal` (not in `full`)
- `bee_orm`: honest boundaries for those types — sqlite stores them as TEXT (`typeof` = `text`; a DECIMAL declaration would gain NUMERIC affinity and silently turn `"1.50"` into REAL 1.5), mysql keeps microseconds in `datetime(6)` / `timestamp(6)` (TIMESTAMP reads back as UTC, DATETIME stays naive), and a pg `numeric` above 29 significant digits reads back as `NULL`; `NaiveTime` / `DateTime<Local>` / `FixedOffset` / nanoseconds are out of scope and need `#[bee(sql_type = "…")]`, and with the feature off the model fails to compile (E0277)

### Fixed
- `bee_orm`: the many-to-many missing-relation hint now spells the target type ident (`#[bee(m2m(Author))]`) instead of the table name, so the suggestion still compiles for a renamed table
- CI: `rust-toolchain` gained the explicit `toolchain: stable` input; the two test files that broke `clippy --all-targets` in the feature-less configuration got file-level cfg gates (bee_cache / bee_kv) and the `bee_orm` m2m doctest became backend-agnostic; the CLI `migrate` end-to-end test now builds its scratch crate online (offline only passed on a warm local cache) and runs in CI via `BEE_CLI_E2E=1`

## [1.2.0] — 2026-10-06

### Added
- `bee_orm`: the ORM now runs end to end — typed `Value` bind parameters (Null / Bool / Int / Float / Text / Bytes), `#[derive(Model)]` with `#[bee(table / column / pk / auto / ignore)]`, model `insert` / `update` / `delete`, and `QuerySet` execution (`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm`: connection pools for all three backends (`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)`, `get()` → `CheckedConn`, `query` / `execute`, `status()`, and transactions via `begin` / `commit` / `rollback`; a connection dropped mid-transaction is rolled back on sqlite and postgres
- `bee_orm`: new `postgres-tls` feature (PostgreSQL over TLS with bundled Mozilla roots) and pool hardening — 30 s wait / 10 s create timeouts plus a prepared-statement cache on the postgres pool
- `bee_orm`: model lifecycle — `#[bee(auto_now_add)]` / `#[bee(auto_now)]` timestamps, `#[bee(soft_delete)]` (delete flips the flag; `with_deleted()` / `hard_delete()` escape it), lifecycle hooks (before/after insert/update/delete), `Model::insert_many` (999-parameter chunks, not transactional) and `QuerySet::filter_in` plus `sum` / `avg` / `min` / `max` aggregates
- `bee_orm`: non-destructive migrations — `migrate::{create_table, add_missing_columns, sync}` render dialect-aware DDL (sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`), adding tables and missing columns but never dropping or altering
- `bee_orm`: foreign-key relations — `#[bee(fk = Target)]` (with `#[bee(sql_type = "…")]`) emits FK DDL, and `rel::{fk_column_to, belongs_to, children, children_for}` read them; enforcement follows the backend: postgres and sqlite (bundled build) enforce the reference, mysql ignores inline `REFERENCES` (a gap for round 5+)
- `bee_orm`: many-to-many relations — `#[bee(m2m(Target))]` declares the relation, `m2m::{attach, detach, related, related_for, related_ids}` read and write it, and `create_table` / `sync` create the join table (`add_missing_columns` never touches it)
- `bee_orm`: JSON columns — `serde_json::Value` fields map to `TEXT` / `JSONB` / `JSON` per backend, with honest NULL semantics: SQL `NULL` decodes to `None`, a stored JSON `null` document to `Some(Json::Null)`
- `bee_orm`: opt-in MySQL table-level foreign keys — `MigrateOptions { table_level_fk }` with `sync_with` / `create_table_with` / `add_missing_columns_with`; off by default, and pre-existing orphan rows make `ADD CONSTRAINT` fail instead of being skipped silently
- `bee_orm`: `Pool::connect_tls_with` for a custom `rustls::ClientConfig`, plus a `bee_orm::rustls` re-export so the config version always matches; `connect_tls` keeps the bundled webpki roots
- `bee_rust`: four forwarding features — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` forward a `bee_orm` backend through `bee_rust` (none of them are in `full`)
- `bee_cli`: `bee-rust migrate init` scaffolds `src/bin/bee_migrate.rs` (refuses to overwrite an existing file) and `bee-rust migrate run` executes it via `cargo run --bin bee_migrate`
- `bee_kv` / `bee_cache`: the Redis backends survive dropped connections — the underlying `ConnectionManager` reconnects on demand (no background thread) with exponential backoff and jitter (the command that hits the break errors; the next one waits for the fresh connection)
- `bee_kv` / `bee_cache`: memcached backends (`MemcacheStore` in bee_kv, `MemcacheCache` in bee_cache); `incr` creates the counter before applying the delta, unsigned counters floor at 0, and TTL 0 deletes the key

### Changed
- bee_orm migrations, relations (including many-to-many), the `bee-rust migrate` CLI subcommand, the Redis / Memcached backends, and the `bee_rust` ORM forwarding features are no longer marked as planned in the docs
- the search / graph / tsdb drivers are no longer marked in the docs as planned or as a trait stub, but as implemented (opt-in features); the old examples were corrected (nonexistent `ElasticsearchEngine` / `Neo4jDB` type names, wrong `bolt://` scheme)

## [1.1.5] — 2026-09-25

### Added
- Condensed crates.io READMEs for 11 more languages: `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`. The language switcher on the crates.io page now points at these instead of the full repository READMEs, so every language gets the same short developer-facing document.

## [1.1.4] — 2026-09-25

### Changed
- The crates.io page (`docs/crates-readme.md`) is now bilingual: English first, Chinese below. crates.io renders only one README per crate and has no in-page language switch, so this puts English on the first screen while keeping the Chinese text on the same page.

## [1.1.3] — 2026-09-25

### Added
- The crates.io page (`docs/crates-readme.md`) now carries a 13-language switcher linking to the repository READMEs

### Fixed
- `docs/api.*`: 11 translations were missing the link to the Chinese original (`api.md`)
- `docs/CHANGELOG.zh.md` and `docs/CONTRIBUTING.zh.md` were the only files in their families without a self-link

## [1.1.2] — 2026-09-24

### Added
- `docs/crates-readme.md`: a dedicated README for the crates.io pages — install, a runnable example (taken from `examples/hello`, which is covered by its E2E tests), a feature-flag table, and a sub-crate index
- `scripts/publish.sh`: publishes workspace crates one by one, skips anything already uploaded, and honours the retry time crates.io returns when rate limiting

### Fixed
- Test suite did not compile (`cargo test --workspace` failed on main): the `bee_router` integration test was missing `Serialize` on `Submit` (it is returned as a `Json` response), `Router::build()` on two helpers, and `&` on eight `status_line` calls
- `hello` example tests referenced the wrong `CARGO_BIN_EXE` name — the binary target keeps its hyphen (`CARGO_BIN_EXE_hello-bee`)
- `hello` template autoescape test asserted `&#39;` while tera emits `&#x27;`; the escaping behaviour itself was correct
- Clippy `manual_split_once` and `manual_range_contains` lints in `bee_cli`; an unused import in `bee_router`'s integration test

### Changed
- All crates now carry `readme` and `repository` metadata — previously no crates.io page rendered a README and six crates lacked `repository` entirely
- `examples/hello` is marked `publish = false`: it stays in the workspace as the E2E test harness but is no longer published to crates.io

## [1.0.6] — 2026-08-07

### Added
- `bee_cli` real implementations: `new` (project scaffolding), `generate controller/model`, `run` with `--watch` hot reload, `pack` (release build + copy to `dist/`)
- CLI unit tests for scaffolding and code generation (7 new tests)

### Fixed
- `bee_rust::init()` now gated behind the `logs` feature — reduced feature builds (e.g. `--no-default-features --features kv`) compile again
- Clippy `unnecessary_map_or` lint in `bee_kv::InMemoryKvStore::exists`
- `rustfmt.toml` removed nightly-only options that were silently ignored on stable; workspace now passes `cargo fmt --all --check`
- `bee_cli` binary `doc = false` to remove rustdoc output filename collision with `bee_rust`
- `hello` example port is now configurable via `PORT` env var

### Changed
- `bee-rust migrate` reports "not implemented" and exits non-zero (planned)
- README / README.en updated to describe actual CLI behavior

## [1.0.4] — 2026-07-29

### Added
- Security attack detection filter via `security-rust` (27 detectors)
- `SecurityFilter` with XSS, SQL injection, command injection, path traversal coverage
- `security` feature flag in `bee_rust` and `bee_router`

### Changed
- Updated README with security feature documentation
- Updated README with payment support section (WeChat Pay / Alipay)

### Fixed
- `bee_template` Tera raw identifier syntax for Rust 2024 edition

## [1.0.3] — 2026-07-29

### Added
- Initial workspace structure with 13 crates
- MVC routing with `Controller` trait and `Router`
- ORM with `QuerySet` builder and `Model` derive macro
- KV/Cache trait abstraction with Redis and Memory backends
- Session management with Memory/Redis backends
- Config management with INI/YAML/ENV support and hot-reload
- Template rendering via Tera
- Logging with tracing integration
- CLI scaffolding and code generation
- Search, Graph, Time-series engine trait stubs (drivers planned)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
