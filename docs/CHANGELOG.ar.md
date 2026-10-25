# سجل التغييرات

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.5.0] — 2026-10-25

### أُضيف
- `bee_orm`: دعم MySQL 5.7 وMariaDB 10.11+ — تغطّي مصفوفة CI الآن الإصدارات 5.7 / 8.0 / 8.4 / 9.x إضافةً إلى وظيفة MariaDB؛ تدرك اختبارات التكامل نوع الخادم (يُبلّغ MariaDB عن JSON بصيغة LONGTEXT ويفرض `REFERENCES` على مستوى العمود؛ بينما MySQL 5.7/8.x يحلّله ويتجاهله).
- `bee_orm`: يستخدم `Model::create` على MySQL الآن بيانات `columns()` الوصفية للنموذج لتخطّي فحص `information_schema` مع كل insert؛ أما النماذج المكتوبة يدويًا دون بيانات وصفية فتحتفظ بالفحص القديم. هذا التلميح مرجعي — انحراف المخطط عن النموذج غير مدعوم.
- `bee_orm::shard`: تكشف `ShardedPools::{pools, pool, len, is_empty}` عن التقسيمات المُهيّأة للعمليات؛ ويمرّر `ShardHandle` تلميح الزيادة التلقائية.
- `bee_kv` / `bee_cache`: مقاييس تجمّع `status()`؛ و`RedisOptions` العامة (`max_size`، `response_timeout`) مع `with_options`؛ ويجدّد `RedisLock::try_lock_with_watchdog` مدة TTL حتى release/drop (يجب أن تكون مدة watchdog TTL ‏30ms على الأقل).
- داخليًا: بوابة CI لاتساق التوثيق (`scripts/check-docs.sh`)، وسكربت لتصيير المعاينة الاجتماعية، وتركيبات الاختبار المشتركة المستخرجة من `tests/mod.rs`.

## [1.4.0] — 2026-10-25

### أُضيف
- `bee_orm`: توافق MySQL 8.0 / 8.4 / 9.x، مُتحقَّق منه عبر مصفوفة CI. يفرض MySQL 9 `REFERENCES` على مستوى العمود (8.0/8.4 يحلّله ويتجاهله)؛ أنشئ الجداول المُشار إليها أولاً.
- `bee_orm`: `shard::{ShardedPools, HashRouter, ShardRouter}` — تقسيم تجزئة يتطلّب مفتاحًا لقواعد البيانات والجداول؛ لا استعلامات أو معاملات عبر التقسيمات.
- `bee_kv` / `bee_cache`: تجمّع اتصالات Redis حقيقي (`deadpool-redis`) و`bee_kv::RedisLock` (`SET NX PX` + تحرير Lua).

## [1.3.0] — 2026-10-13

