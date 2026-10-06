# 변경 로그 (Changelog)

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.2.1] — 2026-10-06

### 추가됨
- `bee_orm`: opt-in feature로 제어하는 날짜 / Decimal 필드 — `chrono`(`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`)와 `rust_decimal`(`Decimal`)이 세 백엔드에서 `Date` / `DateTime` / `DateTimeTz` / `Decimal` SQL 타입으로 매핑; `bee_rust`는 `orm-chrono` / `orm-rust_decimal`로 전달(`full` 미포함)
- `bee_orm`: 이 타입들의 솔직한 경계 — sqlite는 TEXT 저장 (`typeof` = `text`; DECIMAL 선언은 NUMERIC 친화도를 얻어 `"1.50"`을 조용히 REAL 1.5로 만듭니다), mysql은 `datetime(6)` / `timestamp(6)` 마이크로초 유지(TIMESTAMP는 UTC로 읽고 DATETIME은 naive 유지), pg `numeric`은 유효 자릿수 29 초과 시 `NULL`로 읽힘; `NaiveTime` / `DateTime<Local>` / `FixedOffset` / 나노초는 범위 밖이라 `#[bee(sql_type = "…")]` 필요, feature를 끄면 모델이 컴파일되지 않음(E0277)

### 수정됨
- `bee_orm`: m2m 관계 미선언 힌트가 이제 테이블 이름이 아닌 타입 ident(`#[bee(m2m(Author))]`)를 쓰므로 이름이 바뀐 테이블에서도 그대로 컴파일됩니다
- CI: `rust-toolchain`에 명시적 `toolchain: stable` 입력 추가; feature 없는 구성에서 `clippy --all-targets`를 깨뜨리던 테스트 파일 두 개에 파일 단위 cfg 게이트(bee_cache / bee_kv), `bee_orm` m2m doctest는 백엔드 비의존으로 변경; CLI `migrate` E2E 테스트는 내부 빌드를 온라인으로(오프라인은 따뜻한 로컬 캐시에서만 통과) 바꾸고 `BEE_CLI_E2E=1`로 CI에 연결

## [1.2.0] — 2026-10-06

