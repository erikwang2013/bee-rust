<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API 参考

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

返回 [README](../README.md)。

本文档是 Beerust 框架的 API 参考：各模块的用法、代码示例与接口说明。

## Web 核心（bee_router）

### 定义控制器

```rust
use bee_rust::prelude::*;

// 定义控制器
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### 路由注册

```rust
// 路由注册
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context 提供：**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — 响应输出
- `ctx.redirect()` — 重定向
- `ctx.abort()` — 中断请求
- `ctx.session` — 会话访问
- `ctx.params` — 路径参数

## 安全检测（`security` feature）

基于 [security-rust](https://crates.io/crates/security-rust) 的攻击检测过滤器，覆盖 XSS、SQL 注入、命令注入、SSRF 等 27 种攻击类型：

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // 27 个检测器全开
```

在 `Cargo.toml` 中启用：
```toml
bee_rust = { features = ["security"] }
```

## ORM（bee_orm）

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
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql 接口同形

    // 建表——migrate::sync 按模型元数据生成非破坏性 DDL
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // 插入——auto 主键由数据库分配
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // 查询——链式过滤 + 执行：all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // 关系——children 取 has_many，belongs_to 取父行；FK 强制下先删子行再删父行
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // 更新 / 删除——delete 为软删除（翻转标记），with_deleted() / hard_delete() 可绕过
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

    // 事务——先 get() 取出连接；sqlite 的 CheckedConn 为同步接口
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> 迁移可用：`bee_orm::migrate`（`create_table` / `add_missing_columns` / `sync`）按方言生成 DDL，只建表与补列，绝不删改。关系读取覆盖 `belongs_to`、has_many（`children*`）与多对多（`m2m`）；`#[bee(fk = …)]` 生成 FK DDL：postgres 与 sqlite（bundled 构建）强制外键，mysql 可用表级外键 opt-in 开启（见下）。

### 多对多（`m2m`）

结构体级 `#[bee(m2m(Target))]` 声明；join 表随 `create_table` / `sync` 自动创建（先同步目标模型，再同步声明方）：

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // join 表 user_tag，列 user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // 目标模型先建
migrate::sync::<User, _>(&pool).await?;      // 声明方创建 join 表

// auto 主键由数据库分配：插入后查询取回实例，再读写关联
User { id: 0, name: "alice".into() }.insert(&pool).await?;
let user = User::query().filter_eq("name", "alice")?.one(&pool).await?.expect("inserted");
Tag { id: 0, name: "rust".into() }.insert(&pool).await?;
let tag = Tag::query().filter_eq("name", "rust")?.one(&pool).await?.expect("inserted");

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // 重复 attach → 复合主键错误
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // 幂等：再次调用返回 0 行
```

> 覆盖命名：`#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`；join 表只随 `create_table` / `sync` 创建，`add_missing_columns` 不触碰。

### JSON 列

`serde_json::Value` 字段映射为 sqlite `TEXT` / postgres `JSONB` / mysql `JSON`：

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // 必填
    extra: Option<serde_json::Value>,  // 可空
}
```

> NULL 语义：SQL `NULL` 解码为 `None`；存储过的 JSON `null` 文档解码为 `Some(Json::Null)`——二者不同。

### mysql 表级外键（opt-in）

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // 仅 mysql 生效
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> 开启后新建表与新增列携带表级 `FOREIGN KEY`（约束名 `{table}_{column}_fk`）；已有孤儿数据会让 `ADD CONSTRAINT` 报错（不静默跳过，也不 retrofit 既有列）；pg / sqlite 下该选项为无操作（inline 已强制）。

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls 使用内置 webpki 根；自定义 rustls 配置用 connect_tls_with：
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig（重导出，版本与 crate 一致）
```

### 日期与 Decimal 类型

日期与 Decimal 字段按 feature 门控：`bee_orm` 开 `chrono`（`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`）与 `rust_decimal`（`Decimal`）；经 `bee_rust` 使用时对应转发 feature `orm-chrono` / `orm-rust_decimal`。

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date；DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal（三后端一致）
}

