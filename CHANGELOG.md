# Changelog

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
