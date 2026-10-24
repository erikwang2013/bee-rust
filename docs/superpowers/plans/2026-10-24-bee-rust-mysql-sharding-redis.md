# bee_orm MySQL 8/8.4/9 兼容 + 分库分表 + Redis 连接池/分布式锁 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `bee_orm` 在 MySQL 8.0 / 8.4 / 9.x 上通过真库测试，增加“必须带分片键”的分库分表路由，并把 `bee_kv` / `bee_cache` 的 Redis 后端换成真正的连接池，另加一个基于该池的 Redis 分布式锁。

**Architecture:** MySQL 兼容以 CI 三版本矩阵验证为主，不改 SQL 方言；分库分表通过 `Db::resolve_table` 在执行时把逻辑表名换成物理表名，`ShardedPools::for_key` 是拿到可执行句柄的唯一入口，从类型上强制分片键；Redis 用 `deadpool-redis 0.18`（对应现有 redis 0.27）替换 `ConnectionManager`，锁用 `SET NX PX` + Lua 比较删除。

**Tech Stack:** Rust 2024、`mysql_async 0.34`、`deadpool-redis 0.18`、`redis 0.27`、sqlite `r2d2`、GitHub Actions。

**Spec:** 本文件 §背景与既定决策（2026-10-24 会话确认）。

## 背景与既定决策

- MySQL 支持 8.0、8.4、9.x；5.7 不在范围。兼容的定义是“现有集成测试在三个真库版本上全绿”，不新增版本专属 SQL。
- 分库分表：查询必须带分片键；不提供无键的跨分片查询、跨分片排序/分页/聚合、跨分片事务、跨分片 JOIN/FK。
- 分片算法：内置 FNV-1a 64 位哈希取模；库和表各自有 shard 数，混合分片用同一个复合序号 `hash % (db_shards * table_shards)` 保证所有库×表组合可达；表名后缀固定 `_N`（`N` 从 0 开始）。范围/日期路由由用户实现 `ShardRouter`。
- Redis：`bee_kv` / `bee_cache` 的现有 Redis 客户端支持真正的连接池；锁与连接池是两个能力，锁复用同一个池。
- 锁的语义：`try_lock(key, ttl)` 非阻塞获取；`LockGuard::release()` 显式释放；不做自动续期、可重入、阻塞排队；TTL 是兜底，Drop 不自动释放。

## Global Constraints

- 工作区 edition 2024，版本保持 `1.3.0`；本计划不 bump 版本号。
- 只新增一个依赖：`deadpool-redis = "0.18"`（它依赖 redis 0.27，与仓库现有版本一致）；不升级 redis。
- 标识符不引号（仓库既有约定）：物理表名必须匹配 `[A-Za-z_][A-Za-z0-9_]*`，分片层必须拒绝其它字符。
- 分片键只接受 `Value::{Int, Text, Bytes, Bool}`；其它类型报 `OrmError::QueryError`，不 panic、不静默取模。
- 每个任务都先写失败测试；提交前跑 `cargo fmt --all --check` 和 `cargo clippy --workspace --all-targets -- -D warnings`。
- 单文件保持 500 行以内；新模块优先放在已有文件/模块边界内。
- 提交信息用仓库现有的 conventional prefix；不要加 `Co-Authored-By`。

## Review Focus

1. **不支持的分片键类型**（`Null` / `Float` / `Json` / 日期 / Decimal）：必须返回清晰错误，不能 panic 或落到错误分片。测试在 Task 6、Task 7。
2. **自定义 router 返回带 SQL 元字符的物理表名**：必须在拼 SQL 前拒绝，不能拼接。测试在 Task 6。
3. **跨分片隔离**：用 key A 的句柄读写，绝不能碰到 key B 的表。测试在 Task 7、Task 8。
4. **Redis 池在并发/连接被杀/池耗尽时**：不能死锁、不能无限等待；连接被杀后下一次操作要恢复。测试在 Task 9、Task 10。
5. **锁的所有权与 TTL**：释放不能删掉别人的锁；TTL 过期后必须能重新获取。测试在 Task 11。
6. **MySQL 8.4/9 的 `caching_sha2_password`**：不额外加连接参数也能通过真库测试。测试在 Task 1 的 CI 矩阵。

---

### Task 1: MySQL 8.0 / 8.4 / 9 CI 矩阵

**Files:**
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: 现有 `crates/bee_orm/tests/integration_mysql.rs`、`integration_mysql_fk.rs`（都由 `BEE_ORM_MYSQL_DSN` 门控）。
- Produces: 一个 `integration-mysql` job，三个 MySQL 版本各自跑 `cargo test -p bee_orm --features mysql`。

- [ ] **Step 1: 从现有 integration job 移除 MySQL**

删除 `integration` job 里的 `mysql:` service 和 `BEE_ORM_MYSQL_DSN` 环境变量，并把 job 名从 `Integration (postgres + mysql + redis + memcached)` 改成 `Integration (postgres + redis + memcached)`。保留 postgres、redis、memcached 和后面的 pg TLS 步骤。

- [ ] **Step 2: 增加 MySQL 版本矩阵 job**

在 `integration` job 之后加入：

```yaml
  integration-mysql:
    name: Integration (mysql ${{ matrix.mysql }})
    runs-on: ubuntu-latest
    timeout-minutes: 15
    strategy:
      fail-fast: false
      matrix:
        mysql: ["8.0", "8.4", "9"]
    services:
      mysql:
        image: mysql:${{ matrix.mysql }}
        env:
          MYSQL_ROOT_PASSWORD: root
          MYSQL_DATABASE: bee_test
        ports:
          - "3306:3306"
        options: >-
          --health-cmd "mysqladmin ping -h 127.0.0.1 -proot"
          --health-interval 10s --health-timeout 5s --health-retries 10
          --health-start-period 30s
    env:
      BEE_ORM_MYSQL_DSN: mysql://root:root@127.0.0.1:3306/bee_test
    steps:
      - uses: actions/checkout@f548e57e544e1ff5a4c46bf1e1b8685f8e4a348a

      - uses: dtolnay/rust-toolchain@6c977a6ca4077a0ceb28ffbe03f59d46e9ac8772
        with:
          toolchain: stable

      - name: MySQL integration tests (DSN-gated)
        run: cargo test -p bee_orm --features mysql
```

- [ ] **Step 3: 校验 workflow**

Run: `actionlint .github/workflows/ci.yml`
Expected: exit 0，无输出。

- [ ] **Step 4: 本地冒烟（有 Docker 时）**

```bash
docker run -d --rm --name bee-mysql84 \
  -e MYSQL_ROOT_PASSWORD=root -e MYSQL_DATABASE=bee_test \
  -p 3308:3306 mysql:8.4
# 等 healthcheck 通过
BEE_ORM_MYSQL_DSN=mysql://root:root@127.0.0.1:3308/bee_test \
  cargo test -p bee_orm --features mysql
docker stop bee-mysql84
```

