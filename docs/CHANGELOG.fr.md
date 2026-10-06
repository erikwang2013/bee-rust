# Journal des modifications

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.2.1] — 2026-10-06

### Ajouts
- `bee_orm` : champs date et Decimal derrière des features opt-in — `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) et `rust_decimal` (`Decimal`) correspondent aux types SQL `Date` / `DateTime` / `DateTimeTz` / `Decimal` sur les trois backends ; `bee_rust` les transmet via `orm-chrono` / `orm-rust_decimal` (hors `full`)
- `bee_orm` : limites honnêtes de ces types — sqlite stocke en TEXT (`typeof` = `text` ; déclarer DECIMAL donnerait l'affinité NUMERIC et transformerait `"1.50"` en REAL 1.5 sans bruit), mysql garde les microsecondes de `datetime(6)` / `timestamp(6)` (TIMESTAMP se relit en UTC, DATETIME reste naïf), et côté pg un `numeric` au-delà de 29 chiffres significatifs se relit `NULL` ; `NaiveTime` / `DateTime<Local>` / `FixedOffset` / la précision nanoseconde sont hors périmètre et exigent `#[bee(sql_type = "…")]`, et feature éteinte le modèle ne compile pas (E0277)

### Corrections
- `bee_orm` : l'avertissement de relation m2m manquante épelle désormais l'ident du type (`#[bee(m2m(Author))]`) et non le nom de table — la suggestion compile même pour une table renommée
- CI : `rust-toolchain` a reçu l'input explicite `toolchain: stable` ; les deux fichiers de test qui cassaient `clippy --all-targets` sans features ont reçu des gates cfg au niveau fichier (bee_cache / bee_kv) et le doctest m2m de `bee_orm` est devenu agnostique du backend ; le test de bout en bout `migrate` du CLI construit maintenant son crate scratch en ligne (hors ligne ne passait qu'avec un cache local chaud) et tourne en CI via `BEE_CLI_E2E=1`

## [1.2.0] — 2026-10-06