// 写：字段值转成对应 Value 变体；读：单元格按 serde 形解码回字段类型
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // "1.50" 往返数值仍相等
```

> 支持面：`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`；范围外（宏不给 SQL 映射，需 `#[bee(sql_type = "…")]` 逃生舱）：`NaiveTime` / `DateTime<Local>` / `FixedOffset` / 裸 `DateTime`，以及纳秒精度。
> 存储口径：sqlite 一律 TEXT（`typeof(col)` 为 `text`；Decimal 也存文本——声明 DECIMAL 会得到 NUMERIC 亲和，把 "1.50" 吞成 REAL 1.5）；mysql `datetime(6)` / `timestamp(6)` 带微秒，TIMESTAMP 读回按 UTC（RFC3339 带 `Z`，严格往返要求会话时区为 UTC），DATETIME 保持 naive 无后缀，两者按列型分派；pg `timestamptz` 归一 UTC。
> 精度天花板：pg `numeric` 超 29 位有效精度读回 `NULL`（原生 FromSql 拒绝）；mysql 走文本，超精度到解码时才由用户 serde 报错；钱型建议 `#[bee(sql_type = Raw("decimal(12,2)"))]`。
> 编译期契约：feature 关闭时带日期字段的模型报 E0277（`Value: From<NaiveDate>` 不满足），不会静默落错列。

## 配置管理（bee_config）

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

## 存储引擎

**KV/Cache：**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// memcached 后端（feature `memcached`）
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> `RedisStore` 内部是 `ConnectionManager`：断线自动重连（按命令触发、指数退避 + 抖动，无后台健康检查线程）——撞上断线的那条命令会报错，下一条自动等待新连接。memcached：`incr` 先建计数器（缺键为 0）再增减；计数器无符号，减到 0 封底（Redis 可为负）；`expire(≤0)` 等价删除。

**缓存后端（bee_cache）：**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache`（feature `memcache`）接口相同；TTL `Some(0)` 在 Redis 后端走 `DEL`（`SET … EX 0` 会被拒绝），memcached 后端同样按删除处理（其 0 表示永不过期）；`incr` 对非数值报序列化错误。

**搜索引擎（已实现，opt-in）：**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // 需启用 feature `elasticsearch`
let engine = Elasticsearch::new("http://localhost:9200");   // 同步构造（无 `?`）
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> 其余驱动写法相同，均须按需启用：`opensearch`, `clickhouse`（feature 同名）。

**图数据库（已实现，opt-in）：**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // 需启用 feature `neo4j`
let db = Neo4j::new("http://localhost:7474");   // 同步构造（无 `?`）
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> 其余驱动写法相同，均须按需启用：`nebulagraph`, `arangodb`（feature 同名）。

**时序数据库（已实现，opt-in）：**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // 需启用 feature `influxdb`
let tsdb = InfluxDB::new("http://localhost:8086");   // 同步构造（无 `?`）
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> 其余驱动写法相同，均须按需启用：`iotdb`, `questdb`（feature 同名）。

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> 后端由实现了 `bee_cache::Cache` 的任意缓存提供：`MemoryCache` / `RedisCache` / `MemcacheCache`。

## 日志

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## 模板

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI 工具

```bash
# 创建项目（生成可运行的脚手架：Cargo.toml + src/main.rs）
bee-rust new my-app

# 生成代码
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# 开发运行（--watch 监听 src/ 变化自动重启）
bee-rust run
bee-rust run --watch

# 打包部署（cargo build --release + 复制到 dist/）
bee-rust pack

# 数据库迁移（在目标 crate 生成迁移入口并运行）
bee-rust migrate init    # 生成 src/bin/bee_migrate.rs（已存在则拒绝，不覆盖）
bee-rust migrate run     # cargo run --bin bee_migrate
```

> 说明：`pack --target` 参数为预留接口，当前打包流程不区分目标平台。