Expected: 测试全绿；如果 8.4 或 9 失败，先在本任务内定位（预期只可能是认证或 DDL 差异），不要带着红色进入 Task 2。

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: test bee_orm against MySQL 8.0, 8.4, and 9"
```

---

### Task 2: `Db::resolve_table` 与 Model 执行路径

**Files:**
- Modify: `crates/bee_orm/src/db.rs`
- Modify: `crates/bee_orm/src/model.rs`
- Test: `crates/bee_orm/src/tests/mod.rs`, `crates/bee_orm/src/tests/model_ops.rs`

**Interfaces:**
- Produces: `Db::resolve_table(&self, logical: &str) -> Result<String>`，默认返回 `logical.to_owned()`。
- Consumes: 无。
- 后续任务依赖这个签名；不要用 `Cow`（SQL 构建本来就会分配，`String` 避免给 trait 加生命周期）。

- [ ] **Step 1: 给测试 Mock 加前缀解析**

在 `crates/bee_orm/src/tests/mod.rs` 的 `Mock` 增加字段 `resolve_prefix: Option<String>`，并加：

```rust
fn renaming(prefix: &str) -> Self {
    Self { resolve_prefix: Some(prefix.to_string()), ..Self::default() }
}

fn with_dialect_and_prefix(dialect: Dialect, prefix: &str) -> Self {
    Self {
        dialect: Some(dialect),
        resolve_prefix: Some(prefix.to_string()),
        ..Self::default()
    }
}
```

在 `impl Db for Mock` 里加：

```rust
fn resolve_table(&self, logical: &str) -> Result<String> {
    Ok(match &self.resolve_prefix {
        Some(prefix) => format!("{prefix}{logical}"),
        None => logical.to_owned(),
    })
}
```

- [ ] **Step 2: 写失败测试**

在 `src/tests/model_ops.rs` 加：

```rust
#[tokio::test]
async fn insert_uses_the_resolved_table() {
    let db = Mock::renaming("s0_");
    Tiny { id: 1, v: 5 }.insert(&db).await.unwrap();
    let (sql, _) = db.calls().remove(0);
    assert_eq!(sql, "INSERT INTO s0_tiny_items (v) VALUES (?)");
}