### أُصلح
- `bee_tsdb` / `bee_graph`: تمكين ميزة backend واحدة بمفردها لم يكن يُترجم — كان المساعد المشترك `http_client` يقيم داخل وحدتي `influxdb` / `neo4j` المحكومتين بالميزات، فتفشل البناءة التي تُفعّل فقط `iotdb` / `questdb` / `arangodb` / `nebulagraph` بخطأ unresolved import؛ صار الآن في `crate::http` الخاص بكل crate (مع بوابة cfg على الـbackend التي تستخدمه فقط)، وكل ميزة backend منفردة تُترجم وحدها — أُعيد تشغيل مصفوفة الميزات بندًا بندًا
- `bee_router`: ميزة `security` الاختيارية انتقلت إلى **security-rust 4.0.0** — يحتفظ `Scanner` / `DetectionResult` / `AttackCategory` / `Severity` بشكل 1.x، دون أي تغيير في مصدر bee_router؛ توسّعت القدرات: 32 كاشفًا مدمجًا (الجديد: `format_string` و`log4shell` وHPP و`formula_injection` وreDoS)، والكاشفات صارت مصنّفة قوية/ضعيفة (الإشارات الضعيفة مثبّتة عند `Severity::Low`)، وأُضيف `Scanner::assess`. لا يزال المرشّح يحجب عند أي اكتشاف، لذا **صارت الإشارات الضعيفة (مثل معامل استعلام مكرر) محجوبة أيضًا** — لم يعد هناك منفذ لحقن scanner مخصّص؛ للسماح بها ابنِ `Filter` خاصًا بك حول `SecurityFilter::scanner()`. لم يعد الرسم البياني للاعتماديات يحمل سوى security-rust 4.0.0 واحدة بدل 1.x إلى جانب 4.x؛ والوثائق حُدّثت
- دمج التكرارات: احتفظ كل من `bee_tsdb` / `bee_graph` / `bee_search` بنسخة واحدة من مساعدات الاتصال وخرائط الأخطاء (اسم الـbackend يمرّره الآن المستدعي، ونصوص الأخطاء كما هي؛ وانتقل `aggregation_fn` إلى lib في `bee_tsdb`)، وضُمّ `backtick` / `esc` في `bee_graph` إلى وحدة `http` لديه، واستخرج `bee_orm` الدالتين `dedup_pk_values` / `index_rows_by` وبند المفتاح الأجنبي في `migrate`، ويعيد `bee_session::refresh` استخدام `load`، واستوعب `watch_path` في `bee_config` مسار نداء `wait_for_change` المتبقي؛ واستعادت رسائل أخطاء `bee_search` اسم الـbackend، وصار stub اختبار `bee_kv` خاصًّا باختباراته
- إزالة واجهات عامة (ومن هنا الإصدار الفرعي): `Context::set_params` / `param`، و`SecurityFilter::with_scanner`، و`Traversal::edge_labels` → `edge_label: Option<String>` (الـbackends الثلاثة كانت أصلًا تأخذ أول وسم فقط — السلوك دون تغيير)، و`GraphError::EdgeNotFound`، و`ConfigError::ParseError` / `TypeMismatch`، و`bee_config::watch_path_timeout`، و`bee_logs::Logger::async_` (مفتاح لم يقرأه `init()` قط) — جميعها بلا أي مستدعٍ
- إزالة اعتماديات ميتة: `bee_router` (`tracing` / `http` / `bee_config` / `bee_logs`)، و`bee_session` (`async-trait`)، و`bee_config_macro` (`proc-macro2`)، وتكرار `dev` `serde` لدى `bee_tsdb` / `bee_graph` / `bee_search`

## [1.2.3] — 2026-10-06

### أُصلح
- `bee_cli`: لم يعد السقالب يُنشئ مشاريع لا تُترجم — قالب `new` كان يسمّي حزمة لا تُحلّ (`bee-rust`؛ المنشورة باسم `bee_rust`) وقالب المتحكم كان ينقصه `#[async_trait]` ويستخدم مسار الجذر الخاص لـ`RouterError`؛ اختبار جديد محكوم بـ`BEE_CLI_E2E` ينشئ مشروعًا ويشغّل عليه `cargo check` من البداية إلى النهاية
- الوثائق: صار مثال المتحكم في api بالصيغة القابلة للترجمة (اعتماد `async-trait` خاص به، `bee_rust::bee_router::context::RouterError`)؛ وأُضيف توثيق كتابة `#[bee(crate = "bee_rust::bee_orm")]` لمشاريع `bee_rust` وحدها، وصيغة المسار `{name}` مع قاعدة البادئة الفارغة لـ`ns`، واستخدام `create()`؛ وصُحّح جزء كوري تسرّب إلى وثيقة api البنغالية
- `examples/shortlink`: تفاصيل الأخطاء الداخلية تذهب إلى السجل بدل جسم الاستجابة، والتحقق من URL يرفض المضيف الفارغ ومحارف التحكم، و`create_link` / `add_tag` تستخدمان الآن `Model::create()` (إدراج `Click` في المسار الساخن 302 يبقى `insert()`)

## [1.2.2] — 2026-10-06

### أُضيف
- `bee_orm`: `Model::create() -> Result<Self>` — يُدرج ويعيد النسخة الكاملة مع المفتاح الأساسي الذي عيّنته قاعدة البيانات (`INSERT … RETURNING *` في sqlite / postgres، و`LAST_INSERT_ID()` + إعادة قراءة في mysql)؛ و`insert()` كما هو ويعيد عدد الصفوف المتأثرة
- `bee_orm`: السمة `#[bee(crate = "…")]` — تحدد مسار كريت ORM المستخدم في التوسيع (مثل `#[bee(crate = "bee_rust::bee_orm")]` عند الاعتماد على الميتا-كريت `bee_rust` وحده)؛ وبدونها يبقى التوسيع مطابقًا بايت ببايت
- استُكملت إعدادات بناء docs.rs للكُريتات الثمانية ذات الوحدات العامة المحكومة بـ feature (`all-features = true`) — كانت صفحات توثيق feature الخلفيات وجدول سمات derive غير ظاهرة على docs.rs

## [1.2.1] — 2026-10-06