### Ajouts
- `bee_orm` : l'ORM s'exécute désormais de bout en bout — paramètres typés `Value` (Null / Bool / Int / Float / Text / Bytes), `#[derive(Model)]` avec `#[bee(table / column / pk / auto / ignore)]`, `insert` / `update` / `delete` sur le modèle et exécution QuerySet (`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm` : pools de connexions pour les trois backends (`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)`, `get()` → `CheckedConn`, `query` / `execute`, `status()` et transactions via `begin` / `commit` / `rollback` ; une connexion abandonnée en pleine transaction est annulée sur sqlite et postgres
- `bee_orm` : nouvelle feature `postgres-tls` (PostgreSQL en TLS, racines Mozilla embarquées) et durcissement des pools — délais de 30 s (attente) / 10 s (création) et cache de requêtes préparées sur le pool postgres
- `bee_orm` : cycle de vie des modèles — horodatage via `#[bee(auto_now_add)]` / `#[bee(auto_now)]`, suppression logique via `#[bee(soft_delete)]` (delete bascule le drapeau ; `with_deleted()` / `hard_delete()` le contournent), hooks (before/after insert/update/delete), `Model::insert_many` (lots de 999 paramètres, non transactionnel), ainsi que `QuerySet::filter_in` et les agrégats `sum` / `avg` / `min` / `max`
- `bee_orm` : migrations non destructives — `migrate::{create_table, add_missing_columns, sync}` génèrent le DDL selon le dialecte (sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`), créent les tables et ajoutent les colonnes manquantes sans jamais supprimer ni modifier
- `bee_orm` : relations par clé étrangère — `#[bee(fk = Target)]` (avec `#[bee(sql_type = "…")]`) émet le DDL de FK, et `rel::{fk_column_to, belongs_to, children, children_for}` les lisent ; l'application dépend du backend : postgres et sqlite (build bundled) font respecter la référence, mysql ignore `REFERENCES` inline (lacune pour le round 5+)
- `bee_orm` : relations many-to-many — `#[bee(m2m(Target))]` déclare la relation, `m2m::{attach, detach, related, related_for, related_ids}` la lisent et l'écrivent, et `create_table` / `sync` créent la table de jointure (`add_missing_columns` n'y touche jamais)
- `bee_orm` : colonnes JSON — les champs `serde_json::Value` sont mappés vers `TEXT` / `JSONB` / `JSON` selon le backend, avec une sémantique NULL honnête : SQL `NULL` → `None`, un document JSON `null` stocké → `Some(Json::Null)`
- `bee_orm` : clés étrangères au niveau table sous MySQL (opt-in) — `MigrateOptions { table_level_fk }` avec `sync_with` / `create_table_with` / `add_missing_columns_with` ; désactivées par défaut, et des lignes orphelines préexistantes font échouer `ADD CONSTRAINT` au lieu d'être ignorées en silence
- `bee_orm` : `Pool::connect_tls_with` pour une `rustls::ClientConfig` maison, plus un réexport `bee_orm::rustls` pour que la version corresponde toujours ; `connect_tls` garde les racines webpki embarquées
- `bee_rust` : quatre features de transmission — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` transmettent un backend `bee_orm` via `bee_rust` (aucune n'est dans `full`)
- `bee_cli` : `bee-rust migrate init` génère `src/bin/bee_migrate.rs` (refuse d'écraser un fichier existant) et `bee-rust migrate run` l'exécute via `cargo run --bin bee_migrate`
- `bee_kv` / `bee_cache` : les backends Redis survivent aux connexions coupées — le `ConnectionManager` sous-jacent se reconnecte à la demande (sans thread d'arrière-plan) avec backoff exponentiel et jitter (la commande sur la coupure échoue, la suivante attend)
- `bee_kv` / `bee_cache` : backends memcached (`MemcacheStore` dans bee_kv, `MemcacheCache` dans bee_cache) ; `incr` crée le compteur avant le delta, les compteurs non signés s'arrêtent à 0, et TTL 0 supprime la clé

### Modifications
- Les migrations et relations de bee_orm (many-to-many compris), la sous-commande `bee-rust migrate`, les backends Redis / Memcached et les features de transmission ORM de `bee_rust` ne sont plus marqués comme prévus dans la documentation
- les drivers recherche / graphe / séries temporelles ne sont plus marqués dans les docs comme prévus ou comme stub de trait, mais comme implémentés (features opt-in) ; les anciens exemples ont été corrigés (noms de types inexistants `ElasticsearchEngine` / `Neo4jDB`, schéma `bolt://` erroné)

## [1.1.5] — 2026-09-25

### Ajouts
- Des READMEs condensés pour crates.io dans 11 langues supplémentaires : `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`. Le sélecteur de langue de la page crates.io pointe désormais vers ces fichiers au lieu des READMEs complets du dépôt : chaque langue obtient le même document court destiné aux développeurs.

## [1.1.4] — 2026-09-25

### Modifications
- La page crates.io (`docs/crates-readme.md`) est désormais bilingue : l'anglais en haut, le chinois en dessous. crates.io ne rend qu'un seul README par crate et n'offre pas de sélecteur de langue dans la page ; l'anglais apparaît donc dès le premier écran, le texte chinois restant sur la même page.

## [1.1.3] — 2026-09-25

### Ajouts
- La page crates.io (`docs/crates-readme.md`) comporte désormais un sélecteur de 13 langues renvoyant vers les READMEs du dépôt

### Corrections
- `docs/api.*` : 11 traductions n'avaient pas de lien vers l'original chinois (`api.md`)
- `docs/CHANGELOG.zh.md` et `docs/CONTRIBUTING.zh.md` étaient les seuls fichiers de leur famille sans lien vers eux-mêmes

## [1.1.2] — 2026-09-24

### Ajouts
- `docs/crates-readme.md` : un README dédié aux pages crates.io — installation, exemple exécutable (tiré de `examples/hello`, couvert par ses tests E2E), tableau des feature flags et index des sous-crates
- `scripts/publish.sh` : publie les crates du workspace un par un, ignore ceux déjà téléversés et, en cas de limitation de débit, attend l'heure de réessai renvoyée par crates.io

### Corrections
- La suite de tests ne compilait pas (`cargo test --workspace` échouait sur main) : dans le test d'intégration de `bee_router`, `Serialize` manquait sur `Submit` (renvoyé comme réponse `Json`), `Router::build()` manquait sur deux helpers, et `&` manquait sur huit appels à `status_line`
- Les tests de l'exemple `hello` référençaient un mauvais nom `CARGO_BIN_EXE` — la cible binaire conserve son tiret (`CARGO_BIN_EXE_hello-bee`)
- Le test d'auto-échappement du template `hello` vérifiait `&#39;` alors que tera produit `&#x27;` ; le comportement d'échappement lui-même était correct
- Lints Clippy `manual_split_once` et `manual_range_contains` dans `bee_cli` ; un import inutilisé dans le test d'intégration de `bee_router`

### Modifications
- Tous les crates portent désormais les métadonnées `readme` et `repository` — auparavant aucune page crates.io n'affichait de README et six crates n'avaient pas de `repository` du tout
- `examples/hello` est marqué `publish = false` : il reste dans le workspace comme harnais de tests E2E mais n'est plus publié sur crates.io

## [1.0.6] — 2026-08-07

### Ajouts
- Implémentations réelles de `bee_cli` : `new` (scaffolding de projet), `generate controller/model`, `run` avec rechargement à chaud `--watch`, `pack` (build release + copie dans `dist/`)
- Tests unitaires du CLI pour le scaffolding et la génération de code (7 nouveaux tests)

### Corrections
- `bee_rust::init()` est désormais derrière la feature `logs` — les builds avec des features réduites (p. ex. `--no-default-features --features kv`) compilent à nouveau
- Lint Clippy `unnecessary_map_or` dans `bee_kv::InMemoryKvStore::exists`
- `rustfmt.toml` : suppression des options réservées à nightly, silencieusement ignorées sur stable ; le workspace passe désormais `cargo fmt --all --check`
- Binaire `bee_cli` : `doc = false` pour supprimer la collision de nom de fichier de sortie rustdoc avec `bee_rust`
- Le port de l'exemple `hello` est désormais configurable via la variable d'environnement `PORT`

### Modifications
- `bee-rust migrate` signale "not implemented" et se termine avec un code de sortie non nul (prévu)
- README / README.en mis à jour pour décrire le comportement réel du CLI

## [1.0.4] — 2026-07-29

### Ajouts
- Filtre de détection d'attaques via `security-rust` (27 détecteurs)
- `SecurityFilter` couvrant XSS, injection SQL, injection de commandes et traversée de chemins
- Feature flag `security` dans `bee_rust` et `bee_router`

### Modifications
- README mis à jour avec la documentation de la feature de sécurité
- README mis à jour avec la section de soutien par paiement (WeChat Pay / Alipay)

### Corrections
- Syntaxe des identifiants bruts Tera dans `bee_template` pour l'édition Rust 2024

## [1.0.3] — 2026-07-29

### Ajouts
- Structure initiale du workspace avec 13 crates
- Routage MVC avec le trait `Controller` et `Router`
- ORM avec le builder `QuerySet` et la macro dérivée `Model`
- Abstraction par traits KV/Cache avec les backends Redis et Memory
- Gestion de session avec les backends Memory/Redis
- Gestion de configuration avec prise en charge INI/YAML/ENV et rechargement à chaud
- Rendu de templates via Tera
- Journalisation avec intégration tracing
- Scaffolding et génération de code via le CLI
- Stubs de traits pour les moteurs de recherche, graphe et séries temporelles (drivers prévus)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