#[tokio::test]
async fn update_and_deletes_use_the_resolved_table() {
    let db = Mock::renaming("s0_");
    Tiny { id: 7, v: 5 }.update(&db).await.unwrap();
    Tiny { id: 7, v: 5 }.hard_delete(&db).await.unwrap();
    let calls = db.calls();
    assert_eq!(calls[0].0, "UPDATE s0_tiny_items SET v = ? WHERE id = ?");
    assert_eq!(calls[1].0, "DELETE FROM s0_tiny_items WHERE id = ?");
}
```

- [ ] **Step 3: 运行测试确认失败**

Run: `cargo test -p bee_orm --lib insert_uses_the_resolved_table`
Expected: FAIL，实际 SQL 以 `INSERT INTO tiny_items` 开头。

- [ ] **Step 4: 实现 trait 默认方法**

在 `crates/bee_orm/src/db.rs` 的 `Db` trait 里加：

```rust
/// 逻辑表名对应的物理表名；默认原样返回。
///
/// 分片连接覆盖它来加后缀。所有 SQL 在格式化前都经过这里；实现方
/// 必须返回匹配 `[A-Za-z_][A-Za-z0-9_]*` 的名字（SQL 不做标识符引号）。
fn resolve_table(&self, logical: &str) -> Result<String> {
    Ok(logical.to_owned())
}
```

- [ ] **Step 5: 让 Model 的写路径经过 resolve_table**

在 `crates/bee_orm/src/model.rs`：

- `insert_statement` 改成：

```rust
fn insert_statement<M: Model + ?Sized>(
    model: &M,
    table: &str,
    now: i64,
    op: &str,
) -> Result<(String, Vec<(&'static str, Value)>)>
```

并在 `format!` 里用 `table` 而不是 `M::table_name()`。

- `insert` / `create` 开头加 `let table = db.resolve_table(Self::table_name())?;`，调用 `insert_statement(self, &table, ...)`。
- `create` 的 `insert_returning(Self::table_name(), ...)` 改成 `insert_returning(&table, ...)`；兜底 `SELECT * FROM {table}`。
- `update` / `delete` / `hard_delete` 开头解析 `table`，`format!` 用 `table`。
- `insert_many` 开头解析一次 `table`，循环里的 `INSERT INTO {table}` 用它；错误信息仍可用 `Self::table_name()`（逻辑名更适合报错）。

- [ ] **Step 6: 运行测试确认通过**

Run: `cargo test -p bee_orm --lib`
Expected: PASS，既有 SQL 形状测试不变（默认解析是恒等）。

- [ ] **Step 7: Commit**

```bash
git add crates/bee_orm/src/db.rs crates/bee_orm/src/model.rs \
  crates/bee_orm/src/tests/mod.rs crates/bee_orm/src/tests/model_ops.rs
git commit -m "feat(bee_orm): resolve physical table names in model writes"
```

---

### Task 3: QuerySet 执行路径

**Files:**
- Modify: `crates/bee_orm/src/queryset.rs`
- Test: `crates/bee_orm/src/tests/queryset.rs`

**Interfaces:**
- Consumes: `Db::resolve_table`（Task 2）。
- Produces: `QuerySet` 的公开 API 不变；`to_sql()` 仍输出逻辑表名。

- [ ] **Step 1: 写失败测试**

```rust
#[tokio::test]
async fn reads_use_the_resolved_table() {
    let db = Mock::renaming("s0_");
    QuerySet::<Plain>::new("plain").filter_eq("id", 1).unwrap().all(&db).await.unwrap();
    assert_eq!(db.calls()[0].0, "SELECT * FROM s0_plain WHERE id = ?");
}

#[tokio::test]
async fn writes_and_aggregates_use_the_resolved_table() {
    let db = Mock {
        rows: vec![value_row(serde_json::json!(1.0))],
        resolve_prefix: Some("s0_".into()),
        ..Mock::default()
    };
    QuerySet::<Plain>::new("plain")
        .filter_eq("id", 1)
        .unwrap()
        .update(&db, &[("id", Value::Int(2))])
        .await
        .unwrap();
    QuerySet::<Plain>::new("plain").sum(&db, "id").await.unwrap();
    let calls = db.calls();
    assert_eq!(calls[0].0, "UPDATE s0_plain SET id = ? WHERE id = ?");
    assert_eq!(calls[1].0, "SELECT SUM(id) AS value FROM s0_plain");
}

#[test]
fn to_sql_stays_logical() {
    let qs = QuerySet::<Plain>::new("plain").filter_eq("id", 1).unwrap();
    assert_eq!(qs.to_sql(), "SELECT * FROM plain WHERE id = ?");
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p bee_orm --lib reads_use_the_resolved_table`
Expected: FAIL，SQL 仍是 `FROM plain`。

- [ ] **Step 3: 让 QuerySet 的每个执行路径解析表名**

在 `queryset.rs` 加：

```rust
fn table_for<D: Db + ?Sized>(&self, db: &D) -> Result<String> {
    db.resolve_table(&self.table)
}
```

- `select_sql` 改为 `select_sql_for(&self, table: &str, limit_override: Option<usize>) -> String`；`to_sql()` 调 `self.select_sql_for(&self.table, None)`。
- `rows` / `one`：先 `let table = self.table_for(db)?;`，再调 `select_sql_for(&table, ...)`。
- `count` / `exists`：解析 `table`，用 `format!("... FROM {table} ...")`。
- `update` / `delete` / `hard_delete`：解析 `table`，替换 `self.table`。
- `aggregate_f64` / `aggregate_raw`：解析 `table`；`aggregate_sql` 增加 `table: &str` 参数。

注意：所有直接使用 `self.table` 拼 SQL 的地方都要替换；`to_sql()` 是唯一保留逻辑名的地方。

- [ ] **Step 4: 运行测试**

Run: `cargo test -p bee_orm --lib`
Expected: PASS，既有 queryset 测试不改。

- [ ] **Step 5: Commit**

```bash
git add crates/bee_orm/src/queryset.rs crates/bee_orm/src/tests/queryset.rs
git commit -m "feat(bee_orm): resolve physical table names in QuerySet execution"
```

---

### Task 4: migrate 表名解析

**Files:**
- Modify: `crates/bee_orm/src/migrate.rs`
- Test: `crates/bee_orm/src/tests/migrate.rs`

**Interfaces:**
- Consumes: `Db::resolve_table`（Task 2）。
- Produces: `migrate` 的公开函数签名不变；内部按解析后的物理表名建表/加列/建约束。

- [ ] **Step 1: 写失败测试**

```rust
#[tokio::test]
async fn create_table_uses_the_resolved_names() {
    let db = Mock::with_dialect_and_prefix(Dialect::Sqlite, "s0_");
    migrate::create_table::<Article, _>(&db).await.unwrap();
    let ddl = &db.calls()[0].0;
    assert!(ddl.starts_with("CREATE TABLE IF NOT EXISTS s0_articles"), "{ddl}");
    assert!(ddl.contains("REFERENCES s0_authors (id)"), "{ddl}");
}

#[tokio::test]
async fn add_missing_columns_uses_the_resolved_table() {
    let db = Mock {
        dialect: Some(Dialect::Sqlite),
        resolve_prefix: Some("s0_".into()),
        rows: vec![row_of(&[("name", serde_json::json!("id"))])],
        ..Mock::default()
    };
    migrate::add_missing_columns::<Article, _>(&db).await.unwrap();
    let calls = db.calls();
    assert_eq!(calls[0].0, "PRAGMA table_info(s0_articles)");
    assert!(calls[1].0.starts_with("ALTER TABLE s0_articles ADD COLUMN"));
}
```

再在 `src/tests/m2m.rs` 或 `migrate.rs` 加一个 join 表断言：`Reader` 的 DDL 应出现 `CREATE TABLE IF NOT EXISTS s0_reader_author`，且两侧 `REFERENCES s0_readers (id)` / `REFERENCES s0_authors (id)`。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p bee_orm --lib create_table_uses_the_resolved_names`
Expected: FAIL，DDL 仍是 `articles` / `authors`。

- [ ] **Step 3: 改造 migrate 内部签名**

- `create_table_with`：`let table = db.resolve_table(M::table_name())?;`；每个 m2m 关系再解析 `let join_table = db.resolve_table(def.table)?;` 和 `let target_table = db.resolve_table((def.target_table)())?;`。
- `create_table_sql<M, D>(db: &D, dialect, table: &str, options)`：用 `table` 替换 `M::table_name()`。**外键目标也要解析**：对每个带 `references` 的列，先 `let target = db.resolve_table((reference.table)())?;`，再把 `REFERENCES {target} ({reference.pk_column})` 拼上去；表级 FK 的 `fk_clause` 也用这个 `target`。不要在 `column_sql` 里保留未解析的 `(reference.table)()` 路径。
- `join_table_sql<M>(def, dialect, options, join_table: &str, local_table: &str, target_table: &str)`：用三个物理名；`CREATE TABLE` 用 `join_table`，两侧 `REFERENCES` 用 `local_table` / `target_table`。
- `add_missing_columns_with`：解析 `table`；每个缺失列的 `references` 目标同样用 `db.resolve_table((reference.table)())?` 解析后拼 `REFERENCES`；`existing_columns(db, dialect, &table)`；`ALTER TABLE {table}`；`add_constraint(db, &table, column)`。
- `existing_columns<D>(db, dialect, table: &str)`：删掉内部的 `M::table_name()`。
- `add_constraint<D>(db, table: &str, column)`：约束名和 `ALTER TABLE` 都用传入的物理名；约束里的 `REFERENCES` 目标用 `db.resolve_table((reference.table)())?`。
- `constraint_exists` 已经接收 `table`，不用改签名。

- [ ] **Step 4: 运行测试**

Run: `cargo test -p bee_orm --lib`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/bee_orm/src/migrate.rs crates/bee_orm/src/tests/migrate.rs \
  crates/bee_orm/src/tests/m2m.rs
git commit -m "feat(bee_orm): resolve physical table names in migrations"
```

---

### Task 5: m2m 直接 SQL 的表名解析

**Files:**
- Modify: `crates/bee_orm/src/m2m.rs`
- Test: `crates/bee_orm/src/tests/m2m.rs`

**Interfaces:**
- Consumes: `Db::resolve_table`（Task 2）。
- Produces: `m2m` 的公开签名不变；直接拼 SQL 的四个函数使用物理 join 表名。

- [ ] **Step 1: 写失败测试**

```rust
#[tokio::test]
async fn m2m_direct_sql_uses_the_resolved_join_table() {
    let mock = Mock::renaming("s0_");
    m2m::attach(&mock, &Reader { id: 3 }, &Author { id: 7 }).await.unwrap();
    m2m::detach(&mock, &Reader { id: 3 }, &Author { id: 7 }).await.unwrap();
    let calls = mock.calls();
    assert_eq!(calls[0].0, "INSERT INTO s0_reader_author (reader_id, author_id) VALUES (?, ?)");
    assert_eq!(calls[1].0, "DELETE FROM s0_reader_author WHERE reader_id = ? AND author_id = ?");
}

#[tokio::test]
async fn related_ids_uses_the_resolved_join_table() {
    let mock = Mock { rows: vec![row_of(&[("author_id", json!(7))])], resolve_prefix: Some("s0_".into()), ..Mock::default() };
    m2m::related_ids::<Reader, Author, _>(&mock, &Reader { id: 3 }).await.unwrap();
    assert_eq!(mock.calls()[0].0, "SELECT author_id FROM s0_reader_author WHERE reader_id = ?");
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p bee_orm --lib m2m_direct_sql_uses_the_resolved_join_table`
Expected: FAIL，SQL 仍是 `reader_author`。

- [ ] **Step 3: 实现**

在 `related_ids`、`related_for`（第 1 步的 join 查询）、`attach`、`detach` 里：`let table = db.resolve_table(def.table)?;`，再用 `table` 拼 SQL。`related` / `related_for` 第 2 步走 `QuerySet`，由 Task 3 自动解析，不需要改。

- [ ] **Step 4: 运行测试**

Run: `cargo test -p bee_orm --lib`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/bee_orm/src/m2m.rs crates/bee_orm/src/tests/m2m.rs
git commit -m "feat(bee_orm): resolve physical table names in m2m SQL"
```

---

### Task 6: shard 模块（router + pools + handle）

**Files:**
- Create: `crates/bee_orm/src/shard.rs`
- Modify: `crates/bee_orm/src/lib.rs`
- Test: `crates/bee_orm/src/tests/shard.rs`, `crates/bee_orm/src/tests/mod.rs`

**Interfaces:**
- Consumes: `Db`, `Value`, `OrmError`, `Model`, `MigrateOptions`, `migrate::sync_with`。
- Produces:

```rust
pub trait ShardRouter: Send + Sync {
    fn db_shards(&self) -> usize;
    fn table_shards(&self) -> usize;
    fn db_index(&self, key: &Value) -> Result<usize>;
    fn table_name(&self, logical: &str, key: &Value) -> Result<String>;
    fn table_name_at(&self, logical: &str, index: usize) -> Result<String>;
}

pub struct HashRouter { /* db_shards: usize, table_shards: usize */ }
impl HashRouter {
    pub fn new(db_shards: usize, table_shards: usize) -> Result<Self>;
}

pub struct ShardedPools<P> { /* pools: Vec<P>, router: Arc<dyn ShardRouter> */ }
impl<P> ShardedPools<P> {
    pub fn new(pools: Vec<P>, router: Arc<dyn ShardRouter>) -> Result<Self>;
    pub fn for_key(&self, key: impl Into<Value>) -> Result<ShardHandle<'_, P>>;
}
impl<P: Db> ShardedPools<P> {
    pub async fn sync_all<M: Model>(&self, options: MigrateOptions) -> Result<u64>;
}

pub struct ShardHandle<'a, P> { /* pool: &'a P, router: &'a dyn ShardRouter, key: Value */ }
impl<'a, P> ShardHandle<'a, P> {
    pub fn pool(&self) -> &'a P;
}
impl<P: Db> Db for ShardHandle<'_, P> { /* query / execute / dialect / resolve_table / insert_returning */ }
```

`table_name_at(logical, index)` 是 `sync_all` 枚举物理表的入口，必须与 `table_name(logical, key)` 对每个 ordinal 一致：`table_name` 算出的 index 调 `table_name_at` 必须得到同一个名字。自定义 router 的范围/日期路由也要满足这一点。

- [ ] **Step 1: 写失败测试**

`src/tests/mod.rs` 加 `mod shard;`，新建 `src/tests/shard.rs`：

```rust
use super::*;
use crate::shard::{HashRouter, ShardRouter, ShardedPools};
use std::sync::Arc;

#[test]
fn fnv1a_matches_known_vectors() {
    assert_eq!(key_hash(&Value::Text("a".into())).unwrap(), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(key_hash(&Value::Text("foobar".into())).unwrap(), 0x8594_4171_f739_67e8);
}

#[test]
fn hash_router_routes_known_keys() {
    let router = HashRouter::new(2, 4).unwrap();
    // Composite index: hash % 8, db = index / 4, table = index % 4.
    assert_eq!(router.db_index(&Value::Int(1)).unwrap(), 1);
    assert_eq!(router.table_name("users", &Value::Int(1)).unwrap(), "users_0");
    assert_eq!(router.db_index(&Value::Int(2)).unwrap(), 1);
    assert_eq!(router.table_name("users", &Value::Int(2)).unwrap(), "users_3");
}

#[test]
fn mixed_shard_counts_reach_every_database_table_pair() {
    let router = HashRouter::new(2, 4).unwrap();
    let mut seen = std::collections::HashSet::new();
    for key in 0..256 {
        let key = Value::Int(key);
        seen.insert((router.db_index(&key).unwrap(), router.table_name("users", &key).unwrap()));
    }
    assert_eq!(seen.len(), 8, "all 2×4 database/table pairs must be reachable");
}

#[test]
fn one_table_shard_keeps_the_logical_name() {
    let router = HashRouter::new(2, 1).unwrap();
    assert_eq!(router.table_name("users", &Value::Int(2)).unwrap(), "users");
}

#[test]
fn unsupported_keys_are_rejected() {
    for key in [Value::Null, Value::Float(1.0), Value::Json(serde_json::json!({}))] {
        assert!(key_hash(&key).is_err(), "{key:?}");
    }
}

#[test]
fn zero_shards_are_rejected() {
    assert!(HashRouter::new(0, 1).is_err());
    assert!(HashRouter::new(1, 0).is_err());
}

#[tokio::test]
async fn for_key_selects_the_router_database() {
    let a = Mock::default();
    let b = Mock::default();
    let (a_probe, b_probe) = (a.clone(), b.clone());
    let shards = ShardedPools::new(vec![a, b], Arc::new(HashRouter::new(2, 1).unwrap())).unwrap();

    shards.for_key(Value::Int(1)).unwrap().execute("SELECT 1", &[]).await.unwrap();
    assert_eq!(a_probe.calls().len(), 1);
    assert_eq!(b_probe.calls().len(), 0);

    shards.for_key(Value::Int(2)).unwrap().execute("SELECT 1", &[]).await.unwrap();
    assert_eq!(b_probe.calls().len(), 1);
}
```

为了让这个测试能同时探测两个 pool，先在 `src/tests/mod.rs` 把 `Mock.calls` 的类型改成 `Arc<Mutex<Vec<(String, Vec<Value>)>>>`，并给 `Mock` 加 `#[derive(Clone, Default)]`；`calls()` 与 `impl Db` 里的 `.lock()` 写法不变。只改测试夹具，不改任何断言。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p bee_orm --lib fnv1a_matches_known_vectors`
Expected: FAIL（`key_hash` 不存在 / `shard` 模块不存在）。

- [ ] **Step 3: 实现 `shard.rs`**

要点：

- FNV-1a 64：

```rust
pub(crate) fn key_hash(key: &Value) -> Result<u64> {
    let bytes: Vec<u8> = match key {
        Value::Int(value) => value.to_le_bytes().to_vec(),
        Value::Text(value) => value.as_bytes().to_vec(),
        Value::Bytes(value) => value.clone(),
        Value::Bool(value) => vec![u8::from(*value)],
        _ => return Err(OrmError::QueryError(
            "sharding: shard key must be Int, Text, Bytes, or Bool".into(),
        )),
    };
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    Ok(hash)
}
```

- `HashRouter::new`：`db_shards == 0 || table_shards == 0` 返回 `OrmError::QueryError("sharding: shard counts must be at least 1")`。
- 复合分片序号（私有）：`shard = key_hash(key)? % (db_shards * table_shards) as u64`，用 u64 运算。它保证混合分片时所有库×表组合都可达；只用单一 `hash % db` / `hash % table` 时，2×4 只有 4 个组合可达。
- `db_index`：`(shard / table_shards as u64) as usize`。
- `table_name`：`table_index = (shard % table_shards as u64) as usize; self.table_name_at(logical, table_index)`。
- `table_name_at`：`index >= table_shards` 返回错误；`table_shards == 1` 返回 `logical.to_owned()`；否则 `format!("{logical}_{index}")`。
- `validate_table_name`：同 `QuerySet::validate_field` 的字符集；失败返回 `OrmError::QueryError(format!("sharding: invalid physical table name `{name}`"))`。
- `ShardedPools::new`：pools 非空；`router.db_shards() == pools.len()`；否则报错。
- `for_key`：`let key = key.into(); let index = self.router.db_index(&key)?; if index >= self.pools.len() { 报错 }`；返回 `ShardHandle`.
- `ShardHandle` 的 `Db` 实现：`query` / `execute` / `dialect` 转发；`resolve_table` 调 `router.table_name` 后 `validate_table_name`；`insert_returning` **必须显式转发** `self.pool.insert_returning(...)`，否则 MySQL 的回读优化会丢失。
- `sync_all`：对每个 pool、每个 `table_index in 0..router.table_shards()` 构造一个内部 `ShardTarget`（实现 `Db`：转发 query/execute/dialect，`resolve_table` 用 `table_name_at` + 校验），调用 `migrate::sync_with::<M, _>(&target, options).await?`，累加返回值。

- [ ] **Step 4: 补齐测试并运行**

再加以下测试：

```rust
#[test]
fn sharded_pools_rejects_pool_count_mismatch() {
    let result = ShardedPools::new(vec![Mock::default()], Arc::new(HashRouter::new(2, 1).unwrap()));
    assert!(result.is_err());
}

struct BadRouter;
impl ShardRouter for BadRouter {
    fn db_shards(&self) -> usize { 1 }
    fn table_shards(&self) -> usize { 1 }
    fn db_index(&self, _: &Value) -> Result<usize> { Ok(0) }
    fn table_name(&self, _: &str, _: &Value) -> Result<String> {
        Ok("users; DROP TABLE x".into())
    }
    fn table_name_at(&self, _: &str, _: usize) -> Result<String> {
        Ok("users; DROP TABLE x".into())
    }
}

#[tokio::test]
async fn invalid_physical_table_name_is_rejected() {
    let shards = ShardedPools::new(vec![Mock::default()], Arc::new(BadRouter)).unwrap();
    let handle = shards.for_key(Value::Int(1)).unwrap();
    assert!(handle.resolve_table("users").is_err());
}

struct ReturningDb;
#[async_trait]
impl Db for ReturningDb {
    async fn query(&self, _: &str, _: &[Value]) -> Result<Vec<Row>> { Ok(Vec::new()) }
    async fn execute(&self, _: &str, _: &[Value]) -> Result<u64> { Ok(1) }
    async fn insert_returning(
        &self, _: &str, _: &str, _: &str, _: &[Value],
    ) -> Result<Option<Row>> {
        Ok(Some(row_of(&[("id", serde_json::json!(1))])))
    }
}

#[tokio::test]
async fn insert_returning_delegates_to_the_selected_pool() {
    let shards =
        ShardedPools::new(vec![ReturningDb], Arc::new(HashRouter::new(1, 1).unwrap())).unwrap();
    let handle = shards.for_key(Value::Int(1)).unwrap();
    let row = Db::insert_returning(&handle, "users", "id", "INSERT ...", &[]).await.unwrap();
    assert!(row.is_some());
}

#[tokio::test]
async fn dialect_is_forwarded() {
    let shards = ShardedPools::new(
        vec![Mock::with_dialect(Dialect::Mysql)],
        Arc::new(HashRouter::new(1, 1).unwrap()),
    )
    .unwrap();
    assert_eq!(shards.for_key(Value::Int(1)).unwrap().dialect(), Some(Dialect::Mysql));
}
```

Run: `cargo test -p bee_orm --lib`
Expected: PASS。

- [ ] **Step 5: 在 lib.rs 导出**

```rust
pub mod shard;
pub use shard::{HashRouter, ShardHandle, ShardRouter, ShardedPools};
```

Run: `cargo build -p bee_orm`
Expected: PASS。

- [ ] **Step 6: Commit**

```bash
git add crates/bee_orm/src/shard.rs crates/bee_orm/src/lib.rs \
  crates/bee_orm/src/tests/shard.rs crates/bee_orm/src/tests/mod.rs
git commit -m "feat(bee_orm): add key-required hash sharding"
```

---

### Task 7: sqlite 分库分表集成测试

**Files:**
- Create: `crates/bee_orm/tests/sharding_sqlite.rs`

**Interfaces:**
- Consumes: `ShardedPools`, `HashRouter`, `Model`, `migrate::MigrateOptions`（Task 6）。
- Produces: 一个不依赖外部服务的真读写测试，证明路由、建表、跨分片隔离。

- [ ] **Step 1: 写测试**

```rust
#![cfg(feature = "sqlite")]

use std::sync::Arc;

use bee_orm::pool::sqlite::Pool;
use bee_orm::shard::{HashRouter, ShardRouter, ShardedPools};
use bee_orm::{MigrateOptions, Model, OrmError, Value};

#[derive(Model, Debug, Clone, PartialEq)]
#[bee(table = "shard_users")]
struct ShardUser {
    #[bee(pk, auto)]
    id: i64,
    user_id: i64,
    name: String,
}

async fn setup() -> (ShardedPools<Pool>, Pool, Pool, Arc<HashRouter>) {
    let p0 = Pool::connect(":memory:", 1).unwrap();
    let p1 = Pool::connect(":memory:", 1).unwrap();
    let router = Arc::new(HashRouter::new(2, 4).unwrap());
    let shards =
        ShardedPools::new(vec![p0.clone(), p1.clone()], router.clone()).unwrap();
    shards.sync_all::<ShardUser>(MigrateOptions::default()).await.unwrap();
    (shards, p0, p1, router)
}

#[tokio::test]
async fn sync_all_creates_every_physical_table() {
    let (_shards, p0, p1, _router) = setup().await;
    for pool in [&p0, &p1] {
        let rows = pool
            .query(
                "SELECT name FROM sqlite_master WHERE type = 'table' \
                 AND name LIKE 'shard_users_%' ORDER BY name",
                &[],
            )
            .await
            .unwrap();
        let names: Vec<&str> = rows.iter().filter_map(|r| r["name"].as_str()).collect();
        assert_eq!(names, vec!["shard_users_0", "shard_users_1", "shard_users_2", "shard_users_3"]);
    }
}

#[tokio::test]
async fn keyed_crud_stays_on_the_routed_shard() {
    let (shards, p0, p1, _router) = setup().await;

    // Composite 2×4 routing: Int(5) -> pool 0 / shard_users_0;
    // Int(1) -> pool 1 / shard_users_0 (same table name, different database).
    let db1 = shards.for_key(Value::Int(5)).unwrap();
    let user = ShardUser { id: 0, user_id: 5, name: "alice".into() };
    user.insert(&db1).await.unwrap();

    let found = ShardUser::query().filter_eq("user_id", 5).unwrap().all(&db1).await.unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "alice");

    let db2 = shards.for_key(Value::Int(1)).unwrap();
    let other = ShardUser::query().filter_eq("user_id", 5).unwrap().all(&db2).await.unwrap();
    assert!(other.is_empty(), "key 1 must not see key 5's row");

    assert_eq!(count(&p0, "shard_users_0").await, 1);
    assert_eq!(count(&p1, "shard_users_0").await, 0);
}

#[tokio::test]
async fn unsupported_key_is_an_error() {
    let (shards, _p0, _p1, _router) = setup().await;
    assert!(matches!(shards.for_key(Value::Null), Err(OrmError::QueryError(_))));
}

#[tokio::test]
async fn sync_all_is_idempotent() {
    let (shards, _p0, _p1, _router) = setup().await;
    assert_eq!(shards.sync_all::<ShardUser>(MigrateOptions::default()).await.unwrap(), 0);
}

async fn count(pool: &Pool, table: &str) -> i64 {
    let rows = pool.query(&format!("SELECT COUNT(*) AS n FROM {table}"), &[]).await.unwrap();
    rows[0]["n"].as_i64().unwrap()
}
```

- [ ] **Step 2: 运行**

Run: `cargo test -p bee_orm --features sqlite --test sharding_sqlite`
Expected: PASS。若失败，先修 Task 6 的路由，不要放宽断言。

- [ ] **Step 3: Commit**

```bash
git add crates/bee_orm/tests/sharding_sqlite.rs
git commit -m "test(bee_orm): prove sqlite sharding routing and isolation"
```

---

### Task 8: MySQL 分片集成测试

**Files:**
- Create: `crates/bee_orm/tests/sharding_mysql.rs`

**Interfaces:**
- Consumes: Task 6 的 `ShardedPools`；`common::dsn("BEE_ORM_MYSQL_DSN")`。
- Produces: 同库分表 + 跨库分表的真 MySQL 测试；随 Task 1 的矩阵自动在三版本上跑。

- [ ] **Step 1: 写测试**

文件顶部 `#![cfg(feature = "mysql")]`，`mod common;`。模型：

```rust
#[derive(Model, Debug, Clone, PartialEq)]
#[bee(table = "shard_orders")]
struct ShardOrder {
    #[bee(pk, auto)]
    id: i64,
    user_id: i64,
    name: String,
}

fn with_database(dsn: &str, db: &str) -> String {
    // shortcut: DSN path swap ignores query params; upgrade if the CI DSN gains them.
    let base = dsn.rsplit_once('/').map(|(base, _)| base).unwrap_or(dsn);
    format!("{base}/{db}")
}
```

两个测试共用：

```rust
async fn admin() -> Result<Option<(Pool, String)>, OrmError> {
    let Some(dsn) = common::dsn("BEE_ORM_MYSQL_DSN") else { return Ok(None); };
    let admin = Pool::connect(&dsn, 1)?;
    Ok(Some((admin, dsn)))
}
```

两个测试都写成 `async fn ...() -> Result<(), OrmError>`，开头：

```rust
let Some((admin, dsn)) = admin().await? else { return Ok(()) };
```

测试 1：`table_shards_round_trip`（`HashRouter::new(1, 2)`，一个 pool）
- `DROP TABLE IF EXISTS shard_orders_0, shard_orders_1`
- `ShardedPools::new(vec![Pool::connect(&dsn, 2)?], Arc::new(router))`
- `sync_all::<ShardOrder>`
- `shards.for_key(Value::Int(1))` → 表 `shard_orders_0`；`for_key(Value::Int(2))` → `shard_orders_1`
- 各插一行，断言两个句柄各自只看到自己的行；admin 查两张物理表计数 1/1。

测试 2：`database_shards_round_trip`（`HashRouter::new(2, 1)`，两个 pool）
- `admin.execute("CREATE DATABASE IF NOT EXISTS bee_shard_0", &[])`，`bee_shard_1` 同理。
- `let p0 = Pool::connect(&with_database(&dsn, "bee_shard_0"), 2)?;`，p1 同理。
- `DROP TABLE IF EXISTS bee_shard_0.shard_orders` / `bee_shard_1.shard_orders`。
- `sync_all`；`for_key(Int(1))` → p0；`for_key(Int(2))` → p1。
- 各插一行；admin 分别查 `bee_shard_0.shard_orders` 和 `bee_shard_1.shard_orders`，断言 1/1。
- 清理两张表。

DSN 未设置时返回 `Ok(())`（与 `integration_mysql.rs` 的既有模式一致）。

- [ ] **Step 2: 运行（有 MySQL 时）**

```bash
BEE_ORM_MYSQL_DSN=mysql://root:root@127.0.0.1:3306/bee_test \
  cargo test -p bee_orm --features mysql --test sharding_mysql
```
Expected: PASS；没有 DSN 时测试打印 skip 并以 0 退出。

- [ ] **Step 3: Commit**

```bash
git add crates/bee_orm/tests/sharding_mysql.rs
git commit -m "test(bee_orm): prove MySQL table and database sharding"
```

---

### Task 9: `bee_kv` Redis 真连接池

**Files:**
- Modify: `crates/bee_kv/Cargo.toml`
- Modify: `crates/bee_kv/src/redis_store.rs`
- Test: `crates/bee_kv/tests/redis_reconnect.rs`（既有）、`crates/bee_kv/tests/kv_store_e2e.rs`（既有）、新增 `crates/bee_kv/tests/redis_pool.rs`

**Interfaces:**
- Produces:
  - `RedisStore::new(addr: &str) -> Result<Self, KvError>`（签名不变，内部池默认 16）
  - `RedisStore::with_pool_size(addr: &str, max_size: usize) -> Result<Self, KvError>`
  - `pub(crate) async fn connect_pool(addr: &str, max_size: usize) -> Result<deadpool_redis::Pool, KvError>`（给 Task 11 复用）

- [ ] **Step 1: 改 Cargo.toml**

```toml
[features]
redis = ["dep:redis", "dep:deadpool-redis"]

[dependencies.redis]
version = "0.27"
optional = true
features = ["tokio-comp", "aio"]

[dependencies.deadpool-redis]
version = "0.18"
optional = true
```

（去掉 `connection-manager`，因为不再使用 `ConnectionManager`。）

- [ ] **Step 2: 写失败测试**

新建 `crates/bee_kv/tests/redis_pool.rs`：

```rust
#![cfg(feature = "redis")]

use std::sync::Arc;

use bee_kv::{KvStore, RedisStore};

#[tokio::test]
async fn concurrent_gets_with_a_small_pool_all_succeed() {
    let Some(dsn) = std::env::var("BEE_KV_REDIS_DSN").ok().filter(|s| !s.is_empty()) else {
        return;
    };
    let store = Arc::new(RedisStore::with_pool_size(&dsn, 2).await.unwrap());
    store.set("bee_kv:pool:probe", "ok").await.unwrap();

    let mut set = tokio::task::JoinSet::new();
    for _ in 0..32 {
        let store = Arc::clone(&store);
        set.spawn(async move { store.get("bee_kv:pool:probe").await });
    }
    while let Some(result) = set.join_next().await {
        assert_eq!(result.unwrap().unwrap().as_deref(), Some("ok"));
    }
    store.del("bee_kv:pool:probe").await.unwrap();
}
```

Run: `BEE_KV_REDIS_DSN=redis://127.0.0.1:6379 cargo test -p bee_kv --features redis --test redis_pool`
Expected: FAIL（`with_pool_size` 不存在）。

- [ ] **Step 3: 替换连接实现**

在 `redis_store.rs`：

- `struct RedisStore { pool: deadpool_redis::Pool }`
- 池构造：

```rust
pub(crate) async fn connect_pool(
    addr: &str,
    max_size: usize,
) -> Result<deadpool_redis::Pool, KvError> {
    let pool = deadpool_redis::Config::from_url(addr)
        .builder()
        .map_err(conn_err)?
        .max_size(max_size.max(1))
        .timeouts(deadpool_redis::Timeouts {
            wait: Some(Duration::from_secs(30)),
            create: Some(Duration::from_secs(5)),
            recycle: Some(Duration::from_secs(5)),
        })
        .runtime(deadpool_redis::Runtime::Tokio1)
        .build()
        .map_err(conn_err)?;
    let mut conn = pool.get().await.map_err(conn_err)?;
    redis::cmd("PING").query_async::<String>(&mut conn).await.map_err(conn_err)?;
    Ok(pool)
}
```

- `new` → `Self::with_pool_size(addr, 16).await`
- `with_pool_size` → `Ok(Self { pool: connect_pool(addr, max_size).await? })`
- 每个命令：`let mut conn = self.pool.get().await.map_err(conn_err)?; redis::cmd(...).query_async(&mut conn).await.map_err(op_err)`
- 加私有 `conn_err` / `op_err`，把 `PoolError<RedisError>` / `RedisError` 映射到 `KvError::ConnectionError` / `OperationFailed`。
- 更新模块文档：不再说 `ConnectionManager`；说明池在 checkout 时 recycle（PING），连接被杀后下一次操作恢复；池耗尽按 `wait` 超时报错。

- [ ] **Step 4: 跑既有 + 新测试**

```bash
cargo test -p bee_kv --all-features
BEE_KV_REDIS_DSN=redis://127.0.0.1:6379 cargo test -p bee_kv --features redis
```
Expected: PASS；`redis_reconnect.rs` 的“被杀后恢复”仍然成立。

- [ ] **Step 5: Commit**

```bash
git add crates/bee_kv/Cargo.toml crates/bee_kv/src/redis_store.rs \
  crates/bee_kv/tests/redis_pool.rs
git commit -m "feat(bee_kv): back RedisStore with a real connection pool"
```

---

### Task 10: `bee_cache` Redis 真连接池

**Files:**
- Modify: `crates/bee_cache/Cargo.toml`
- Modify: `crates/bee_cache/src/redis_cache.rs`
- Test: `crates/bee_cache/tests/remote_caches.rs`（既有）、新增 `crates/bee_cache/tests/redis_pool.rs`

**Interfaces:**
- Produces:
  - `RedisCache::new(addr: &str) -> Result<Self, CacheError>`（签名不变）
  - `RedisCache::with_pool_size(addr: &str, max_size: usize) -> Result<Self, CacheError>`

- [ ] **Step 1: 改 Cargo.toml**

同 Task 9：`redis` 依赖去掉 `connection-manager`，加 `deadpool-redis = "0.18"`，feature 加 `dep:deadpool-redis`。

- [ ] **Step 2: 写失败测试**

`crates/bee_cache/tests/redis_pool.rs` 与 Task 9 同构：`RedisCache::with_pool_size(&dsn, 2)`，32 个并发 `get`，全部成功；用 `Cache` trait 的 `set` / `get` / `delete`。

Run: `BEE_KV_REDIS_DSN=redis://127.0.0.1:6379 cargo test -p bee_cache --features redis --test redis_pool`
Expected: FAIL（`with_pool_size` 不存在）。

- [ ] **Step 3: 替换实现**

与 Task 9 相同：`struct RedisCache { pool: deadpool_redis::Pool }`；`connect_pool` 的复制版（错误类型换成 `CacheError`，这是仓库对 memcache 也采用的“每个 trait 一份小实现”策略）；`new` 默认池 16；命令走 `pool.get()`。更新模块文档里的 `ConnectionManager` 描述。

- [ ] **Step 4: 跑测试**

```bash
cargo test -p bee_cache --all-features
BEE_KV_REDIS_DSN=redis://127.0.0.1:6379 cargo test -p bee_cache --features redis
```
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/bee_cache/Cargo.toml crates/bee_cache/src/redis_cache.rs \
  crates/bee_cache/tests/redis_pool.rs
git commit -m "feat(bee_cache): back RedisCache with a real connection pool"
```

---

### Task 11: Redis 分布式锁

**Files:**
- Create: `crates/bee_kv/src/redis_lock.rs`
- Modify: `crates/bee_kv/src/lib.rs`
- Modify: `crates/bee_kv/src/redis_store.rs`（`lock()` 访问器）
- Test: `crates/bee_kv/tests/redis_lock.rs`

**Interfaces:**
- Produces:

```rust
pub struct RedisLock { /* pool */ }
impl RedisLock {
    pub async fn connect(addr: &str) -> Result<Self, KvError>;
    pub async fn with_pool_size(addr: &str, max_size: usize) -> Result<Self, KvError>;
    pub async fn try_lock(&self, key: &str, ttl: Duration) -> Result<Option<LockGuard>, KvError>;
}

pub struct LockGuard { /* pool, key, token */ }
impl LockGuard {
    pub async fn release(self) -> Result<bool, KvError>;
}

impl RedisStore {
    pub fn lock(&self) -> RedisLock; // 复用同一个池
}
```

- [ ] **Step 1: 写失败测试**

`crates/bee_kv/tests/redis_lock.rs`：

```rust
#![cfg(feature = "redis")]

use std::time::Duration;

use bee_kv::RedisLock;

fn key(name: &str) -> String {
    format!("bee_kv:lock:test:{name}:{}", std::process::id())
}

#[tokio::test]
async fn try_lock_is_exclusive_and_release_frees_it() {
    let Some(dsn) = std::env::var("BEE_KV_REDIS_DSN").ok().filter(|s| !s.is_empty()) else {
        return;
    };
    let lock = RedisLock::connect(&dsn).await.unwrap();
    let k = key("exclusive");

    let guard = lock.try_lock(&k, Duration::from_secs(5)).await.unwrap().unwrap();
    assert!(lock.try_lock(&k, Duration::from_secs(5)).await.unwrap().is_none());
    assert!(guard.release().await.unwrap());
    assert!(lock.try_lock(&k, Duration::from_secs(5)).await.unwrap().is_some());
}

#[tokio::test]
async fn ttl_expires_the_lock() {
    let Some(dsn) = std::env::var("BEE_KV_REDIS_DSN").ok().filter(|s| !s.is_empty()) else {
        return;
    };
    let lock = RedisLock::connect(&dsn).await.unwrap();
    let k = key("ttl");

    assert!(lock.try_lock(&k, Duration::from_millis(100)).await.unwrap().is_some());
    assert!(lock.try_lock(&k, Duration::from_secs(5)).await.unwrap().is_none());
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(lock.try_lock(&k, Duration::from_secs(5)).await.unwrap().is_some());
}

#[tokio::test]
async fn releasing_an_expired_guard_returns_false() {
    let Some(dsn) = std::env::var("BEE_KV_REDIS_DSN").ok().filter(|s| !s.is_empty()) else {
        return;
    };
    let lock = RedisLock::connect(&dsn).await.unwrap();
    let k = key("expired");

    let guard = lock.try_lock(&k, Duration::from_millis(100)).await.unwrap().unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!guard.release().await.unwrap());
}
```

- [ ] **Step 2: 运行确认失败**

Run: `BEE_KV_REDIS_DSN=redis://127.0.0.1:6379 cargo test -p bee_kv --features redis --test redis_lock`
Expected: FAIL（`RedisLock` 不存在）。

- [ ] **Step 3: 实现 `redis_lock.rs`**

要点：

- `RedisLock::connect` / `with_pool_size` 调 `crate::redis_store::connect_pool`（Task 9 的 `pub(crate)`）。
- `try_lock`：
  - 空 key 或 `ttl.as_millis() == 0` → `KvError::OperationFailed("lock: key and ttl must be non-empty"...)`。
  - token：`format!("{}-{}-{}", std::process::id(), nanos, COUNTER.fetch_add(1, Relaxed))`；`nanos` 用 `SystemTime::now().duration_since(UNIX_EPOCH)`，失败回退 0；`static COUNTER: AtomicU64`。
  - `redis::cmd("SET").arg(key).arg(&token).arg("NX").arg("PX").arg(ttl_ms).query_async::<Option<String>>(&mut conn)`；`Some(_)` → `Some(LockGuard { ... })`，`None` → `None`。
- `release`：Lua：

```lua
if redis.call("GET", KEYS[1]) == ARGV[1] then
    return redis.call("DEL", KEYS[1])
else
    return 0
end
```

  `redis::Script::new(...).key(&self.key).arg(&self.token).invoke_async::<i64>(&mut conn)`；返回 `Ok(n != 0)`。`self` 被消费，重复释放不会发生。
- `Drop` 不实现释放；文档写明 TTL 是兜底、必须显式 `release`。
- `lib.rs`：`#[cfg(feature = "redis")] mod redis_lock;` + `pub use redis_lock::{LockGuard, RedisLock};`
- `RedisStore::lock(&self) -> RedisLock`：`RedisLock::from_pool(self.pool.clone())`（`from_pool` 用 `pub(crate)`）。

- [ ] **Step 4: 跑测试**

```bash
BEE_KV_REDIS_DSN=redis://127.0.0.1:6379 cargo test -p bee_kv --features redis
cargo test -p bee_kv --all-features
```
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/bee_kv/src/redis_lock.rs crates/bee_kv/src/lib.rs \
  crates/bee_kv/src/redis_store.rs crates/bee_kv/tests/redis_lock.rs
git commit -m "feat(bee_kv): add a pooled Redis distributed lock"
```

---

### Task 12: 文档与变更记录

**Files:**
- Modify: `README.md`
- Modify: `docs/README.en.md`
- Modify: `docs/CHANGELOG.md`
- Modify: `docs/CHANGELOG.zh.md`

**Interfaces:**
- Consumes: 前面所有任务的公开 API。

- [ ] **Step 1: README 功能表**

- 在关系型后端表里把 MySQL 行改成 `MySQL 8.0 / 8.4 / 9.x`。
- 在 ORM 段落补一句：`分库分表（ShardedPools + HashRouter，查询必须带分片键；不支持跨分片查询/事务）`。
- 在 KV/缓存段落补一句：`Redis 真连接池（deadpool-redis）与分布式锁（SET NX PX + Lua 释放）`。
- 中文根 README 和 `docs/README.en.md` 同步。

- [ ] **Step 2: CHANGELOG**

在 `docs/CHANGELOG.md` 和 `docs/CHANGELOG.zh.md` 顶部加 `## [Unreleased]`，`### Added` 写三条：

- `bee_orm`: MySQL 8.0 / 8.4 / 9.x compatibility, verified by a CI matrix.
- `bee_orm`: `shard::{ShardedPools, HashRouter, ShardRouter}` — key-required hash sharding for databases and tables; no cross-shard queries/transactions.
- `bee_kv` / `bee_cache`: real Redis connection pool (`deadpool-redis`) and `bee_kv::RedisLock` (`SET NX PX` + Lua release).

- [ ] **Step 3: 更新测试计数表**

先按仓库既有口径逐 crate 计数：

```bash
for c in bee_config bee_cache bee_template bee_logs bee_kv bee_search bee_graph \
         bee_tsdb bee_orm bee_orm_macro bee_session bee_router bee_cli; do
  n=$(grep -rE '^\s*#\[(tokio::)?test\]' "crates/$c" --include='*.rs' | wc -l)
  echo "$c $n"
done
```

把结果写回 README 的“测试覆盖”表（中文根 README 与英文 README），并同步总数。注意：当前表已经与代码有漂移（`bee_orm` 141 vs 实际 161），本步骤以命令输出为准，不要沿用旧数字。

- [ ] **Step 4: 校验文档**

Run: `git diff --check`
Expected: 无空白错误。人工确认中英文两处数字一致。

- [ ] **Step 5: Commit**

```bash
git add README.md docs/README.en.md docs/CHANGELOG.md docs/CHANGELOG.zh.md
git commit -m "docs: document MySQL 8.0-9.x, sharding, and Redis pool/lock"
```

---

## 自检（Self-Review）

**Spec coverage**

| 需求 | 任务 |
|---|---|
| MySQL 8.0 / 8.4 / 9 真库验证 | Task 1、Task 8 |
| 分库分表（必须带分片键） | Task 2–8 |
| 分片 DDL / 迁移 | Task 4、Task 6、Task 7 |
| Redis 真连接池 | Task 9、Task 10 |
| Redis 分布式锁 | Task 11 |
| 文档/变更记录 | Task 12 |

**Type consistency**

- `Db::resolve_table` 在 Task 2 定义为 `Result<String>`；Task 3、4、5、6 的所有调用都按这个签名。
- `ShardRouter::table_name_at` 返回 `Result<String>`；`sync_all` 与自定义 router 测试都按这个签名。
- `ShardedPools::for_key` 返回 `Result<ShardHandle>`；Task 7、8 的测试都带 `?`/`unwrap()`。
- `RedisStore::with_pool_size` 增加 `usize` 参数；Task 9 的测试用 `2`；`RedisCache` 同构。

**Review Focus 对应的测试**

- 不支持的分片键：Task 6 `unsupported_keys_are_rejected`、Task 7 `unsupported_key_is_an_error`。
- 非法物理表名：Task 6 `invalid_physical_table_name_is_rejected`。
- 跨分片隔离：Task 7 `keyed_crud_stays_on_the_routed_shard`、Task 8 两个 round-trip。
- Redis 池并发/恢复：Task 9/10 `concurrent_gets_with_a_small_pool_all_succeed` + 既有 `redis_reconnect`。
- 锁所有权/TTL：Task 11 三个测试。
- MySQL 8.4/9 认证：Task 1 的 CI 矩阵 + Task 8 的真库测试。

**Proportion**

计划中的代码块是签名、关键 SQL 形状和测试断言；实现体留给执行者。没有为“以后可能需要”的跨分片查询、自动续期、Redis Cluster 预留代码。
