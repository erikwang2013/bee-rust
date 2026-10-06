# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.2.3] — 2026-10-06

### Behoben
- `bee_cli`: das Scaffold erzeugt keine nicht kompilierbaren Projekte mehr — die `new`-Vorlage nannte ein Paket, das sich nicht auflösen lässt (`bee-rust`; veröffentlicht als `bee_rust`), und der Controller-Vorlage fehlte `#[async_trait]`, außerdem nutzte sie den privaten `RouterError`-Wurzelpfad; ein neuer `BEE_CLI_E2E`-gated Test scaffoldet ein Projekt und `cargo check`t es Ende-zu-Ende
- Doku: das api-Controller-Beispiel liegt jetzt in kompilierbarer Form vor (eigene `async-trait`-Abhängigkeit, `bee_rust::bee_router::context::RouterError`); dokumentiert sind außerdem die Schreibweise `#[bee(crate = "bee_rust::bee_orm")]` für reine `bee_rust`-Projekte, die `{name}`-Routensyntax mit der `ns`-Leerpräfix-Regel und die `create()`-Verwendung; im bengalischen api-Dokument wurde ein koreanisches Fragment entfernt
- `examples/shortlink`: interne Fehlerdetails gehen ins Log statt in den Response-Body, die URL-Validierung lehnt leere Host-Teile und Steuerzeichen ab, und `create_link` / `add_tag` nutzen jetzt `Model::create()` (der `Click`-Insert im 302-Hot-Path bleibt `insert()`)

## [1.2.2] — 2026-10-06

### Neu hinzugefügt
- `bee_orm`: `Model::create() -> Result<Self>` — fügt ein und liefert die vollständige Instanz zurück, inklusive des von der Datenbank vergebenen Primärschlüssels (`INSERT … RETURNING *` bei sqlite / postgres, `LAST_INSERT_ID()` + Rücklesen bei mysql); `insert()` bleibt unverändert und liefert weiterhin die betroffenen Zeilen
- `bee_orm`: das Attribut `#[bee(crate = "…")]` — zeigt dem Derive den Pfad zum ORM-Crate (z. B. `#[bee(crate = "bee_rust::bee_orm")]`, wenn nur das Metacrate `bee_rust` eine Abhängigkeit ist); ohne Angabe bleibt die Expansion byte-identisch
- docs.rs-Build-Konfiguration für die acht Crates mit feature-gesteuerten öffentlichen Modulen vervollständigt (`all-features = true`) — Backend-Feature-Doku und die Derive-Attributtabelle waren auf docs.rs bisher unsichtbar

## [1.2.1] — 2026-10-06

### Neu hinzugefügt
- `bee_orm`: typisierte Datums-/Decimal-Felder hinter Opt-in-Features — `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) und `rust_decimal` (`Decimal`) bilden auf die SQL-Typen `Date` / `DateTime` / `DateTimeTz` / `Decimal` in allen drei Backends ab; `bee_rust` leitet sie als `orm-chrono` / `orm-rust_decimal` weiter (nicht in `full`)
- `bee_orm`: ehrliche Grenzen dieser Typen — sqlite speichert als TEXT (`typeof` = `text`; eine DECIMAL-Deklaration bekäme NUMERIC-Affinität und machte `"1.50"` still zu REAL 1.5), mysql behält Mikrosekunden in `datetime(6)` / `timestamp(6)` (TIMESTAMP liest als UTC, DATETIME bleibt naiv), und ein pg `numeric` über 29 signifikante Stellen liest als `NULL`; `NaiveTime` / `DateTime<Local>` / `FixedOffset` / Nanosekunden sind außerhalb des Umfangs und brauchen `#[bee(sql_type = "…")]`, und ohne Feature kompiliert das Modell nicht (E0277)

