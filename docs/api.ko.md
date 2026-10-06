<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API 참조

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

[README](README.ko.md)로 돌아가기.

이 문서는 Beerust 프레임워크의 API 참조입니다: 각 모듈의 사용법, 코드 예제와 인터페이스 설명.

## Web 핵심 (bee_router)

### 컨트롤러 정의

```rust
use bee_rust::prelude::*;

// 컨트롤러 정의
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### 라우팅 등록

```rust
// 라우팅 등록
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context 제공:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — 응답 출력
- `ctx.redirect()` — 리다이렉트
- `ctx.abort()` — 요청 중단
- `ctx.session` — 세션 접근
- `ctx.params` — 경로 파라미터

## 보안 감지 (`security` feature)

[security-rust](https://crates.io/crates/security-rust) 기반의 공격 감지 필터로, XSS, SQL 인젝션, 명령어 인젝션, SSRF 등 27가지 공격 유형을 다룹니다:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // 27개 탐지기 모두 켜짐
```

`Cargo.toml`에서 활성화:
```toml
bee_rust = { features = ["security"] }
```

## ORM (bee_orm)

```rust
use bee_orm::pool::sqlite::Pool;
use bee_orm::{Model, Value, migrate, rel};

#[derive(Model)]
#[bee(table = "users")]
struct User {
    #[bee(pk, auto)]
    id: i64,
    #[bee(column = "user_name")]
    name: String,
    age: Option<i32>,
    active: bool,
    #[bee(auto_now_add)]
    created_at: i64,
    #[bee(soft_delete)]
    deleted: bool,
    #[bee(ignore)]
    cache: Vec<u8>,
}

#[derive(Model)]
#[bee(table = "posts")]
struct Post {
    #[bee(pk, auto)]
    id: i64,
    #[bee(fk = User)]
    user_id: Option<i64>,
    title: String,
    #[bee(soft_delete)]
    deleted: bool,
}

async fn demo() -> Result<(), bee_orm::OrmError> {
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql은 같은 형태

    // 스키마 — migrate::sync가 모델 메타데이터로 비파괴 DDL을 생성
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — `auto` 기본 키는 데이터베이스가 할당
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — 체이닝 필터 + 실행: all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // 관계 — children은 has_many, belongs_to는 부모 행; FK 강제 시 자식을 먼저 삭제
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete는 소프트 삭제(플래그 반전); with_deleted() / hard_delete()로 우회
    Post::query().filter_eq("title", "hello")?.hard_delete(&pool).await?;
    if let Some(mut u) = author {
        u.age = Some(31);
        u.update(&pool).await?;
        u.delete(&pool).await?;
        let live = User::query().count(&pool).await?;
        let every = User::query().with_deleted().count(&pool).await?;
        u.hard_delete(&pool).await?;
    }
    User::query().filter_eq("user_name", "bob")?.update(&pool, &[("user_name", "bobby".into())]).await?;

    // 트랜잭션 — `get()`으로 커넥션을 보유; sqlite의 `CheckedConn`은 동기
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> 마이그레이션 사용 가능: `bee_orm::migrate`(`create_table` / `add_missing_columns` / `sync`)가 방언별 DDL을 생성합니다 — 테이블 생성과 누락 컬럼 추가만 하고, 삭제나 변경은 하지 않습니다. 관계 읽기는 `belongs_to`, has_many(`children*`), many-to-many(`m2m`)를 지원합니다. `#[bee(fk = …)]`는 외래 키 DDL을 생성합니다: postgres와 sqlite(bundled 빌드)는 강제하고, mysql은 테이블 수준 외래 키를 opt-in으로 켤 수 있습니다(아래).

### 다대다(`m2m`)

구조체 수준에서 `#[bee(m2m(Target))]`로 선언; 조인 테이블은 `create_table` / `sync`가 만듭니다(대상 모델 먼저, 그다음 선언 모델):

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // 조인 테이블 user_tag, 컬럼 user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // 대상 모델 먼저
migrate::sync::<User, _>(&pool).await?;      // 선언 모델이 조인 테이블 생성

// `auto` 키는 데이터베이스가 할당: 관계 조작 전에 인스턴스를 다시 읽으세요
User { id: 0, name: "alice".into() }.insert(&pool).await?;
let user = User::query().filter_eq("name", "alice")?.one(&pool).await?.expect("inserted");
Tag { id: 0, name: "rust".into() }.insert(&pool).await?;
let tag = Tag::query().filter_eq("name", "rust")?.one(&pool).await?.expect("inserted");

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // 중복 attach -> 복합 PK 오류
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // 멱등: 두 번째 호출은 0행
```

> 이름 재정의: `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`; 조인 테이블은 `create_table` / `sync`로만 생성되며 `add_missing_columns`는 건드리지 않습니다.

### JSON 컬럼

`serde_json::Value` 필드는 sqlite `TEXT` / postgres `JSONB` / mysql `JSON`에 매핑됩니다:

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // 필수
    extra: Option<serde_json::Value>,  // nullable
}
```

> NULL 의미론: SQL `NULL`은 `None`으로, 저장된 JSON `null` 문서는 `Some(Json::Null)`로 디코딩됩니다 — 둘은 다릅니다.

### MySQL 테이블 수준 외래 키(opt-in)

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // mysql 전용
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> 켜면 새 테이블과 추가된 컬럼에 테이블 수준 `FOREIGN KEY`가 붙습니다(제약 이름 `{table}_{column}_fk`); 기존의 고아 행이 있으면 `ADD CONSTRAINT`가 실패합니다(조용한 건너뜀 없음, 기존 컬럼 retrofit 없음); pg / sqlite에서는 이 옵션이 무효입니다(inline이 이미 강제).

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls는 내장 webpki 루트 사용; 사용자 정의 rustls 설정은 connect_tls_with:
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig(재수출, crate와 같은 버전)
```

### 날짜 및 Decimal 타입

날짜와 Decimal 필드는 feature로 제어합니다: `bee_orm`에서 `chrono`(`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`)와 `rust_decimal`(`Decimal`)을 켜고, `bee_rust` 경유 시 전달 feature는 `orm-chrono` / `orm-rust_decimal`입니다.

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date, DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal (세 백엔드 동일)
}

// 쓰기: 필드가 대응 Value 변형으로; 읽기: 셀이 serde로 복원
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // "1.50"은 같은 값으로 왕복
```

> 지원: `NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`. 범위 밖 — 매크로가 SQL 매핑을 내지 않으므로 `#[bee(sql_type = "…")]` 탈출구를 쓰세요: `NaiveTime` / `DateTime<Local>` / `FixedOffset` / 순수 `DateTime`, 나노초 정밀도.
> 저장: sqlite는 항상 TEXT (`typeof(col)`는 `text`; Decimal도 동일 — DECIMAL 선언은 NUMERIC 친화도를 얻어 "1.50"을 REAL 1.5로 삼킵니다); mysql은 `datetime(6)` / `timestamp(6)` 마이크로초 — TIMESTAMP는 UTC로 읽힘(RFC3339 `Z`; 엄격한 왕복은 세션 타임존 UTC 전제), DATETIME은 naive로 접미사 없음, 열 타입으로 분기; pg `timestamptz`는 UTC로 정규화.
> 정밀도 상한: pg `numeric`은 유효 자릿수 29를 넘으면 `NULL`로 읽힘(네이티브 FromSql 거부); mysql은 텍스트 경로라 초과분은 디코드 시 serde에서 오류; 금액에는 `#[bee(sql_type = Raw("decimal(12,2)"))]` 권장.
> 컴파일 계약: feature가 꺼져 있으면 날짜 필드 모델은 E0277(`Value: From<NaiveDate>` 불충족)로 실패 — 조용히 잘못된 열이 되지 않습니다.

## 설정 관리 (bee_config)

```rust
#[derive(Config)]
#[config(file = "conf/app.conf")]
struct AppConfig {
    app_name: String,
    http_port: u16,
    run_mode: String,
}

let cfg = AppConfig::load("conf/app.conf")?;
```

## 스토리지 엔진

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// memcached 백엔드(feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> `RedisStore` 내부는 `ConnectionManager`입니다: 끊긴 연결은 자동으로 재연결됩니다(명령 시 트리거 — 백그라운드 스레드 없음 — 지수 백오프와 지터) — 끊김에 걸린 명령은 오류가 되고, 다음 명령은 새 연결을 기다립니다. memcached: `incr`은 먼저 카운터를 만들고(없으면 0) 그다음 증감합니다; 카운터는 부호 없음이라 감소는 0에서 멈춥니다(Redis는 음수로 내려갑니다); `expire(≤0)`은 삭제와 같습니다.

**캐시 백엔드(bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache`(feature `memcache`)는 같은 인터페이스; TTL `Some(0)`은 Redis 백엔드에서 `DEL`로 처리되고(`SET … EX 0`은 거부됨), memcached에서도 삭제로 취급됩니다(거기서 0은 만료 없음을 뜻합니다); 비숫자 값에 대한 `incr`은 직렬화 오류입니다.

**검색 엔진 (구현 완료, opt-in):**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // feature `elasticsearch` 필요
let engine = Elasticsearch::new("http://localhost:9200");   // 동기 생성자 (`?` 불필요)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> 나머지 드라이버도 사용법이 같고 모두 opt-in입니다: `opensearch`, `clickhouse`(동일한 이름의 feature).

**그래프 데이터베이스 (구현 완료, opt-in):**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // feature `neo4j` 필요
let db = Neo4j::new("http://localhost:7474");   // 동기 생성자 (`?` 불필요)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> 나머지 드라이버도 사용법이 같고 모두 opt-in입니다: `nebulagraph`, `arangodb`(동일한 이름의 feature).

**시계열 데이터베이스 (구현 완료, opt-in):**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // feature `influxdb` 필요
let tsdb = InfluxDB::new("http://localhost:8086");   // 동기 생성자 (`?` 불필요)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> 나머지 드라이버도 사용법이 같고 모두 opt-in입니다: `iotdb`, `questdb`(동일한 이름의 feature).

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> 백엔드는 `bee_cache::Cache`를 구현한 임의의 캐시입니다: `MemoryCache` / `RedisCache` / `MemcacheCache`.

## 로그

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## 템플릿

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI 도구

```bash
# 프로젝트 생성 (실행 가능한 스캐폴딩 생성: Cargo.toml + src/main.rs)
bee-rust new my-app

# 코드 생성
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# 개발 실행 (--watch는 src/ 변경을 감지해 자동 재시작)
bee-rust run
bee-rust run --watch

# 패키징 배포 (cargo build --release + dist/로 복사)
bee-rust pack

# 데이터베이스 마이그레이션 (대상 crate에 진입점을 생성하고 실행)
bee-rust migrate init    # src/bin/bee_migrate.rs 생성 (기존 파일은 덮어쓰지 않음)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> 참고: `pack --target` 파라미터는 예약된 인터페이스이며, 현재 패키징 프로세스는 대상 플랫폼을 구분하지 않습니다.
