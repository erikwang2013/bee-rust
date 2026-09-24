<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

Rust 生产级 Web 框架，设计哲学源自 Go 的 Beego，用 Rust 惯用的 trait / macro / 类型系统重新表达。

MVC 控制器 · 命名空间路由 · 过滤器链 · ORM · 多存储引擎统一 trait 抽象

---

## 安装

```bash
cargo add bee_rust
```

或写入 `Cargo.toml`：

```toml
[dependencies]
bee_rust = "1.1.2"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

默认启用 `full` feature（`router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template`）。

## 快速开始

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

完整可运行项目见 [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello)（含端到端测试）。

## Feature 开关

`search`、`graph`、`tsdb` **不在默认的 `full` 里**，需要单独开启。

| Feature | 拉起的子 crate | 说明 |
|---------|---------------|------|
| `full` *(默认)* | router, orm, kv, config, logs, cache, session, security, template | 一站式 |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Web 核心：路由 + 控制器 + 过滤器链 |
| `orm` | bee_orm, bee_config, bee_cache | Model 派生宏 + QuerySet |
| `kv` | bee_kv | KV 统一抽象 |
| `cache` | bee_cache, bee_config | 缓存抽象 |
| `session` | bee_session, bee_cache | Session 多后端 |
| `config` | bee_config | INI / YAML / ENV + 热更新 |
| `logs` | bee_logs | 多级日志 + tracing |
| `template` | bee_template | tera 模板渲染 |
| `security` | bee_router/security | 27 种攻击检测过滤器 |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

只想要最小核心（不拖 ORM、数据库驱动）：

```toml
bee_rust = { version = "1.1.2", default-features = false, features = ["router", "logs", "config"] }
```

## 子 Crate

本框架由多个可独立使用的 crate 组成，`bee_rust` 是统一入口，按需 re-export。

| Crate | 功能 | 对标 Beego |
|-------|------|-----------|
| `bee_rust` | 元 crate，统一入口 | — |
| `bee_router` | 路由 + 控制器 + Context + 过滤器 | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_config` | 配置管理 + 热更新 | `client/config` |
| `bee_logs` | 日志 | `logs` |
| `bee_cache` | 缓存抽象 | `client/cache` |
| `bee_session` | Session 管理 | `server/web/session` |
| `bee_template` | 模板渲染 | —（增强） |
| `bee_kv` | KV/Cache 统一抽象 | `client/cache`（扩展） |
| `bee_search` | 搜索/分析引擎 | —（新增） |
| `bee_graph` | 图数据库 | —（新增） |
| `bee_tsdb` | 时序数据库 | —（新增） |
| `bee_cli` | 脚手架 / 代码生成 / 开发运行 | `bee` 工具 |

这些 crate 也可以单独依赖，例如只用一个 ES 客户端：

```toml
bee_search = { version = "1.1.2", features = ["elasticsearch"] }
```

## 支持的数据库

| 类别 | 数据库 | Crate | Feature |
|------|--------|-------|---------|
| 关系型 | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / 缓存 | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| 搜索 / 分析 | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| 图数据库 | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| 时序数据库 | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## 环境要求

- Rust 1.80+（MSRV）
- edition 2024

## 链接

- **API 文档**：[docs.rs/bee_rust](https://docs.rs/bee_rust)
- **仓库**：[github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **完整 API 参考**（12 语言）：[docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## 许可证

Apache-2.0