### Behoben
- `bee_orm`: der m2m-Hinweis bei fehlender Beziehung nennt jetzt den Typ-Ident (`#[bee(m2m(Author))]`) statt des Tabellennamens — der Vorschlag kompiliert auch bei umbenannter Tabelle
- CI: `rust-toolchain` erhielt den expliziten `toolchain: stable`-Input; die zwei Testdateien, die ohne Features `clippy --all-targets` brachen, bekamen Datei-Cfg-Gates (bee_cache / bee_kv) und der `bee_orm`-m2m-Doctest wurde backend-agnostisch; der CLI-`migrate`-End-to-End-Test baut sein Scratch-Crate jetzt online (offline klappte nur mit warmem lokalem Cache) und läuft in CI über `BEE_CLI_E2E=1`

## [1.2.0] — 2026-10-06

### Neu hinzugefügt
- `bee_orm`: Das ORM läuft jetzt durchgängig — typisierte `Value`-Parameter (Null / Bool / Int / Float / Text / Bytes), `#[derive(Model)]` mit `#[bee(table / column / pk / auto / ignore)]`, `insert` / `update` / `delete` am Modell und `QuerySet`-Ausführung (`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm`: Verbindungspools für alle drei Backends (`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)`, `get()` → `CheckedConn`, `query` / `execute`, `status()` und Transaktionen über `begin` / `commit` / `rollback`; eine mitten in der Transaktion verworfene Verbindung wird bei sqlite und postgres zurückgerollt
- `bee_orm`: neues Feature `postgres-tls` (PostgreSQL über TLS, gebündelte Mozilla-Roots) und gehärtete Pools — 30 s Warte- / 10 s Aufbau-Timeout sowie ein Prepared-Statement-Cache im postgres-Pool
- `bee_orm`: Modell-Lebenszyklus — Zeitstempel über `#[bee(auto_now_add)]` / `#[bee(auto_now)]`, Soft-Delete über `#[bee(soft_delete)]` (delete setzt die Markierung; `with_deleted()` / `hard_delete()` umgehen sie), Lifecycle-Hooks (before/after für insert/update/delete), `Model::insert_many` (Chunks zu 999 Parametern, nicht transaktional) sowie `QuerySet::filter_in` und Aggregate `sum` / `avg` / `min` / `max`
- `bee_orm`: nicht-destruktive Migrationen — `migrate::{create_table, add_missing_columns, sync}` erzeugen dialektgerechtes DDL (sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`) und legen Tabellen sowie fehlende Spalten an, ohne je zu löschen oder zu ändern
- `bee_orm`: Fremdschlüssel-Beziehungen — `#[bee(fk = Target)]` (mit `#[bee(sql_type = "…")]`) erzeugt FK-DDL, `rel::{fk_column_to, belongs_to, children, children_for}` lesen sie; die Durchsetzung hängt vom Backend ab: postgres und sqlite (bundled-Build) erzwingen die Referenz, mysql ignoriert inline `REFERENCES` (Lücke für round 5+)
- `bee_orm`: Many-to-many-Beziehungen — `#[bee(m2m(Target))]` deklariert die Beziehung, `m2m::{attach, detach, related, related_for, related_ids}` lesen und schreiben sie, und `create_table` / `sync` legen die Join-Tabelle an (`add_missing_columns` rührt sie nie an)
- `bee_orm`: JSON-Spalten — `serde_json::Value`-Felder werden je Backend auf `TEXT` / `JSONB` / `JSON` abgebildet, mit ehrlicher NULL-Semantik: SQL `NULL` → `None`, ein gespeichertes JSON-`null`-Dokument → `Some(Json::Null)`
- `bee_orm`: opt-in Tabellen-Fremdschlüssel für MySQL — `MigrateOptions { table_level_fk }` mit `sync_with` / `create_table_with` / `add_missing_columns_with`; standardmäßig aus, und vorhandene verwaiste Zeilen lassen `ADD CONSTRAINT` scheitern statt still übersprungen zu werden
- `bee_orm`: `Pool::connect_tls_with` für eine eigene `rustls::ClientConfig`, dazu ein `bee_orm::rustls`-Re-Export, damit die Version immer passt; `connect_tls` behält die gebündelten webpki-Roots
- `bee_rust`: vier Weiterleitungs-Features — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` reichen ein `bee_orm`-Backend über `bee_rust` durch (keines davon in `full`)
- `bee_cli`: `bee-rust migrate init` erzeugt `src/bin/bee_migrate.rs` (überschreibt eine vorhandene Datei nicht), `bee-rust migrate run` führt sie per `cargo run --bin bee_migrate` aus
- `bee_kv` / `bee_cache`: Die Redis-Backends überstehen abgerissene Verbindungen — der zugrunde liegende `ConnectionManager` verbindet bei Bedarf neu (kein Hintergrund-Thread) mit exponentiellem Backoff und Jitter (der Befehl am Bruch schlägt fehl, der nächste wartet auf die frische Verbindung)
- `bee_kv` / `bee_cache`: memcached-Backends (`MemcacheStore` in bee_kv, `MemcacheCache` in bee_cache); `incr` legt den Zähler vor dem Delta an, vorzeichenlose Zähler bleiben bei 0 stehen, TTL 0 löscht den Schlüssel

### Geändert
- bee_orm-Migrationen, -Beziehungen (inklusive many-to-many), der `bee-rust migrate`-Unterbefehl, die Redis- / Memcached-Backends und die ORM-Weiterleitungs-Features von `bee_rust` sind in den Docs nicht mehr als geplant markiert
- die Treiber für Suche / Graph / Zeitreihen sind in den Docs nicht mehr als geplant bzw. Trait-Stub markiert, sondern als implementiert (opt-in-Features); die alten Beispiele wurden korrigiert (nicht existierende Typnamen `ElasticsearchEngine` / `Neo4jDB`, falsches `bolt://`)

## [1.1.5] — 2026-09-25

### Neu hinzugefügt
- Gekürzte crates.io-READMEs für 11 weitere Sprachen: `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`. Die Sprachauswahl auf der crates.io-Seite verweist nun auf diese statt auf die vollständigen READMEs des Repositories — jede Sprache erhält dasselbe kurze, entwicklerorientierte Dokument.

## [1.1.4] — 2026-09-25

### Geändert
- Die crates.io-Seite (`docs/crates-readme.md`) ist jetzt zweisprachig: Englisch oben, Chinesisch darunter. crates.io rendert nur ein README pro Crate und bietet keinen Sprachumschalter auf der Seite, daher steht Englisch nun auf dem ersten Bildschirm, während der chinesische Text auf derselben Seite bleibt.

## [1.1.3] — 2026-09-25

### Neu hinzugefügt
- Die crates.io-Seite (`docs/crates-readme.md`) enthält jetzt eine Sprachauswahl für 13 Sprachen mit Links zu den READMEs im Repository

### Behoben
- `docs/api.*`: In 11 Übersetzungen fehlte der Link zum chinesischen Original (`api.md`)
- `docs/CHANGELOG.zh.md` und `docs/CONTRIBUTING.zh.md` waren die einzigen Dateien ihrer Familien ohne Selbstlink

## [1.1.2] — 2026-09-24

### Neu hinzugefügt
- `docs/crates-readme.md`: ein eigenes README für die crates.io-Seiten — Installation, ein lauffähiges Beispiel (aus `examples/hello`, durch dessen E2E-Tests abgedeckt), eine Feature-Flag-Tabelle und eine Übersicht der Sub-Crates
- `scripts/publish.sh`: veröffentlicht die Crates des Workspace einzeln, überspringt bereits hochgeladene und wartet bei Ratenbegrenzung auf die von crates.io zurückgegebene Wiederholungszeit

### Behoben
- Die Testsuite ließ sich nicht kompilieren (`cargo test --workspace` schlug auf main fehl): Im Integrationstest von `bee_router` fehlte `Serialize` bei `Submit` (wird als `Json`-Antwort zurückgegeben), `Router::build()` bei zwei Hilfsfunktionen und `&` bei acht `status_line`-Aufrufen
- Die Tests des `hello`-Beispiels verwiesen auf den falschen `CARGO_BIN_EXE`-Namen — das Binary-Target behält seinen Bindestrich (`CARGO_BIN_EXE_hello-bee`)
- Der Autoescape-Test des `hello`-Templates prüfte auf `&#39;`, während tera `&#x27;` ausgibt; das Escaping-Verhalten selbst war korrekt
- Clippy-Lints `manual_split_once` und `manual_range_contains` in `bee_cli`; ein ungenutzter Import im Integrationstest von `bee_router`

### Geändert
- Alle Crates tragen jetzt `readme`- und `repository`-Metadaten — zuvor rendierte keine crates.io-Seite ein README, und sechs Crates fehlte `repository` vollständig
- `examples/hello` ist mit `publish = false` markiert: bleibt als E2E-Test-Harness im Workspace, wird aber nicht mehr auf crates.io veröffentlicht

## [1.0.6] — 2026-08-07

### Neu hinzugefügt
- `bee_cli` echte Implementierungen: `new` (Projekt-Scaffolding), `generate controller/model`, `run` mit `--watch`-Hot-Reload, `pack` (Release-Build + Kopieren nach `dist/`)
- CLI-Unit-Tests für Scaffolding und Codegenerierung (7 neue Tests)

### Behoben
- `bee_rust::init()` ist jetzt hinter dem `logs`-Feature abgesperrt — reduzierte Feature-Builds (z. B. `--no-default-features --features kv`) kompilieren wieder
- Clippy-Lint `unnecessary_map_or` in `bee_kv::InMemoryKvStore::exists`
- `rustfmt.toml`: nur-Nightly-Optionen entfernt, die auf stable stillschweigend ignoriert wurden; der Workspace besteht jetzt `cargo fmt --all --check`
- `bee_cli`-Binary mit `doc = false`, um die Kollision der rustdoc-Ausgabedatei mit `bee_rust` zu beheben
- Der Port des `hello`-Beispiels ist jetzt über die Umgebungsvariable `PORT` konfigurierbar

### Geändert
- `bee-rust migrate` meldet „nicht implementiert" und endet mit einem Exit-Code ungleich null (geplant)
- README / README.en aktualisiert, um das tatsächliche CLI-Verhalten zu beschreiben

## [1.0.4] — 2026-07-29

### Neu hinzugefügt
- Sicherheitsfilter zur Angriffserkennung über `security-rust` (27 Detektoren)
- `SecurityFilter` mit Abdeckung von XSS, SQL-Injection, Command-Injection und Path-Traversal
- `security`-Feature-Flag in `bee_rust` und `bee_router`

### Geändert
- README um die Dokumentation des Sicherheitsfeatures erweitert
- README um einen Abschnitt zur Zahlungsunterstützung erweitert (WeChat Pay / Alipay)

### Behoben
- Tera-Syntax für rohe Bezeichner in `bee_template` für die Rust-2024-Edition

## [1.0.3] — 2026-07-29

### Neu hinzugefügt
- Initiale Workspace-Struktur mit 13 Crates
- MVC-Routing mit `Controller`-Trait und `Router`
- ORM mit `QuerySet`-Builder und `Model`-Derivierungsmakro
- Trait-Abstraktion für KV/Cache mit Redis- und Memory-Backends
- Session-Verwaltung mit Memory/Redis-Backends
- Konfigurationsverwaltung mit INI/YAML/ENV-Unterstützung und Hot-Reload
- Template-Rendering über Tera
- Logging mit tracing-Integration
- CLI-Scaffolding und Codegenerierung
- Trait-Stubs für Such-, Graph- und Zeitreihen-Engines (Treiber geplant)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
