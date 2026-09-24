# Журнал изменений

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
