# 更新日志

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.2.0] — 2026-10-06

### 新增
- `bee_orm`：ORM 端到端可用——类型化 `Value` 绑定参数（Null / Bool / Int / Float / Text / Bytes）、`#[derive(Model)]` 支持 `#[bee(table / column / pk / auto / ignore)]`、模型 `insert` / `update` / `delete`，以及 `QuerySet` 执行层（`all` / `one` / `count` / `exists` / `update` / `delete`）
- `bee_orm`：三后端连接池（`pool::{sqlite, postgres, mysql}::Pool`）——`connect(dsn, max_size)`、`get()` → `CheckedConn`、`query` / `execute`、`status()`，事务用 `begin` / `commit` / `rollback`；事务中途被 drop 的连接在 sqlite 与 postgres 上自动回滚
- `bee_orm`：新增 `postgres-tls` feature（PostgreSQL TLS，内置 Mozilla 根证书）；postgres 池加固——30s 等待 / 10s 建连超时与预编译语句缓存
- `bee_orm`：模型生命周期增强——`#[bee(auto_now_add)]` / `#[bee(auto_now)]` 自动时间戳、`#[bee(soft_delete)]` 软删除（delete 改为翻转标记，`with_deleted()` / `hard_delete()` 可绕过）、insert/update/delete 钩子、`Model::insert_many` 批量插入（999 参数分批，非事务），以及 `QuerySet::filter_in` 与 `sum` / `avg` / `min` / `max` 聚合
- `bee_orm`：非破坏性迁移——`migrate::{create_table, add_missing_columns, sync}` 按方言生成 DDL（sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`），只建表与补列，绝不删除或修改既有列
- `bee_orm`：外键关系——`#[bee(fk = Target)]`（配合 `#[bee(sql_type = "…")]`）生成 FK DDL，`rel::{fk_column_to, belongs_to, children, children_for}` 提供读取；强制力因后端而异：postgres 与 sqlite（bundled 构建）强制，mysql 忽略 inline `REFERENCES`（round 5+ 的 gap）
- `bee_orm`：多对多关系——`#[bee(m2m(Target))]` 声明关系，`m2m::{attach, detach, related, related_for, related_ids}` 读写，join 表由 `create_table` / `sync` 自动创建（`add_missing_columns` 不触碰）
- `bee_orm`：JSON 列——`serde_json::Value` 字段按后端映射为 `TEXT` / `JSONB` / `JSON`；NULL 语义诚实：SQL `NULL` 解码为 `None`，存储的 JSON `null` 文档解码为 `Some(Json::Null)`
- `bee_orm`：MySQL 表级外键 opt-in——`MigrateOptions { table_level_fk }` 配合 `sync_with` / `create_table_with` / `add_missing_columns_with`；默认关闭，已有孤儿数据会让 `ADD CONSTRAINT` 报错而非静默跳过
- `bee_orm`：`Pool::connect_tls_with` 支持自定义 `rustls::ClientConfig`，并重导出 `bee_orm::rustls` 保证版本一致；`connect_tls` 仍用内置 webpki 根证书
- `bee_rust`：四个转发 feature——`orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` 通过 `bee_rust` 转发对应 `bee_orm` 后端（均不在 `full` 内）
- `bee_cli`：`bee-rust migrate init` 生成 `src/bin/bee_migrate.rs`（已存在则拒绝覆盖），`bee-rust migrate run` 通过 `cargo run --bin bee_migrate` 执行
- `bee_kv` / `bee_cache`：Redis 后端断线可恢复——底层 `ConnectionManager` 按需重连（无后台线程），指数退避 + 抖动（撞上断线的那条命令报错，下一条自动等待新连接）
- `bee_kv` / `bee_cache`：memcached 后端（bee_kv 的 `MemcacheStore`、bee_cache 的 `MemcacheCache`）；`incr` 先建计数器再增减，无符号计数器在 0 封底，TTL 0 等价删除

### 变更
- bee_orm 的迁移、关系（含多对多）、`bee-rust migrate` 子命令、Redis / Memcached 后端与 `bee_rust` 的 ORM 转发 feature，在文档中均不再标注为「计划中」
- 文档中 search / graph / tsdb 驱动不再标注为「计划中 / trait stub」，而是已实现（opt-in feature）；同时修正了旧示例的类型名与连接方式（`ElasticsearchEngine` / `Neo4jDB` 并不存在、`bolt://` 写错）

