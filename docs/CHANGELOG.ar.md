# سجل التغييرات

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
