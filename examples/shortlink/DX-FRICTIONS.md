# DX 摩擦日志 — 用 bee_rust 1.2.1 从零写一个短链服务

- 日期: 2026-10-06
- 框架: `bee_rust = 1.2.1`（crates.io 实拉，feature: `orm-sqlite`, `router`, `cache`, `logs`）
- 工具链: `rustc 1.99.0 (b940084d7 2026-09-28)` / `cargo 1.99.0`
- 平台: Linux 6.6.155-amd64-desktop-hwe x86_64
- 参考面: 仓库 README.md、docs/crates-readme.md、docs/api.md、docs.rs 的 bee_* rustdoc
- 翻源码: **0 次**（未读 `crates/**`、未读 cargo registry 源码，无救急条目）

---

## F-1 `#[derive(Model)]` 展开引用 `bee_orm::`，只依赖 `bee_rust` 时直接编译失败

- 类别: 报错质量（+ 文档缺失）
- 场景: 第一次 `cargo build`，写 `src/models.rs` 里的 `#[derive(Model)]`
- 最小复现:
  ```toml
  # Cargo.toml —— 按 crates-readme 的建议只依赖元 crate
  bee_rust = { version = "1.2.1", default-features = false, features = ["orm-sqlite", "router", "cache", "logs"] }
  ```
  ```rust
  use bee_rust::bee_orm::Model;
  #[derive(Model)]
  #[bee(table = "t")]
  struct T { #[bee(pk, auto)] id: i64 }
  ```
  报错原文:
  ```
  error[E0433]: cannot find module or crate `bee_orm` in this scope
   --> src/models.rs:3:10
    = note: this error originates in the derive macro `Model`
  ```
- 解卡方式: 试错（1 个编译回合）。在调用点把重导出引入作用域即可：`use bee_rust::bee_orm;`
- 影响: ~1 分钟；不卡死，但首屏报错指向 `#[derive(Model)]`，第一反应是"feature 没开对"，会先去查 feature 表
- 改进建议: 宏展开改用可从 `bee_rust` 解析的路径（或 `proc-macro-crate` 探测）；至少在 api.md 的 ORM 小节注明"只用 `bee_rust` 时要 `use bee_rust::bee_orm;`，或直接依赖 `bee_orm`"——目前 api.md 的例子全部写 `use bee_orm::…`，暗含"bee_orm 是直接依赖"，与 crates-readme 的推荐依赖集不一致

## F-2 Pool/Cache 如何注入 handler，公开文档零覆盖

- 类别: 文档缺失
- 场景: 设计 `build_app`——每个真应用的第一件事："路由 handler 里怎么拿到 `Pool` 和 `Cache`？"
- 最小复现: README 快速开始的 handler 是 `async fn health() -> &'static str`（无参数）；api.md 的 ORM 段是一个孤立的 `async fn demo()`，不经过任何路由。两处都没有"带状态的 handler"。全文档搜不到 `with_state` / `State` / 依赖注入的用法
- 解卡方式: 文档内解决（换 rustdoc 面）：docs.rs `bee_router::router::Router` 页有一句 "The state type S … passed to handlers through axum extractors such as `axum::extract::State`"，据此写 `Router::new().ns(...).with_state(state)` + `State<AppState>` 提取器，一次编译通过
- 影响: ~15 分钟（翻 rustdoc 才敢动手）；对不想学 axum 的 Beego 背景用户是硬伤——README 主推的 `Controller`/`Context` 路线反而没有（我最终用的是裸 axum handler + bee_router 只做路由注册）
- 改进建议: api.md 加一节最小示例："带 state 的 handler"（Pool + Cache 注入 + 一个查询），这是把 demo 变成真应用的分水岭

## F-3 docs.rs rustdoc 的覆盖缺口（derive 宏空页 / 后端页 404）

- 类别: 文档缺失
- 场景: 查 `#[bee(...)]` 全部属性、`Pool::connect` 签名、m2m 函数签名，准备写模型层
- 最小复现:
  - `https://docs.rs/bee_orm/1.2.1/bee_orm/derive.Model.html` 正文只有一句 "Derive bee_orm::Model for a struct."——属性列表 / 生成的 `query()` / `insert` 语义全无
  - `https://docs.rs/bee_orm/1.2.1/bee_orm/pool/sqlite/...` → 404（docs.rs 构建未开 sqlite feature，`Pool`/`CheckedConn` 无 rustdoc）
  - bee_router 页头标注 "22.73% of the crate is documented"
  - `Model::query()` 在 trait 的 rustdoc（22 个方法）中不存在——它由 derive 生成，只在 docs.rs 看不到；api.md 却在用它