## [1.1.5] — 2026-09-25

### 新增

- 另 11 个语言的精简版 crates.io README：`docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`。crates.io 页面顶部的语言入口改指向这些文件，不再指向完整版仓库 README——各语言读者拿到的都是同一份面向开发者的短文。

## [1.1.4] — 2026-09-25

### 变更

- crates.io 页面（`docs/crates-readme.md`）改为中英双语：英文在上、中文在下。crates.io 每个 crate 只能渲染一份 README，也不支持站内切语言，这样国际开发者首屏即可读到英文，中文内容也不必离开同一页面。

## [1.1.3] — 2026-09-25

### 新增

- crates.io 页面（`docs/crates-readme.md`）新增 13 语言入口，指向仓库内的各语言 README

### 修复

- `docs/api.*`：11 个译本缺少指向中文原版 `api.md` 的链接
- `docs/CHANGELOG.zh.md` 与 `docs/CONTRIBUTING.zh.md` 是各自文档族中唯一没有自链接的文件

## [1.1.2] — 2026-09-24

### 新增

- `docs/crates-readme.md`：crates.io 页面专用 README——安装、可运行示例（取自 `examples/hello`，由其 E2E 测试覆盖）、feature 开关表、子 crate 一览
- `scripts/publish.sh`：逐个发布工作区 crate，跳过已上传的，并在被限流时按 crates.io 返回的重试时间等待

### 修复

- 测试套件无法编译（`cargo test --workspace` 在 main 上失败）：`bee_router` 集成测试中 `Submit` 缺少 `Serialize`（该结构体作为 `Json` 响应返回）、两个辅助函数缺少 `Router::build()`、8 处 `status_line` 调用缺少 `&`
- `hello` 示例测试引用了错误的 `CARGO_BIN_EXE` 名称——二进制 target 保留连字符（`CARGO_BIN_EXE_hello-bee`）
- `hello` 模板自动转义测试断言 `&#39;`，而 tera 实际输出 `&#x27;`；转义行为本身是正确的
- `bee_cli` 中的 Clippy `manual_split_once` 与 `manual_range_contains` lint；`bee_router` 集成测试中未使用的 import

### 变更

- 所有 crate 现已带有 `readme` 与 `repository` 元数据——此前 crates.io 页面均不渲染 README，且有 6 个 crate 完全缺少 `repository`
- `examples/hello` 标记 `publish = false`：它作为 E2E 测试载体保留在工作区内，但不再发布到 crates.io

## [1.0.6] — 2026-08-07

### 新增

- `bee_cli` 真实实现：`new`（项目脚手架）、`generate controller/model`、带 `--watch` 热重载的 `run`、`pack`（release 构建 + 复制到 `dist/`）
- CLI 脚手架与代码生成的单元测试（新增 7 个测试）

### 修复

- `bee_rust::init()` 现由 `logs` feature 门控——精简 feature 构建（如 `--no-default-features --features kv`）重新可编译
- 修复 `bee_kv::InMemoryKvStore::exists` 中的 Clippy `unnecessary_map_or` 警告
- 移除 `rustfmt.toml` 中在 stable 上被静默忽略的 nightly-only 选项；工作区现可通过 `cargo fmt --all --check`
- `bee_cli` 二进制设置 `doc = false`，消除与 `bee_rust` 的 rustdoc 输出文件名冲突
- `hello` 示例端口现可通过 `PORT` 环境变量配置

### 变更

- `bee-rust migrate` 报告"未实现"并以非零码退出（规划中）
- README / README.en 更新为描述真实的 CLI 行为

## [1.0.4] — 2026-07-29

### 新增

- 基于 `security-rust` 的安全攻击检测过滤器（27 个检测器）
- `SecurityFilter`，覆盖 XSS、SQL 注入、命令注入、路径穿越
- `bee_rust` 与 `bee_router` 中的 `security` feature 标志

### 变更

- 更新 README，加入安全特性文档
- 更新 README，加入打赏支持区块（微信支付 / 支付宝）

### 修复

- `bee_template` 适配 Rust 2024 edition 的 Tera 原始标识符语法

## [1.0.3] — 2026-07-29

### 新增

- 包含 13 个 crate 的初始工作区结构
- 带 `Controller` trait 与 `Router` 的 MVC 路由
- 带 `QuerySet` 构建器与 `Model` 派生宏的 ORM
- 带 Redis 与 Memory 后端的 KV/Cache trait 抽象

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
