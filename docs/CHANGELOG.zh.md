# 更新日志

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