- 解卡方式: 文档内解决（rustdoc + api.md 交叉）：签名从 rustdoc 拿（`filter_eq -> Result<Self>`、`m2m::attach` 等），`Pool::connect(dsn, max_size)` 与 `query()` 从 api.md 拿
- 影响: ~20 分钟显式翻页；没卡死，但"文档写没写"靠碰运气，可靠性取决于例子
- 改进建议: docs.rs 构建时打开后端 feature；给 derive 宏页补属性表；`query()` 的生成物在 rustdoc 里不可见，至少在 `Model` trait 文档里文字说明"derive 会生成 `query()` 与 `insert/update/delete` 固有方法"

## F-4 路由路径模板语法与 `ns` 前缀语义未文档化

- 类别: 文档缺失
- 场景: 注册 `GET /:code`（根路径）与 `POST /api/links/:code/tags`
- 最小复现: rustdoc `RouteGroup::get(path, handler)` 对 `path` 只字未提模板语法；api.md 的 `ns("/api/v1", |ns| ns.get("/users"))` 也没说明前缀是拼接还是替换、允不允许空前缀
- 解卡方式: 试错（先验默认 axum 0.8 语法）：`/{code}` 编译通过、行为正确；根路由用 `.ns("", |g| g.get("/{code}", …))` 也工作。但要赌对两件事：a) bee_router 不翻译 `:code` 而是透传 axum（`{code}` 才合法）；b) 空前缀合法。若先写 `:code`（Beego 用户直觉），得到的会是 axum 的 panic 而不是文档
- 影响: ~5 分钟（恰好熟悉 axum 0.8）；Beego 背景用户大概率先踩 `:param` 坑
- 改进建议: rustdoc/API 文档写清"path 即 axum 0.8 语法，参数 `{name}`"；`ns` 注明前缀拼接规则与空前缀合法性

## F-5 `insert()` 的 u64 返回值未文档化；auto 主键导致每个创建路径都要二次查询

- 类别: API 设计（+ 文档缺失）
- 场景: `POST /api/links` 要返回 `{id, code, url, created_at}`，但 `insert(&self) -> Result<u64>`，且 `#[bee(pk, auto)]` 由数据库分配 id、`#[bee(auto_now_add)]` 由数据库写时间
- 最小复现（实测探针）:
  ```rust
  let ret = link.insert(&pool).await?;      // 第 2 行的 ret = 1
  let row = Link::query().filter_eq("code", code)?.one(&pool).await?.unwrap();
  // row.id = 2 —— 返回的是受影响行数，不是新主键
  ```
- 解卡方式: 试错（写临时探针测试实测返回值），然后按 api.md 的"插入后查询取回实例"二次 SELECT
- 影响: ~5 分钟探针 + 代码里 3 处"insert 完再 query 一次"（create_link、tag find-or-create、测试夹具）；每次创建多一次 DB 往返
- 改进建议: `insert` 返回主键（sqlite 的 `last_insert_rowid` 本来就有）；或在文档里写明返回行数、并提供一个返回完整实例的 `create(&self) -> Result<Self>`；`auto_now_add` 也可以改成应用侧生成（框架已有 `unix seconds` 口径），让响应无需回查

## F-6 [仓库/环境，非框架] `examples/` 下新建独立包被根 workspace 拒绝

- 类别: 报错质量（cargo/仓库结构；不属框架三类，如实标注）
- 场景: 第一次在 `examples/shortlink/` 执行 `cargo build`
- 最小复现: 根 `Cargo.toml` 已有 `exclude = ["examples/*"]`，但 cargo 1.99 仍报:
  ```
  error: current package believes it's in a workspace when it's not:
  current:   /home/wwwroot/bee-rust/examples/shortlink/Cargo.toml
  workspace: /home/wwwroot/bee-rust/Cargo.toml
  ```
- 解卡方式: 按 cargo 提示，在包内 `Cargo.toml` 顶部加空 `[workspace]` 表（只改本目录，未动根配置）
- 影响: ~2 分钟；此目录下任何新 example 都会复现（lead 的任务书假设 "examples/ 被根 workspace 排除"，实测该假设不成立）
- 改进建议: 根 `Cargo.toml` 把 `examples/shortlink` 像 `examples/hello` 一样写进 `workspace.members`，或核实 exclude glob 在该 cargo 版本下的行为