### أُضيف
- `bee_orm`: حقول التاريخ / Decimal خلف features اختيارية — `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) و`rust_decimal` (`Decimal`) تُعيَّن إلى أنواع SQL ‏`Date` / `DateTime` / `DateTimeTz` / `Decimal` في الواجهات الثلاث؛ و`bee_rust` يمررها كـ `orm-chrono` / `orm-rust_decimal` (ليست ضمن `full`)
- `bee_orm`: حدود صريحة لهذه الأنواع — sqlite يخزن TEXT (`typeof` = `text`؛ إعلان DECIMAL يمنح ألفة NUMERIC ويحوّل `"1.50"` بصمت إلى REAL 1.5)، وmysql يحفظ ميكروثواني `datetime(6)` / `timestamp(6)` (TIMESTAMP يُقرأ كـ UTC وDATETIME يبقى naive)، وفي pg يُقرأ `numeric` فوق 29 رقمًا معنويًا كـ `NULL`؛ و`NaiveTime` / `DateTime<Local>` / `FixedOffset` / النانوثانية خارج النطاق وتتطلب `#[bee(sql_type = "…")]`، ومع إيقاف feature لا يُترجم النموذج (E0277)

### أُصلح
- `bee_orm`: تنبيه العلاقة m2m المفقودة يكتب الآن ident النوع (`#[bee(m2m(Author))]`) بدل اسم الجدول، فيبقى الاقتراح قابلًا للترجمة حتى مع جدول مُعاد تسميته
- CI: أُضيف الإدخال الصريح `toolchain: stable` إلى `rust-toolchain`؛ وملفا اختبار كانا يكسران `clippy --all-targets` بدون features حصلا على بوابات cfg على مستوى الملف (bee_cache / bee_kv)، وصار doctest الـ m2m في `bee_orm` مستقلًا عن الواجهة؛ واختبار `migrate` الشامل في CLI يبني الآن scratch crate عبر الشبكة (الوضع دون اتصال كان ينجح فقط مع ذاكرة محلية دافئة) ويعمل في CI عبر `BEE_CLI_E2E=1`

## [1.2.0] — 2026-10-06

