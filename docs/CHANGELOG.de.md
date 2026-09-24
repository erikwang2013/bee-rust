# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
