# Journal des modifications

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
