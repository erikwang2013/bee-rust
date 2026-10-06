# Журнал изменений

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.2.3] — 2026-10-06

### Исправлено
- `bee_cli`: скаффолд больше не генерирует некомпилируемые проекты — шаблон `new` указывал пакет, который не резолвится (`bee-rust`; публикуется как `bee_rust`), а в шаблоне контроллера не хватало `#[async_trait]` и использовался приватный корневой путь `RouterError`; новый тест под гейтом `BEE_CLI_E2E` создаёт проект и прогоняет `cargo check` от начала до конца
- документация: пример контроллера в api приведён к компилируемому виду (собственная зависимость `async-trait`, `bee_rust::bee_router::context::RouterError`); задокументированы написание `#[bee(crate = "bee_rust::bee_orm")]` для проектов только с `bee_rust`, синтаксис маршрутов `{name}` с правилом пустого префикса `ns` и использование `create()`; исправлен корейский фрагмент в бенгальском api-документе
- `examples/shortlink`: детали внутренних ошибок уходят в лог, а не в тело ответа, валидация URL отклоняет пустой хост и управляющие символы, а `create_link` / `add_tag` теперь используют `Model::create()` (вставка `Click` на горячем пути 302 остаётся `insert()`)

## [1.2.2] — 2026-10-06

### Добавлено
- `bee_orm`: `Model::create() -> Result<Self>` — вставляет и возвращает полный экземпляр, включая назначенный базой первичный ключ (`INSERT … RETURNING *` в sqlite / postgres, `LAST_INSERT_ID()` + перечитывание в mysql); поведение `insert()` не изменилось — по-прежнему возвращает число затронутых строк
- `bee_orm`: атрибут `#[bee(crate = "…")]` — задаёт путь к крейту ORM для раскрытия derive (например, `#[bee(crate = "bee_rust::bee_orm")]`, когда зависимость — только метакрейт `bee_rust`); без атрибута раскрытие остаётся побайтово прежним
- конфигурация сборки docs.rs дополнена для восьми крейтов с публичными модулями под feature (`all-features = true`) — документация backend-feature и таблица атрибутов derive ранее не отображались на docs.rs

## [1.2.1] — 2026-10-06

### Добавлено
- `bee_orm`: типизированные поля даты и Decimal за opt-in feature — `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) и `rust_decimal` (`Decimal`) отображаются в SQL-типы `Date` / `DateTime` / `DateTimeTz` / `Decimal` на всех трёх бэкендах; `bee_rust` пробрасывает их как `orm-chrono` / `orm-rust_decimal` (не в `full`)
- `bee_orm`: честные границы этих типов — sqlite хранит как TEXT (`typeof` = `text`; объявление DECIMAL дало бы NUMERIC-аффинность и молча превратило `"1.50"` в REAL 1.5), mysql сохраняет микросекунды `datetime(6)` / `timestamp(6)` (TIMESTAMP читается как UTC, DATETIME остаётся naive), а pg `numeric` свыше 29 значащих цифр читается как `NULL`; `NaiveTime` / `DateTime<Local>` / `FixedOffset` и наносекунды вне области и требуют `#[bee(sql_type = "…")]`, а с выключенным feature модель не компилируется (E0277)

### Исправлено
- `bee_orm`: подсказка об отсутствующей m2m-связи теперь пишет ident типа (`#[bee(m2m(Author))]`), а не имя таблицы — совет компилируется и для переименованной таблицы
- CI: в `rust-toolchain` добавлен явный вход `toolchain: stable`; два тестовых файла, ломавших `clippy --all-targets` без feature, получили файловые cfg-гейты (bee_cache / bee_kv), а m2m-doctest `bee_orm` стал независим от бэкенда; сквозной тест CLI `migrate` теперь собирает scratch-крейт онлайн (офлайн проходил лишь на тёплом локальном кэше) и запускается в CI через `BEE_CLI_E2E=1`

## [1.2.0] — 2026-10-06

