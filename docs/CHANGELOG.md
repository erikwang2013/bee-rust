# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