### أُضيف
- `bee_orm`: أصبح الـ ORM يعمل من البداية إلى النهاية — معاملات `Value` مُنمَّطة (Null / Bool / Int / Float / Text / Bytes)، و`#[derive(Model)]` يدعم `#[bee(table / column / pk / auto / ignore)]`، و`insert` / `update` / `delete` على النموذج، وتنفيذ `QuerySet` (`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm`: تجمّعات اتصالات للخلفيات الثلاث (`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)` و`get()` → `CheckedConn` و`query` / `execute` و`status()`، مع معاملات عبر `begin` / `commit` / `rollback`؛ الاتصال المهجور أثناء المعاملة يُرجَع (rollback) في sqlite وpostgres
- `bee_orm`: ميزة جديدة `postgres-tls` (PostgreSQL عبر TLS بجذور Mozilla المضمّنة) وتقوية التجمّعات — مهلة انتظار 30 ثانية / إنشاء 10 ثوانٍ وذاكرة تخزين مؤقت للعبارات المُجهَّزة في تجمّع postgres
- `bee_orm`: دورة حياة النموذج — طوابع زمنية عبر `#[bee(auto_now_add)]` / `#[bee(auto_now)]`، وحذف منطقي عبر `#[bee(soft_delete)]` (يصبح delete قلب العلامة، و`with_deleted()` / `hard_delete()` يتجاوزانه)، وخطافات insert/update/delete، و`Model::insert_many` (دفعات من 999 معاملًا، بلا معاملة)، مع `QuerySet::filter_in` وتجميعات `sum` / `avg` / `min` / `max`
- `bee_orm`: ترحيلات غير مُدمِّرة — تولّد `migrate::{create_table, add_missing_columns, sync}` عبارات DDL حسب اللهجة (sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`)، فتنشئ الجداول وتضيف الأعمدة الناقصة دون حذف أو تعديل أبدًا
- `bee_orm`: علاقات المفاتيح الأجنبية — ينتج `#[bee(fk = Target)]` (مع `#[bee(sql_type = "…")]`) تعريف FK، وتقرؤها `rel::{fk_column_to, belongs_to, children, children_for}`؛ يعتمد الإلزام على الخلفية: postgres وsqlite (بناء bundled) يفرضان المرجع، بينما mysql يتجاهل `REFERENCES` المضمّن (فجوة في round 5+)
- `bee_orm`: علاقات متعدد إلى متعدد — `#[bee(m2m(Target))]` تعلن العلاقة، و`m2m::{attach, detach, related, related_for, related_ids}` تقرؤها وتكتبها، وينشئ `create_table` / `sync` جدول الربط (`add_missing_columns` لا يمسه)
- `bee_orm`: أعمدة JSON — تُعيَّن حقول `serde_json::Value` إلى `TEXT` / `JSONB` / `JSON` حسب الخلفية، بدلالات NULL صادقة: SQL `NULL` → `None`، ومستند JSON `null` المخزَّن → `Some(Json::Null)`
- `bee_orm`: مفاتيح أجنبية على مستوى الجدول في MySQL (opt-in) — `MigrateOptions { table_level_fk }` مع `sync_with` / `create_table_with` / `add_missing_columns_with`؛ معطّلة افتراضيًا، والصفوف اليتيمة الموجودة مسبقًا تُفشل `ADD CONSTRAINT` بدل تخطيها بصمت
- `bee_orm`: `Pool::connect_tls_with` لإعداد `rustls::ClientConfig` مخصص، مع إعادة تصدير `bee_orm::rustls` ليتطابق الإصدار دائمًا؛ ويُبقي `connect_tls` على جذور webpki المضمّنة
- `bee_rust`: أربع features للتمرير — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` تمرّر خلفية `bee_orm` عبر `bee_rust` (ولا واحدة منها في `full`)
- `bee_cli`: `bee-rust migrate init` يولّد `src/bin/bee_migrate.rs` (يرفض استبدال ملف موجود)، و`bee-rust migrate run` يشغّله عبر `cargo run --bin bee_migrate`
- `bee_kv` / `bee_cache`: خلفيات Redis تنجو من الاتصالات المقطوعة — يعيد `ConnectionManager` الداخلي الاتصال عند الطلب (بدون خيط في الخلفية) بتراجع أُسّي مع jitter (الأمر الذي يصادف القطع يفشل، والتالي ينتظر الاتصال الجديد)
- `bee_kv` / `bee_cache`: خلفيات memcached (`MemcacheStore` في bee_kv و`MemcacheCache` في bee_cache)؛ ينشئ `incr` العدّاد قبل الفرق، وتتوقف العدّادات غير الموقّعة عند 0، وTTL 0 يحذف المفتاح

### تغيّر
- لم تعد تُوسم في الوثائق بأنها مخطط لها: ترحيلات bee_orm وعلاقاته (بما فيها many-to-many)، وأمر `bee-rust migrate` الفرعي، وخلفيات Redis / Memcached، وfeatures تمرير ORM في `bee_rust`
- لم تعد مشغلات search / graph / tsdb في الوثائق موسومة كمخططة أو trait-stub، بل كمُنفَّذة (ميزات opt-in)، وصُحّحت الأمثلة القديمة (أسماء أنواع غير موجودة `ElasticsearchEngine` / `Neo4jDB`، ومخطط `bolt://` خاطئ)

## [1.1.5] — 2026-09-25

### أُضيف
- ملفات README مختصرة لـ crates.io بإحدى عشرة لغة إضافية: `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`. أصبح مبدّل اللغة في صفحة crates.io يشير إليها بدل ملفات README الكاملة في المستودع، فيحصل كل لغة على المستند القصير نفسه الموجّه للمطوّرين.

## [1.1.4] — 2026-09-25

### تغيّر
- أصبحت صفحة crates.io ‏(`docs/crates-readme.md`) ثنائية اللغة: الإنجليزية في الأعلى والصينية في الأسفل. يعرض crates.io ملف README واحدًا فقط لكل حزمة ولا يوفر مبدّل لغة داخل الصفحة، لذا تظهر الإنجليزية في الشاشة الأولى ويبقى النص الصيني في الصفحة نفسها.

## [1.1.3] — 2026-09-25

### أُضيف
- أصبحت صفحة crates.io (`docs/crates-readme.md`) تتضمن مبدّلًا لـ 13 لغة يرتبط بملفات README في المستودع

### أُصلح
- `docs/api.*`: كانت 11 ترجمة تفتقر إلى رابط الأصل الصيني (`api.md`)
- كان `docs/CHANGELOG.zh.md` و`docs/CONTRIBUTING.zh.md` الملفين الوحيدين في مجموعتيهما بدون رابط ذاتي

## [1.1.2] — 2026-09-24

### أُضيف
- `docs/crates-readme.md`: ملف README مخصّص لصفحات crates.io — التثبيت، ومثال قابل للتشغيل (مأخوذ من `examples/hello`، وتغطّيه اختبارات E2E الخاصة به)، وجدول أعلام feature، وفهرس الحزم الفرعية
- `scripts/publish.sh`: ينشر حزم مساحة العمل واحدة تلو الأخرى، ويتجاوز ما سبق رفعه، وعند تقييد المعدّل ينتظر وقت إعادة المحاولة الذي يعيده crates.io

### أُصلح
- لم تكن مجموعة الاختبارات قابلة للترجمة (`cargo test --workspace` كان يفشل على main): في اختبار التكامل الخاص بـ `bee_router` كان `Serialize` مفقودًا على `Submit` (يُعاد كاستجابة `Json`)، و`Router::build()` مفقودًا في دالتين مساعدتين، و`&` مفقودًا في ثمانية استدعاءات لـ `status_line`
- اختبارات مثال `hello` أشارت إلى اسم `CARGO_BIN_EXE` خاطئ — هدف الملف التنفيذي يحتفظ بالشرطة (`CARGO_BIN_EXE_hello-bee`)
- اختبار التهريب التلقائي للقوالب في `hello` كان يتحقق من `&#39;` بينما tera يُخرج `&#x27;`؛ سلوك التهريب نفسه كان صحيحًا
- تنبيهات Clippy ‏`manual_split_once` و`manual_range_contains` في `bee_cli`؛ واستيراد غير مستخدم في اختبار التكامل الخاص بـ `bee_router`

### تغيّر
- جميع الحزم تحمل الآن بيانات `readme` و`repository` الوصفية — سابقًا لم تكن أي صفحة على crates.io تعرض README، وست حزم كانت تفتقر إلى `repository` تمامًا
- تم تعليم `examples/hello` بـ `publish = false`: يبقى في مساحة العمل كمنصّة اختبارات E2E لكنه لم يعد يُنشر على crates.io

## [1.0.6] — 2026-08-07

### أُضيف
- تنفيذات حقيقية لـ `bee_cli`: `new` (سقالات المشروع)، `generate controller/model`، `run` مع إعادة تحميل ساخنة `--watch`، `pack` (بناء release + نسخ إلى `dist/`)
- اختبارات وحدة CLI للسقالات وتوليد الكود (7 اختبارات جديدة)

### أُصلح
- `bee_rust::init()` أصبح الآن خلف feature `logs` — البناءات المختزلة الميزات (مثل `--no-default-features --features kv`) تُترجم مجددًا
- تحذير Clippy `unnecessary_map_or` في `bee_kv::InMemoryKvStore::exists`
- أُزيلت خيارات `rustfmt.toml` الخاصة بـ nightly التي كانت تُتجاهل بصمت على stable؛ المكدس الآن يجتاز `cargo fmt --all --check`
- ثنائي `bee_cli` الآن `doc = false` لإزالة تضارب اسم مخرجات rustdoc مع `bee_rust`
- منفذ مثال `hello` أصبح قابلاً للتكوين عبر متغير البيئة `PORT`

### تغيّر
- `bee-rust migrate` يبلغ "غير منفذ" ويخرج برمز خروج غير صفري (مخطط)
- تحديث README / README.en ليصفا سلوك CLI الفعلي

## [1.0.4] — 2026-07-29

### أُضيف
- فلتر كشف هجمات الأمان عبر `security-rust` (27 أداة كشف)
- `SecurityFilter` بتغطية XSS و SQL injection و command injection و path traversal
- feature flag `security` في `bee_rust` و `bee_router`

### تغيّر
- تحديث README بتوثيق ميزة الأمان
- تحديث README بقسم دعم الدفع (WeChat Pay / Alipay)

### أُصلح
- صيغة Tera للمعرّفات الأولية في `bee_template` لإصدار Rust 2024

## [1.0.3] — 2026-07-29

### أُضيف
- بنية workspace أولية بـ 13 crate
- توجيه MVC عبر `Controller` trait و `Router`
- ORM عبر منشئ `QuerySet` وماكرو اشتقاق `Model`
- تجريد traits لـ KV/Cache مع خلفيات Redis و Memory
- إدارة الجلسات مع خلفيات Memory/Redis
- إدارة التكوين بدعم INI/YAML/ENV وإعادة تحميل ساخنة
- تقديم القوالب عبر Tera
- السجلات بتكامل tracing
- سقالات CLI وتوليد الكود
- trait stubs لمحركات Search و Graph والسلاسل الزمنية (برامج التشغيل مخطط لها)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