### Добавлено
- `bee_orm`: ORM теперь работает от начала до конца — типизированные параметры `Value` (Null / Bool / Int / Float / Text / Bytes), `#[derive(Model)]` с `#[bee(table / column / pk / auto / ignore)]`, `insert` / `update` / `delete` у модели и выполнение QuerySet (`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm`: пулы соединений для всех трёх бэкендов (`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)`, `get()` → `CheckedConn`, `query` / `execute`, `status()` и транзакции через `begin` / `commit` / `rollback`; соединение, брошенное посреди транзакции, откатывается в sqlite и postgres
- `bee_orm`: новый feature `postgres-tls` (PostgreSQL поверх TLS, встроенные корни Mozilla) и укрепление пулов — таймауты 30 с на ожидание / 10 с на создание и кэш подготовленных выражений в postgres-пуле
- `bee_orm`: жизненный цикл модели — метки времени `#[bee(auto_now_add)]` / `#[bee(auto_now)]`, мягкое удаление `#[bee(soft_delete)]` (delete переключает флаг; `with_deleted()` / `hard_delete()` его обходят), хуки (before/after для insert/update/delete), `Model::insert_many` (пакеты по 999 параметров, без транзакции), а также `QuerySet::filter_in` и агрегаты `sum` / `avg` / `min` / `max`
- `bee_orm`: неразрушающие миграции — `migrate::{create_table, add_missing_columns, sync}` генерируют DDL под диалект (sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`): создают таблицы и добавляют недостающие колонки, никогда не удаляя и не изменяя
- `bee_orm`: связи по внешнему ключу — `#[bee(fk = Target)]` (вместе с `#[bee(sql_type = "…")]`) генерирует DDL внешнего ключа, а `rel::{fk_column_to, belongs_to, children, children_for}` читают связи; соблюдение зависит от бэкенда: postgres и sqlite (bundled-сборка) его обеспечивают, mysql игнорирует inline `REFERENCES` (пробел для round 5+)
- `bee_orm`: связи many-to-many — `#[bee(m2m(Target))]` объявляет связь, `m2m::{attach, detach, related, related_for, related_ids}` читают и пишут её, а `create_table` / `sync` создают таблицу связей (`add_missing_columns` её не трогает)
- `bee_orm`: JSON-колонки — поля `serde_json::Value` отображаются на `TEXT` / `JSONB` / `JSON` в зависимости от бэкенда, с честной семантикой NULL: SQL `NULL` → `None`, сохранённый JSON-документ `null` → `Some(Json::Null)`
- `bee_orm`: внешние ключи на уровне таблицы в MySQL (opt-in) — `MigrateOptions { table_level_fk }` с `sync_with` / `create_table_with` / `add_missing_columns_with`; по умолчанию выключено, а существующие «осиротевшие» строки приводят к ошибке `ADD CONSTRAINT`, а не к тихому пропуску
- `bee_orm`: `Pool::connect_tls_with` для своей `rustls::ClientConfig` плюс реэкспорт `bee_orm::rustls`, чтобы версии всегда совпадали; `connect_tls` по-прежнему использует встроенные корни webpki
- `bee_rust`: четыре пробрасывающих feature — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` пробрасывают бэкенд `bee_orm` через `bee_rust` (ни один не входит в `full`)
- `bee_cli`: `bee-rust migrate init` создаёт `src/bin/bee_migrate.rs` (существующий файл не перезаписывается), `bee-rust migrate run` запускает его через `cargo run --bin bee_migrate`
- `bee_kv` / `bee_cache`: Redis-бэкенды переживают разрыв соединения — внутренний `ConnectionManager` переподключается по требованию (без фонового потока) с экспоненциальным backoff и джиттером (команда на разрыве падает, следующая ждёт нового соединения)
- `bee_kv` / `bee_cache`: бэкенды memcached (`MemcacheStore` в bee_kv, `MemcacheCache` в bee_cache); `incr` создаёт счётчик до дельты, беззнаковые счётчики упираются в 0, а TTL 0 удаляет ключ

### Изменено
- Миграции и связи bee_orm (включая many-to-many), подкоманда `bee-rust migrate`, Redis- / Memcached-бэкенды и пробрасывающие ORM-feature `bee_rust` больше не помечены в документации как планируемые
- драйверы поиска / графов / временных рядов больше не помечены в документации как планируемые или как trait-stub, а как реализованные (opt-in feature); старые примеры исправлены (несуществующие имена типов `ElasticsearchEngine` / `Neo4jDB`, ошибочная схема `bolt://`)

## [1.1.5] — 2026-09-25

### Добавлено
- Сокращённые README для crates.io ещё на 11 языках: `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`. Переключатель языков на странице crates.io теперь ведёт на них, а не на полные README репозитория, — на любом языке читатель получает один и тот же короткий документ для разработчика.

## [1.1.4] — 2026-09-25

### Изменено
- Страница на crates.io (`docs/crates-readme.md`) стала двуязычной: английский сверху, китайский ниже. crates.io отображает только один README на крейт и не умеет переключать язык на странице, поэтому английский теперь виден сразу, а китайский текст остаётся на той же странице.

## [1.1.3] — 2026-09-25

### Добавлено
- На страницу crates.io (`docs/crates-readme.md`) добавлен переключатель на 13 языков со ссылками на README в репозитории

### Исправлено
- `docs/api.*`: в 11 переводах отсутствовала ссылка на китайский оригинал (`api.md`)
- `docs/CHANGELOG.zh.md` и `docs/CONTRIBUTING.zh.md` были единственными файлами в своих наборах без ссылки на себя

## [1.1.2] — 2026-09-24

### Добавлено
- `docs/crates-readme.md`: отдельный README для страниц на crates.io — установка, рабочий пример (взят из `examples/hello`, покрыт его E2E-тестами), таблица feature-флагов и список подкрейтов
- `scripts/publish.sh`: публикует крейты воркспейса по одному, пропускает уже загруженные и при ограничении частоты ждёт время повторной попытки, возвращённое crates.io

### Исправлено
- Набор тестов не компилировался (`cargo test --workspace` падал на main): в интеграционном тесте `bee_router` у `Submit` отсутствовал `Serialize` (он возвращается как `Json`-ответ), у двух хелперов отсутствовал `Router::build()`, в восьми вызовах `status_line` отсутствовал `&`
- Тесты примера `hello` ссылались на неверное имя `CARGO_BIN_EXE` — цель-бинарник сохраняет дефис (`CARGO_BIN_EXE_hello-bee`)
- Тест автоэкранирования шаблонов в `hello` проверял `&#39;`, тогда как tera выводит `&#x27;`; само экранирование было корректным
- Линты Clippy `manual_split_once` и `manual_range_contains` в `bee_cli`; неиспользуемый импорт в интеграционном тесте `bee_router`

### Изменено
- Все крейты теперь содержат метаданные `readme` и `repository` — ранее ни одна страница на crates.io не отображала README, а у шести крейтов вообще не было `repository`
- `examples/hello` помечен `publish = false`: остаётся в воркспейсе как основа E2E-тестов, но больше не публикуется на crates.io

## [1.0.6] — 2026-08-07

### Добавлено
- Реальные реализации `bee_cli`: `new` (скаффолдинг проекта), `generate controller/model`, `run` с горячей перезагрузкой `--watch`, `pack` (сборка release + копирование в `dist/`)
- Модульные тесты CLI для скаффолдинга и генерации кода (7 новых тестов)

### Исправлено
- `bee_rust::init()` теперь за флагом `logs` — сборки с сокращёнными features (например, `--no-default-features --features kv`) снова компилируются
- Линт Clippy `unnecessary_map_or` в `bee_kv::InMemoryKvStore::exists`
- Из `rustfmt.toml` убраны опции только для nightly, которые молча игнорировались на stable; воркспейс теперь проходит `cargo fmt --all --check`
- У бинарника `bee_cli` выставлен `doc = false`, чтобы устранить коллизию имён файлов вывода rustdoc с `bee_rust`
- Порт примера `hello` теперь настраивается через переменную окружения `PORT`

### Изменено
- `bee-rust migrate` сообщает «не реализовано» и завершается с ненулевым кодом (в планах)
- README / README.en обновлены: описано фактическое поведение CLI

## [1.0.4] — 2026-07-29

### Добавлено
- Фильтр обнаружения атак через `security-rust` (27 детекторов)
- `SecurityFilter` с покрытием XSS, SQL-инъекций, инъекций команд, обхода пути
- Флаг `security` в `bee_rust` и `bee_router`

### Изменено
- README обновлён: добавлена документация по feature `security`
- README обновлён: добавлен раздел поддержки проекта (WeChat Pay / Alipay)

### Исправлено
- Синтаксис «сырых» идентификаторов Tera в `bee_template` для Rust 2024 edition

## [1.0.3] — 2026-07-29

### Добавлено
- Начальная структура воркспейса из 13 crate
- MVC-маршрутизация с trait `Controller` и `Router`
- ORM с билдером `QuerySet` и производным макросом `Model`
- Trait-абстракция KV/кэша с бэкендами Redis и Memory
- Управление сессиями с бэкендами Memory/Redis
- Управление конфигурацией с поддержкой INI/YAML/ENV и горячей перезагрузкой
- Рендеринг шаблонов через Tera
- Логирование с интеграцией tracing
- Скаффолдинг и генерация кода в CLI
- Trait-stub движков поиска, графов и временных рядов (драйверы в планах)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
