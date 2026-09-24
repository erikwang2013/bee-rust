# 변경 로그 (Changelog)

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