### 추가됨
- `bee_orm`: ORM이 엔드투엔드로 동작합니다 — 타입화된 `Value` 파라미터(Null / Bool / Int / Float / Text / Bytes), `#[bee(table / column / pk / auto / ignore)]`를 지원하는 `#[derive(Model)]`, 모델의 `insert` / `update` / `delete`, 그리고 `QuerySet` 실행(`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm`: 세 백엔드 모두 커넥션 풀 제공(`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)`, `get()` → `CheckedConn`, `query` / `execute`, `status()`, `begin` / `commit` / `rollback` 트랜잭션; 트랜잭션 도중 버려진 커넥션은 sqlite와 postgres에서 롤백됩니다
- `bee_orm`: 새 feature `postgres-tls`(TLS 기반 PostgreSQL, Mozilla 루트 인증서 내장)와 풀 강화 — 30초 대기 / 10초 생성 타임아웃, postgres 풀의 프리페어드 스테이트먼트 캐시
- `bee_orm`: 모델 라이프사이클 — `#[bee(auto_now_add)]` / `#[bee(auto_now)]` 자동 타임스탬프, `#[bee(soft_delete)]` 소프트 삭제(delete가 플래그를 뒤집고 `with_deleted()` / `hard_delete()`로 우회), insert/update/delete 훅, `Model::insert_many`(999 파라미터 단위 분할, 비트랜잭션), 그리고 `QuerySet::filter_in`과 `sum` / `avg` / `min` / `max` 집계
- `bee_orm`: 비파괴 마이그레이션 — `migrate::{create_table, add_missing_columns, sync}`가 방언별 DDL(sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`)을 생성해 테이블 생성과 누락 컬럼 추가만 하고 삭제나 변경은 하지 않습니다
- `bee_orm`: 외래 키 관계 — `#[bee(fk = Target)]`(`#[bee(sql_type = "…")]`와 함께)가 FK DDL을 생성하고 `rel::{fk_column_to, belongs_to, children, children_for}`로 읽습니다. 강제 여부는 백엔드에 따라 다릅니다: postgres와 sqlite(bundled 빌드)는 강제하고, mysql은 inline `REFERENCES`를 무시합니다(round 5+의 gap)
- `bee_orm`: 다대다 관계 — `#[bee(m2m(Target))]`가 관계를 선언하고 `m2m::{attach, detach, related, related_for, related_ids}`가 읽고 쓰며, 조인 테이블은 `create_table` / `sync`가 생성합니다(`add_missing_columns`는 건드리지 않음)
- `bee_orm`: JSON 컬럼 — `serde_json::Value` 필드는 백엔드별로 `TEXT` / `JSONB` / `JSON`에 매핑되며 NULL 의미론은 정직합니다: SQL `NULL` → `None`, 저장된 JSON `null` 문서 → `Some(Json::Null)`
- `bee_orm`: MySQL 테이블 수준 외래 키 opt-in — `MigrateOptions { table_level_fk }`와 `sync_with` / `create_table_with` / `add_missing_columns_with`; 기본은 꺼짐이며, 기존 고아 행은 조용히 건너뛰지 않고 `ADD CONSTRAINT`를 실패시킵니다
- `bee_orm`: 사용자 정의 `rustls::ClientConfig`를 위한 `Pool::connect_tls_with`와 버전 일치를 보장하는 `bee_orm::rustls` 재수출; `connect_tls`는 내장 webpki 루트 유지
- `bee_rust`: 네 가지 전달 feature — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql`이 `bee_rust`를 통해 `bee_orm` 백엔드를 전달합니다(모두 `full`에 미포함)
- `bee_cli`: `bee-rust migrate init`이 `src/bin/bee_migrate.rs`를 생성하고(기존 파일은 덮어쓰지 않음), `bee-rust migrate run`이 `cargo run --bin bee_migrate`로 실행합니다
- `bee_kv` / `bee_cache`: Redis 백엔드는 끊긴 연결에서 복구됩니다 — 내부 `ConnectionManager`가 필요할 때(백그라운드 스레드 없이) 지수 백오프와 지터로 재연결(끊김에 걸린 명령은 오류, 다음 명령은 대기)
- `bee_kv` / `bee_cache`: memcached 백엔드(bee_kv의 `MemcacheStore`, bee_cache의 `MemcacheCache`); `incr`은 델타 전에 카운터를 만들고, 부호 없는 카운터는 0에서 멈추며, TTL 0은 키를 삭제합니다

### 변경됨
- bee_orm의 마이그레이션, 관계(many-to-many 포함), `bee-rust migrate` 하위 명령, Redis / Memcached 백엔드, `bee_rust`의 ORM 전달 feature는 문서에서 더 이상 '계획 중'으로 표기되지 않습니다
- 문서에서 search / graph / tsdb 드라이버를 '계획 중 / trait 스텁'에서 구현 완료(opt-in feature)로 바꾸고, 예전 예제의 타입 이름과 연결 방식도 수정(존재하지 않는 `ElasticsearchEngine` / `Neo4jDB`, 잘못된 `bolt://`)

## [1.1.5] — 2026-09-25

### 추가됨
- 나머지 11개 언어의 축약형 crates.io README: `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`. crates.io 페이지 상단의 언어 전환이 이제 저장소의 전체 README 대신 이 파일들을 가리키므로, 어느 언어든 동일한 개발자용 요약 문서를 받습니다.

## [1.1.4] — 2026-09-25

### 변경됨
- crates.io 페이지(`docs/crates-readme.md`)를 중영 이중 언어로 변경: 영어가 위, 중국어가 아래. crates.io는 크레이트당 README를 하나만 렌더링하고 페이지 내 언어 전환도 지원하지 않으므로, 영어를 첫 화면에 두면서 중국어도 같은 페이지에 유지합니다.

## [1.1.3] — 2026-09-25

### 추가됨
- crates.io 페이지(`docs/crates-readme.md`)에 13개 언어 전환 링크 추가(저장소의 각 언어 README로 연결)

### 수정됨
- `docs/api.*`: 11개 번역본에 중국어 원본(`api.md`)으로 가는 링크가 누락되어 있었음
- `docs/CHANGELOG.zh.md`와 `docs/CONTRIBUTING.zh.md`는 각 문서군에서 유일하게 자기 링크가 없는 파일이었음

## [1.1.2] — 2026-09-24

### 추가됨
- `docs/crates-readme.md`: crates.io 페이지 전용 README — 설치, 실행 가능한 예제(`examples/hello`에서 가져옴, 해당 E2E 테스트로 검증), feature 플래그 표, 서브 크레이트 목록
- `scripts/publish.sh`: 워크스페이스 크레이트를 하나씩 배포하고, 이미 업로드된 것은 건너뛰며, 속도 제한 시 crates.io가 반환한 재시도 시각까지 대기

### 수정됨
- 테스트 스위트가 컴파일되지 않았음(`cargo test --workspace`가 main에서 실패): `bee_router` 통합 테스트에서 `Submit`에 `Serialize` 누락(이는 `Json` 응답으로 반환됨), 두 헬퍼에 `Router::build()` 누락, 8곳의 `status_line` 호출에 `&` 누락
- `hello` 예제 테스트가 잘못된 `CARGO_BIN_EXE` 이름을 참조 — 바이너리 타깃은 하이픈을 유지함(`CARGO_BIN_EXE_hello-bee`)
- `hello` 템플릿 자동 이스케이프 테스트가 `&#39;`를 검증했으나 tera는 `&#x27;`를 출력함. 이스케이프 동작 자체는 정상이었음
- `bee_cli`의 Clippy `manual_split_once` / `manual_range_contains` lint, `bee_router` 통합 테스트의 미사용 import

### 변경됨
- 모든 크레이트가 이제 `readme`와 `repository` 메타데이터를 가짐 — 이전에는 crates.io 페이지에 README가 렌더링되지 않았고 6개 크레이트는 `repository`가 아예 없었음
- `examples/hello`를 `publish = false`로 표시: E2E 테스트 하네스로서 워크스페이스에는 남지만 crates.io에는 더 이상 배포되지 않음

## [1.0.6] — 2026-08-07

### 추가됨
- `bee_cli` 실제 구현: `new` (프로젝트 스캐폴딩), `generate controller/model`, `--watch` 핫 리로드가 포함된 `run`, `pack` (release 빌드 + `dist/`로 복사)
- 스캐폴딩 및 코드 생성을 위한 CLI 단위 테스트 (신규 테스트 7개)

### 수정됨
- `bee_rust::init()`이 이제 `logs` feature 뒤로 게이트됨 — 축소된 feature 빌드 (예: `--no-default-features --features kv`)가 다시 컴파일됨
- `bee_kv::InMemoryKvStore::exists`의 Clippy `unnecessary_map_or` 린트
- `rustfmt.toml`에서 stable에서 조용히 무시되던 nightly 전용 옵션 제거; 이제 워크스페이스가 `cargo fmt --all --check`를 통과함
- `bee_cli` 바이너리에 `doc = false`를 적용하여 `bee_rust`와의 rustdoc 출력 파일명 충돌 제거
- `hello` 예제의 포트를 이제 `PORT` 환경 변수로 설정 가능

### 변경됨
- `bee-rust migrate`가 "not implemented"를 보고하고 0이 아닌 코드로 종료함 (계획됨)
- 실제 CLI 동작을 설명하도록 README / README.en 업데이트

## [1.0.4] — 2026-07-29

### 추가됨
- `security-rust`를 통한 보안 공격 감지 필터 (탐지기 27개)
- XSS, SQL 인젝션, 명령어 인젝션, 경로 탐색을 다루는 `SecurityFilter`
- `bee_rust`와 `bee_router`의 `security` feature 플래그

### 변경됨
- 보안 feature 문서를 포함하도록 README 업데이트
- 결제 지원 섹션 (WeChat Pay / Alipay)을 포함하도록 README 업데이트

### 수정됨
- Rust 2024 에디션을 위한 `bee_template` Tera 원시 식별자(raw identifier) 구문

## [1.0.3] — 2026-07-29

### 추가됨
- 13개 crate로 구성된 초기 워크스페이스 구조
- `Controller` trait과 `Router`를 사용한 MVC 라우팅
- `QuerySet` 빌더와 `Model` 파생 매크로를 사용한 ORM
- Redis 및 Memory 백엔드를 지원하는 KV/Cache trait 추상화
- Memory/Redis 백엔드를 지원하는 Session 관리
- INI/YAML/ENV 지원 및 핫 리로드가 포함된 설정 관리
- Tera를 통한 템플릿 렌더링
- tracing 통합 로깅
- CLI 스캐폴딩 및 코드 생성
- 검색, 그래프, 시계열 엔진 trait 스텁 (드라이버는 계획 중)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
