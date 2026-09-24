<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

Rust로 만든 프로덕션급 웹 프레임워크입니다. Go의 Beego 설계 철학을 Rust다운 trait / macro / 타입 시스템으로 다시 표현했습니다.

MVC 컨트롤러 · 네임스페이스 라우팅 · 필터 체인 · ORM · 스토리지 엔진 통합 trait 추상화

## 설치

```bash
cargo add bee_rust
```

또는 `Cargo.toml`에 작성：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

기본 `full` feature는 `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`를 활성화합니다.

## 빠른 시작

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

완전히 동작하는 프로젝트는 [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello)에 있습니다(E2E 테스트 포함).

## Feature 플래그

`search`, `graph`, `tsdb`는 기본 `full`에 **포함되지 않습니다**. 명시적으로 활성화하세요.

| Feature | 포함되는 것 | 설명 |
|---------|-----------|-------|
| `full` *(기본)* | router, orm, kv, config, logs, cache, session, security, template | 올인원 |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Web 코어: 라우팅 + 컨트롤러 + 필터 체인 |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | KV 추상화 |
| `cache` | bee_cache, bee_config | 캐시 추상화 |
| `session` | bee_session, bee_cache | 다중 백엔드 세션 |
| `config` | bee_config | INI / YAML / ENV + 핫 리로드 |
| `logs` | bee_logs | 레벨별 로깅 + tracing |
| `template` | bee_template | tera 템플릿 |
| `security` | bee_router/security | 27종 공격 탐지 필터 |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

최소 구성(ORM과 데이터베이스 드라이버 제외)：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## 서브 크레이트

이 프레임워크는 독립적으로 사용할 수 있는 크레이트 모음이며, `bee_rust`는 필요에 따라 이들을 재수출하는 단일 진입점입니다.

| Crate | 기능 | Beego 대응 |
|-------|--------------|-------------------|
| `bee_rust` | 메타 크레이트, 단일 진입점 | — |
| `bee_router` | 라우팅 + 컨트롤러 + `Context` + 필터 | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + 마이그레이션 | `client/orm` |
| `bee_config` | 설정 + 핫 리로드 | `client/config` |
| `bee_logs` | 로깅 | `logs` |
| `bee_cache` | 캐시 추상화 | `client/cache` |
| `bee_session` | 세션 관리 | `server/web/session` |
| `bee_template` | 템플릿 렌더링 | — (확장) |
| `bee_kv` | KV/캐시 추상화 | `client/cache` (확장) |
| `bee_search` | 검색 / 분석 엔진 | — (신규) |
| `bee_graph` | 그래프 데이터베이스 | — (신규) |
| `bee_tsdb` | 시계열 데이터베이스 | — (신규) |
| `bee_cli` | 스캐폴딩 / 코드 생성 / 개발 러너 | `bee` 도구 |

이들은 개별적으로 의존할 수도 있습니다. 예를 들어 Elasticsearch 클라이언트만 쓰는 경우：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## 지원 데이터베이스

| 종류 | 데이터베이스 | Crate | Feature |
|------|-----------|-------|---------|
| 관계형 | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / 캐시 | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| 검색 / 분석 | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| 그래프 | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| 시계열 | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## 요구 사항

- Rust 1.80+ (MSRV)
- edition 2024

## 링크

- **API 문서**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **저장소**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **전체 API 레퍼런스(13개 언어)**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## 라이선스

Apache-2.0
