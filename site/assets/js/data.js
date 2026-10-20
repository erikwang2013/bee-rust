/* Bee Rust site — pet expressions + feature evolution milestones */
window.BeeData = {
  pets: [
    { file: 'smile',      zh: { name: '笑',     scene: '测试全绿的时候' },       en: { name: 'Smile',      scene: 'when the tests are green' } },
    { file: 'joy',        zh: { name: '欢乐',   scene: '新版本发布' },           en: { name: 'Joy',        scene: 'release day' } },
    { file: 'cry',        zh: { name: '哭',     scene: '迁移把列删了' },         en: { name: 'Cry',        scene: 'a migration dropped a column' } },
    { file: 'depressed',  zh: { name: '沮丧',   scene: '又遇到 flaky test' },    en: { name: 'Depressed',  scene: 'another flaky test' } },
    { file: 'terrified',  zh: { name: '惊恐',   scene: '依赖大版本升级' },       en: { name: 'Terrified',  scene: 'a major dependency bump' } },
    { file: 'angry',      zh: { name: '愤怒',   scene: '有人改了锁文件' },       en: { name: 'Angry',      scene: 'someone touched the lockfile' } },
    { file: 'coquettish', zh: { name: '撒娇',   scene: '想要一个新 feature' },   en: { name: 'Coquettish', scene: 'asking for a new feature' } },
    { file: 'cute',       zh: { name: '卖萌',   scene: '文档写完了' },           en: { name: 'Cute',       scene: 'the docs are done' } },
    { file: 'pleasing',   zh: { name: '讨好',   scene: 'code review 通过' },     en: { name: 'Pleasing',   scene: 'code review approved' } },
    { file: 'knocking',   zh: { name: '敲打',   scene: '等 CI 跑完' },           en: { name: 'Knocking',   scene: 'waiting for CI' } },
    { file: 'facepalm',   zh: { name: '捂脸',   scene: '忘了开 --features' },    en: { name: 'Facepalm',   scene: 'forgot the feature flag' } },
    { file: 'tongue',     zh: { name: '吐舌头', scene: 'bug 是你自己写的' },     en: { name: 'Tongue out', scene: 'the bug was yours all along' } }
  ],
  milestones: [
    {
      version: '1.0', date: '2026-07', pet: 'smile',
      zh: { title: '地基成形', desc: '13 个 crate 的工作区、MVC 路由、Controller、QuerySet、KV/Cache trait 与安全过滤器落地。' },
      en: { title: 'Foundation', desc: 'A 13-crate workspace, MVC routing, Controller, QuerySet, KV/cache traits, and the security filter.' }
    },
    {
      version: '1.1', date: '2026-09', pet: 'cute',
      zh: { title: '文档与国际化', desc: '13 语言 README、crates.io 专用页、中英双语页面与各语言文档互链。' },
      en: { title: 'Docs and i18n', desc: 'A 13-language README, a dedicated crates.io page, bilingual docs, and cross-links between translations.' }
    },
    {
      version: '1.2', date: '2026-10', pet: 'joy',
      zh: { title: 'ORM 拼图完成', desc: 'CRUD、三后端连接池、事务、迁移、关系、JSON、chrono/Decimal、Redis/Memcached 与 migrate CLI。' },
      en: { title: 'ORM complete', desc: 'CRUD, three backends with pools, transactions, migrations, relations, JSON columns, chrono/Decimal, Redis/Memcached, and the migrate CLI.' }
    },
    {
      version: '1.3', date: '2026-10', pet: 'knocking',
      zh: { title: '硬化与收敛', desc: '单后端特性矩阵修复、security-rust 4（32 个检测器）、重复实现合并与死依赖清理。' },
      en: { title: 'Hardening', desc: 'Single-backend feature matrix fixed, security-rust 4 with 32 detectors, duplicate implementations merged, dead dependencies removed.' }
    }
  ]
};
