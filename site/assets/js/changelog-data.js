/* Bee Rust site — curated release history (source: docs/CHANGELOG.zh.md + CHANGELOG.md) */
window.BeeChangelog = [
  {
    version: '1.5.0', date: '2026-10-25', tag: 'latest',
    zh: {
      title: 'MySQL 5.7、create 快路径与 Redis 看门狗',
      summary: 'ORM 支持 MySQL 5.7 与 MariaDB 10.11，create() 在 MySQL 上跳过 catalog 探测；ShardedPools 开放池访问器；Redis 补齐状态、超时配置与锁自动续期。',
      items: [
        'MySQL 5.7 / 8.0 / 8.4 / 9.x 与 MariaDB 10.11：CI 矩阵覆盖五个服务端；MariaDB 的 JSON 实为带校验的 LONGTEXT，内联 REFERENCES 会被强制执行。',
        'bee_orm：Model::create() 在 MySQL 上按模型 AUTO_INCREMENT 元数据走快路径，跳过每次插入的 information_schema 探测；ShardedPools 新增 pools() / pool() / len() / is_empty() 访问器，作为跨分片操作与事务的逃生口。',
        'bee_kv / bee_cache：RedisOptions 统一连接池大小与响应超时（默认 30s），RedisStore / RedisCache 新增 status() 池指标。',
        'bee_kv::RedisLock：try_lock_with_watchdog 按 TTL/3 在后台续期，直到释放或丢弃；try_lock 仍不续期。',
        'CI：新增 docs 一致性检查（版本、安装片段与测试计数），在 push 与 PR 上运行。'
      ]
    },
    en: {
      title: 'MySQL 5.7, a create fast path, and a Redis watchdog',
      summary: 'The ORM adds MySQL 5.7 and MariaDB 10.11, create() skips the MySQL catalog probe, ShardedPools exposes its pools, and Redis gains status, configurable timeouts, and lock renewal.',
      items: [
        'MySQL 5.7 / 8.0 / 8.4 / 9.x and MariaDB 10.11: a five-server CI matrix; MariaDB stores JSON as validated LONGTEXT and enforces inline REFERENCES.',
        'bee_orm: Model::create() takes a fast path on MySQL when model metadata marks the pk AUTO_INCREMENT, skipping the per-insert information_schema probe; ShardedPools gains pools() / pool() / len() / is_empty() as the escape hatch for cross-shard operations and transactions.',
        'bee_kv / bee_cache: RedisOptions sets pool size and response timeout (30s default), and RedisStore / RedisCache expose status() pool metrics.',
        'bee_kv::RedisLock: try_lock_with_watchdog renews the TTL every TTL/3 in the background until release or drop; try_lock still does not renew.',
        'CI: a docs consistency gate (version, install snippets, test counts) now runs on push and pull requests.'
      ]
    }
  },
  {
    version: '1.4.0', date: '2026-10-25', tag: 'orm',
    zh: {
      title: 'MySQL 9、分库分表与 Redis 连接池',
      summary: 'ORM 打通 MySQL 8.0–9.x，新增必须带分片键的两级分片；Redis 后端换成真实连接池，并补上分布式锁。',
      items: [
        'MySQL 8.0 / 8.4 / 9.x：CI 三版本矩阵；MySQL 9 强制执行列级 REFERENCES，建表时先建被引用表。',
        'bee_orm::shard：ShardedPools / HashRouter / ShardHandle；复合哈希路由覆盖全部库×表组合；不支持跨分片查询与事务。',
        'bee_kv / bee_cache：deadpool-redis 真连接池，30 秒命令响应上限；断线自动替换，池耗尽按等待超时失败。',
        'bee_kv::RedisLock：SET NX PX 获取，Lua 比较 token 后释放；TTL 兜底，不自动续期。'
      ]
    },
    en: {
      title: 'MySQL 9, sharding, and a Redis pool',
      summary: 'The ORM now runs on MySQL 8.0–9.x, gains key-required two-level sharding, and the Redis backends move to real connection pools with a distributed lock.',
      items: [
        'MySQL 8.0 / 8.4 / 9.x: a three-version CI matrix; MySQL 9 enforces column-level REFERENCES, so referenced tables must be created first.',
        'bee_orm::shard: ShardedPools / HashRouter / ShardHandle; composite hashing reaches every database/table pair; no cross-shard queries or transactions.',
        'bee_kv / bee_cache: real deadpool-redis pools with a 30s per-command bound; killed connections are replaced, pool exhaustion waits then errors.',
        'bee_kv::RedisLock: SET NX PX acquire and Lua compare-and-delete release; TTL is the fallback, with no automatic renewal.'
      ]
    }
  },
  {
    version: '1.3.0', date: '2026-10-13', tag: 'fixes',
    zh: {
      title: '硬化与收敛',
      summary: '这一版不扩功能面，专修特性矩阵、收敛实现与依赖。每个单开后端特性都可独立编译，安全过滤器升级到 security-rust 4。',
      items: [
        'bee_tsdb / bee_graph 的共享 HTTP helper 移入各自的 crate::http，单开后端特性可独立编译。',
        'security-rust 4.0.0：32 个内置检测器，新增 format_string、log4shell、HPP、formula_injection、reDoS；检测器分强/弱档。',
        '合并重复的连接与错误映射辅助函数，移除零调用公共 API 与死依赖。',
        '弱信号（如重复查询参数）现在同样会被默认拦截；需要放行请自建 Filter 包 scanner。'
      ]
    },
    en: {
      title: 'Hardening & consolidation',
      summary: 'A release about tightening, not widening: every single-backend feature builds on its own, duplicate implementations and dead dependencies are gone, and the security filter moves to security-rust 4.',
      items: [
        'Shared HTTP helpers in bee_tsdb / bee_graph moved into each crate::http, so enabling one backend feature on its own compiles.',
        'security-rust 4.0.0: 32 built-in detectors, including new format_string, log4shell, HPP, formula_injection, and reDoS; detectors are tiered strong/weak.',
        'Duplicate connection and error-mapping helpers merged; zero-caller public APIs and dead dependencies removed.',
        'Weak signals (such as repeated query parameters) are now blocked by default; wrap SecurityFilter::scanner() in a custom Filter to let them through.'
      ]
    }
  },
  {
    version: '1.2.3', date: '2026-10-06', tag: 'fixes',
    zh: {
      title: '脚手架与文档修正',
      summary: 'CLI 不再生成无法编译的项目；API 示例、派生写法与示例项目同步修正。',
      items: [
        'bee-rust new 模板修正依赖名、async_trait 与 RouterError 路径；新增 BEE_CLI_E2E 门控的端到端 cargo check 测试。',
        'api 控制器示例改为可编译形态；补记 #[bee(crate = "...")]、{name} 路由语法与 create() 用法。',
        'shortlink 示例：500 细节走日志、URL 校验更严格、302 热路径保留 insert()。'
      ]
    },
    en: {
      title: 'Scaffold and docs fixes',
      summary: 'The CLI no longer scaffolds projects that fail to compile, and the API examples, derive spelling, and example project were corrected.',
      items: [
        'bee-rust new fixed the dependency name, async_trait, and RouterError path; a BEE_CLI_E2E-gated cargo check test now pins the scaffold end to end.',
        'The API controller example is compilable again; #[bee(crate = "...")], {name} route syntax, and create() usage are documented.',
        'shortlink: internal error details go to the log, URL validation is stricter, and the 302 hot path keeps insert().'
      ]
    }
  },
  {
    version: '1.2.2', date: '2026-10-06', tag: 'orm',
    zh: {
      title: 'create() 与派生路径',
      summary: '插入后直接拿到完整模型；派生宏可以指向任意 ORM crate 路径。',
      items: [
        'Model::create() 插入并返回完整实例，数据库分配的 auto 主键已填充（sqlite / postgres 走 RETURNING，mysql 回查）。',
        '#[bee(crate = "...")] 指定派生展开路径；只依赖 bee_rust 时写 #[bee(crate = "bee_rust::bee_orm")]。',
        '8 个带 feature 门控公共模块的 crate 补齐 docs.rs all-features = true，后端文档不再隐藏。'
      ]
    },
    en: {
      title: 'create() and crate paths',
      summary: 'Insert and get the full model back; point the derive macro at any ORM crate path.',
      items: [
        'Model::create() inserts and returns the full instance, with the database-assigned auto primary key included (RETURNING on sqlite / postgres, read-back on mysql).',
        '#[bee(crate = "...")] selects the ORM path used by the derive; with only bee_rust as a dependency, write #[bee(crate = "bee_rust::bee_orm")].',
        'Eight crates with feature-gated public modules gained docs.rs all-features = true, so backend docs are no longer invisible.'
      ]
    }
  },
  {
    version: '1.2.1', date: '2026-10-06', tag: 'orm',
    zh: {
      title: '日期、Decimal 与 CI',
      summary: 'chrono 与 rust_decimal 字段映射落地，并把三种后端的边界写清楚。',
      items: [
        'chrono / rust_decimal 字段映射 Date / DateTime / Decimal；通过 bee_rust 的 orm-chrono / orm-rust_decimal 转发。',
        'sqlite 存 TEXT；mysql 保留微秒；pg numeric 超过 29 位有效精度会读回 NULL——边界全部如实记录。',
        '无 feature 配置下 clippy --all-targets 修复；CLI migrate 端到端测试改为在线构建并接入 CI。'
      ]
    },
    en: {
      title: 'Date, Decimal, and CI',
      summary: 'chrono and rust_decimal fields land, with honest boundaries across all three backends.',
      items: [
        'chrono / rust_decimal map to Date / DateTime / Decimal; bee_rust forwards them as orm-chrono / orm-rust_decimal.',
        'sqlite stores TEXT, mysql keeps microseconds, and a pg numeric above 29 significant digits reads back NULL — boundaries are documented.',
        'Clippy --all-targets fixed in the feature-less configuration; the CLI migrate end-to-end test now builds online in CI.'
      ]
    }
  },
  {
    version: '1.2.0', date: '2026-10-06', tag: 'orm',
    zh: {
      title: 'ORM 端到端完成',
      summary: '1.2 系列把 ORM 从骨架补成可用：连接池、事务、迁移、关系、JSON、类型映射与两类缓存后端。',
      items: [
        '类型化 Value、Model 派生、CRUD 与 QuerySet 执行层（all / one / count / exists / update / delete）。',
        '三后端连接池 + 事务 + postgres TLS + 预编译语句缓存；事务中途被 drop 的连接会自动回滚。',
        '非破坏性迁移、belongs_to / has_many / m2m、JSON 列、auto_now / soft_delete 生命周期与批量插入。',
        'Redis 断线自动重连；Memcached 后端；CLI migrate init / run 落地。'
      ]
    },
    en: {
      title: 'ORM, end to end',
      summary: 'The 1.2 series turns the ORM from a skeleton into something usable: pools, transactions, migrations, relations, JSON, type mapping, and two cache backends.',
      items: [
        'Typed Value, Model derive, CRUD, and the QuerySet execution layer (all / one / count / exists / update / delete).',
        'Pools for all three backends, transactions, postgres TLS, and a prepared-statement cache; a connection dropped mid-transaction rolls back.',
        'Non-destructive migrations, belongs_to / has_many / m2m, JSON columns, auto_now / soft_delete lifecycle, and bulk inserts.',
        'Redis reconnects on demand; Memcached backends; CLI migrate init / run.'
      ]
    }
  },
  {
    version: '1.1.5', date: '2026-09-25', tag: 'docs',
    zh: {
      title: '更多 crates.io 多语言 README',
      summary: '另 11 个语言的精简版 crates.io README 上线，语言入口改指向这些短文。',
      items: [
        '新增 docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md。',
        'crates.io 页面顶部的语言入口改指向精简版，不再跳到完整仓库 README。'
      ]
    },
    en: {
      title: 'More crates.io READMEs',
      summary: 'Condensed crates.io READMEs for 11 more languages; the switcher now points at them instead of the full repository READMEs.',
      items: [
        'Added docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md.',
        'The crates.io language switcher now points at the condensed files, so every language gets the same short developer-facing document.'
      ]
    }
  },
  {
    version: '1.1.4', date: '2026-09-25', tag: 'docs',
    zh: {
      title: 'crates.io 页面双语',
      summary: 'crates.io 每个 crate 只能渲染一份 README，于是把英文放上半页、中文留在下半页。',
      items: [
        'docs/crates-readme.md 改为中英双语：英文在上，中文在下。',
        '国际开发者首屏即可读到英文，中文内容也不必离开同一页面。'
      ]
    },
    en: {
      title: 'Bilingual crates.io page',
      summary: 'crates.io renders one README per crate, so the page puts English first and keeps Chinese below.',
      items: [
        'docs/crates-readme.md is now bilingual: English first, Chinese below.',
        'English readers get the first screen; Chinese readers stay on the same page.'
      ]
    }
  },
  {
    version: '1.1.3', date: '2026-09-25', tag: 'docs',
    zh: {
      title: '13 语言入口与文档互链',
      summary: 'crates.io 页面新增 13 语言入口，补齐 api / changelog / contributing 的自链接与回链。',
      items: [
        'crates.io 页面新增 13 语言入口，指向仓库内各语言 README。',
        '11 个 api 译本补上指向中文原版的链接。',
        'CHANGELOG.zh.md 与 CONTRIBUTING.zh.md 补上自链接。'
      ]
    },
    en: {
      title: '13-language switcher',
      summary: 'A 13-language switcher on the crates.io page, plus missing self-links and back-links across api, changelog, and contributing docs.',
      items: [
        'The crates.io page gained a 13-language switcher linking to each repository README.',
        'Eleven api translations gained a link back to the Chinese original.',
        'CHANGELOG.zh.md and CONTRIBUTING.zh.md gained their self-links.'
      ]
    }
  },
  {
    version: '1.1.2', date: '2026-09-24', tag: 'docs',
    zh: {
      title: 'crates.io README 与发布脚本',
      summary: '为 crates.io 单独准备 README，发布脚本逐个上传 crate，并修复了主分支测试无法编译的问题。',
      items: [
        'docs/crates-readme.md：安装、可运行示例、feature 开关表与子 crate 索引。',
        'scripts/publish.sh 逐个发布，跳过已上传的 crate，并按 crates.io 的限流重试时间等待。',
        '修复测试套件编译失败、hello 示例 CARGO_BIN_EXE 名称与模板转义断言；所有 crate 补齐 readme / repository 元数据。'
      ]
    },
    en: {
      title: 'crates.io README and publish script',
      summary: 'A dedicated crates.io README, a publish script that uploads crates one by one, and fixes for a test suite that did not compile on main.',
      items: [
        'docs/crates-readme.md: install, a runnable example, a feature-flag table, and a sub-crate index.',
        'scripts/publish.sh publishes one crate at a time, skips uploaded versions, and honours crates.io retry timings.',
        'Fixed the test-suite build, the hello CARGO_BIN_EXE name, and a template-escape assertion; every crate gained readme / repository metadata.'
      ]
    }
  },
  {
    version: '1.0.6', date: '2026-08-07', tag: 'cli',
    zh: {
      title: 'CLI 真正落地',
      summary: '脚手架、代码生成、热重载运行与打包从占位变成真实实现。',
      items: [
        'bee_cli 实现 new、generate controller/model、run --watch 与 pack。',
        '新增 7 个 CLI 脚手架与代码生成单元测试。',
        'bee_rust::init() 由 logs feature 门控；修复 clippy、rustfmt 与 rustdoc 文件名冲突。'
      ]
    },
    en: {
      title: 'The CLI becomes real',
      summary: 'Scaffolding, code generation, hot-reload run, and packing move from placeholders to real implementations.',
      items: [
        'bee_cli implements new, generate controller/model, run --watch, and pack.',
        'Seven new unit tests cover scaffolding and code generation.',
        'bee_rust::init() is now gated behind the logs feature; clippy, rustfmt, and a rustdoc filename collision were fixed.'
      ]
    }
  },
  {
    version: '1.0.4', date: '2026-07-29', tag: 'security',
    zh: {
      title: '安全过滤器',
      summary: '基于 security-rust 的攻击检测过滤器加入 Web 核心，XSS、SQL 注入、命令注入与路径穿越一行开启。',
      items: [
        '新增 SecurityFilter，覆盖 XSS、SQL 注入、命令注入与路径穿越。',
        'bee_rust 与 bee_router 增加 security feature 标志。',
        'bee_template 适配 Rust 2024 edition 的 Tera 原始标识符语法；README 加入打赏支持区块。'
      ]
    },
    en: {
      title: 'Security filter',
      summary: 'A security-rust attack-detection filter joins the web core: XSS, SQL injection, command injection, and path traversal behind one line.',
      items: [
        'Added SecurityFilter with XSS, SQL injection, command injection, and path traversal coverage.',
        'The security feature flag lands in bee_rust and bee_router.',
        'bee_template adapted to Tera raw identifiers under Rust 2024; the README gained a support section.'
      ]
    }
  },
  {
    version: '1.0.3', date: '2026-07-29', tag: 'foundation',
    zh: {
      title: '初始工作区',
      summary: '13 个 crate 的工作区、MVC 路由、ORM 骨架与 KV/Cache trait 抽象一起出现。',
      items: [
        '包含 13 个 crate 的初始工作区结构。',
        '带 Controller trait 与 Router 的 MVC 路由。',
        '带 QuerySet 构建器与 Model 派生宏的 ORM。',
        '带 Redis 与 Memory 后端的 KV/Cache trait 抽象。'
      ]
    },
    en: {
      title: 'Initial workspace',
      summary: 'A 13-crate workspace, MVC routing, the ORM skeleton, and KV/cache trait abstractions arrive together.',
      items: [
        'The initial workspace with 13 crates.',
        'MVC routing with the Controller trait and Router.',
        'An ORM with the QuerySet builder and the Model derive macro.',
        'KV/cache trait abstractions with Redis and Memory backends.'
      ]
    }
  }
];
