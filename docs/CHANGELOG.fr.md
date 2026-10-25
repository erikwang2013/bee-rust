# Journal des modifications

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

> Versionnement : tant que le projet est en 1.x, les versions mineures peuvent inclure des changements cassants d'API ; épinglez une version exacte si vous faites un match exhaustif sur `OrmError`. Voir 1.3.0 et 1.6.0.

## [1.6.1] — 2026-10-26

### Sécurité
- La mise à jour des dépendances ferme 7 vulnérabilités RUSTSEC et 6 avertissements d'audit. `rustls` 0.23.46, `rustls-webpki` 0.103.16, `webpki-roots` 1.0.11, `ctutils` 0.4.4, `tokio` 1.54.1, `lru` 0.18.5 et `yoke` / `yoke-derive` 0.8.4 remplacent les versions concernées ; `mysql_async` 0.37.1 et `redis` 1.6 / `deadpool-redis` 0.23.1 suppriment les chaînes non maintenues `rustls-pemfile`, `proc-macro-error2` et l'ancien `lru`. `cargo audit` signale désormais 0 vulnérabilité et 0 avertissement.

### Modifications
- `bee_kv` / `bee_cache` : la première URL Sentinel fournit désormais aussi les identifiants ACL, la base de données, le protocole et TLS de la connexion au maître via `SentinelNodeConnectionInfo` ; un maître protégé par ACL ou uniquement TLS fonctionne lorsque son URL Sentinel porte les mêmes réglages. Les connexions du pool sont toujours uniquement recyclées par PING, sans vérification de rôle.
- La mise à niveau du pilote MySQL vers `mysql_async` 0.37 laisse le comportement observable inchangé et a été vérifiée avec MySQL 5.7.44 / 8.0.46 / 8.4.11 / 9.7.4 et MariaDB 10.11.19.

## [1.6.0] — 2026-10-25

### Ajouts
- `bee_orm` : ergonomie du sharding — `#[bee(shard_key)]` marque le champ de routage (au plus un ; rejeté sur les champs `ignore` et sur les champs `auto` / `auto_now_add` / `auto_now` attribués par la base, car le routage a lieu avant l'insert), `Model::shard_key()` l'expose (`None` / `NULL` n'est pas routable), et `ShardedPools::for_model` route une instance sans répéter la clé. `ShardedPools::all_shards()` itère sur les pools configurés, et `sync_all_parallel` migre chaque pool × shard de table en parallèle (un DDL déjà appliqué n'est pas annulé).
- `bee_orm` : `OrmError::ShardError` distingue un mauvais usage du sharding d'un échec de requête ; `OrmError` est désormais `#[non_exhaustive]` — les matchs externes nécessitent une branche générique.
- `bee_orm` : `PoolStatus` inter-backends avec `pool_status()` sur les pools sqlite, postgres et mysql (`size` / `available` valent `None` sur mysql, qui n'expose aucune statistique), plus `TimedDb` pour les callbacks de requêtes lentes. Les benchmarks Criterion couvrent le hachage du routeur et le rendu SQL de QuerySet.
- `bee_kv` / `bee_cache` : `RedisLock::acquire(key, ttl, wait)` retente le verrou au plus toutes les 50ms jusqu'à expiration de l'attente bornée ; `RedisPoolStatus` / `pool_status()` fournissent un instantané de pool propre à la crate ; la feature `redis-tls` active les connexions `rediss://` et les identifiants ACL passent dans l'URL.
- `bee_kv` / `bee_cache` : stores/caches Sentinel (`RedisSentinelStore`, `RedisSentinelCache`) et stores/caches Cluster (`RedisClusterStore`, `RedisClusterCache`) derrière les features `redis-sentinel` / `redis-cluster` ; les `mget` / `mset` de cluster s'exécutent comme des commandes à clé unique pour éviter `CROSSSLOT`.
- `examples/sharding` et `examples/redis-lock` ; la CI compile les deux exemples et les benchmarks, épingle MySQL 9.7.4 et conserve le job MariaDB.
- `bee_router` : budget de latence opt-in pour GET/HEAD avec repli sur cache périmé — `Router::latency(LatencyConfig::new(cache))` utilise 6 ms au total par défaut (4 ms de handler + 2 ms de réserve), met en cache les réponses GET 200 publiques jusqu'à 64 KiB et les rejoue en cas de timeout (`X-Bee-Cache: stale`, `Age`) ou renvoie 504. Les GET expirés continuent dans une tâche d'arrière-plan bornée pour réchauffer le cache, et `LatencyStats` expose des compteurs ainsi que des P50/P99 approximatifs. Les méthodes d'écriture conservent le comportement 30 s/408.

### Modifications
- `OrmError` est `#[non_exhaustive]` et les échecs de sharding rapportent désormais `ShardError` au lieu de `QueryError` ; les matchs en aval nécessitent une branche générique.

### Limitations
- Sentinel : les connexions au maître du pool sont recyclées par `PING`, sans vérification de rôle ; après un failover, un maître rétrogradé qui reste joignable peut donc servir des connexions obsolètes. Via `Config::from_urls` de `deadpool-redis` 0.18, la connexion au maître ne reçoit pas non plus d'identifiants ACL ni de TLS (les identifiants/TLS des URLs Sentinel ne s'appliquent qu'aux Sentinels) ; un maître protégé par ACL ou uniquement TLS doit se connecter directement.
- Cluster : les commandes ne sont pas bornées par `RedisOptions::response_timeout` — `deadpool-redis` 0.18 n'expose aucun setter ; checkout, create et recycle conservent leurs bornes de 30s / 5s / 5s.
- Budget de latence : GET/HEAD uniquement. L'échéance borne la réponse, pas le travail côté serveur — les requêtes bloquantes SQLite ne peuvent pas être annulées et les commandes Postgres/MySQL/Redis continuent. Les réponses en cache sont publiques uniquement et peuvent être périmées jusqu'au TTL ; les écritures n'invalident jamais le cache ; seuls les corps de taille exacte ≤64 KiB sont mis en cache ; `MemoryCache` est non borné. Les routes GET en long-poll/streaming doivent utiliser un Router sans `latency`.

## [1.5.0] — 2026-10-25

### Ajouts
- `bee_orm` : prise en charge de MySQL 5.7 et MariaDB 10.11+ — la matrice CI couvre désormais 5.7 / 8.0 / 8.4 / 9.x plus un job MariaDB ; les tests d'intégration tiennent compte du vendeur (MariaDB renvoie le JSON en LONGTEXT et applique `REFERENCES` au niveau des colonnes ; MySQL 5.7/8.x l'analysent et l'ignorent).
- `bee_orm` : `Model::create` sur MySQL utilise désormais les métadonnées `columns()` du modèle pour éviter la sonde `information_schema` à chaque insert ; les modèles écrits à la main sans métadonnées conservent l'ancienne sonde. Cet indice fait foi — une dérive schéma/modèle n'est pas prise en charge.
- `bee_orm::shard` : `ShardedPools::{pools, pool, len, is_empty}` exposent les shards configurés pour l'exploitation ; `ShardHandle` relaie l'indice d'auto-incrément.
- `bee_kv` / `bee_cache` : métriques de pool `status()` ; `RedisOptions` public (`max_size`, `response_timeout`) avec `with_options` ; `RedisLock::try_lock_with_watchdog` renouvelle le TTL jusqu'au release/drop (le TTL du watchdog doit être ≥ 30ms).
- Interne : une garde CI de cohérence de la documentation (`scripts/check-docs.sh`), un script de rendu d'aperçu social et les fixtures de test partagées extraites de `tests/mod.rs`.

## [1.4.0] — 2026-10-25

### Ajouts
- `bee_orm` : compatibilité MySQL 8.0 / 8.4 / 9.x, vérifiée par une matrice CI. MySQL 9 applique `REFERENCES` au niveau des colonnes (8.0/8.4 l'analysent et l'ignorent) ; créez d'abord les tables référencées.
- `bee_orm` : `shard::{ShardedPools, HashRouter, ShardRouter}` — sharding par hachage à clé obligatoire pour les bases et les tables ; pas de requêtes ni de transactions inter-shards.
- `bee_kv` / `bee_cache` : vrai pool de connexions Redis (`deadpool-redis`) et `bee_kv::RedisLock` (`SET NX PX` + libération Lua).

## [1.3.0] — 2026-10-13

### Corrections
- `bee_tsdb` / `bee_graph` : activer une seule feature de backend ne compilait pas — l'aide partagée `http_client` vivait dans les modules `influxdb` / `neo4j` soumis à feature, si bien qu'une build avec seulement `iotdb` / `questdb` / `arangodb` / `nebulagraph` échouait sur un unresolved import ; elle réside désormais dans le `crate::http` de chaque crate (gaté sur exactement les backends qui l'utilisent), et chaque feature de backend isolée compile seule — la matrice de features a été repassée point par point
- `bee_router` : la feature optionnelle `security` passe à **security-rust 4.0.0** — `Scanner` / `DetectionResult` / `AttackCategory` / `Severity` gardent la forme 1.x, aucun changement de source n'a été nécessaire côté bee_router ; la surface de détection s'élargit : 32 détecteurs intégrés (nouveaux : `format_string`, `log4shell`, HPP, `formula_injection`, reDoS), les détecteurs sont désormais gradés fort/faible (les signaux faibles sont figés à `Severity::Low`) et `Scanner::assess` fait son apparition. Le filtre bloque toujours à la moindre détection, donc **les signaux faibles (p. ex. un paramètre de requête répété) sont maintenant bloqués eux aussi** — il n'existe plus de point d'injection pour un scanner personnalisé ; pour les laisser passer, écrivez votre propre `Filter` autour de `SecurityFilter::scanner()`. Le graphe de dépendances ne contient plus qu'un seul security-rust 4.0.0 au lieu de 1.x à côté de 4.x ; la documentation est alignée
- doublons fusionnés : `bee_tsdb` / `bee_graph` / `bee_search` ne gardent qu'une copie des aides de connexion et de mappage d'erreurs (le nom du backend est maintenant passé par l'appelant, messages d'erreur inchangés ; `aggregation_fn` part dans la lib de `bee_tsdb`), `backtick` / `esc` de `bee_graph` rejoignent son module `http`, `bee_orm` extrait `dedup_pk_values` / `index_rows_by` et la clause de clé étrangère de `migrate`, `bee_session::refresh` réutilise `load`, et `watch_path` de `bee_config` absorbe le chemin d'appel `wait_for_change` restant ; les messages d'erreur de `bee_search` retrouvent le nom du backend, et le stub de test de `bee_kv` devient privé à ses tests
- API publique supprimée (d'où la version mineure) : `Context::set_params` / `param`, `SecurityFilter::with_scanner`, `Traversal::edge_labels` → `edge_label: Option<String>` (les trois backends ne prenaient déjà que le premier label — comportement inchangé), `GraphError::EdgeNotFound`, `ConfigError::ParseError` / `TypeMismatch`, `bee_config::watch_path_timeout` et `bee_logs::Logger::async_` (un interrupteur que `init()` n'a jamais lu) — tous sans aucun appelant
- dépendances mortes supprimées : `bee_router` (`tracing` / `http` / `bee_config` / `bee_logs`), `bee_session` (`async-trait`), `bee_config_macro` (`proc-macro2`), et les `dev` `serde` en double dans `bee_tsdb` / `bee_graph` / `bee_search`

## [1.2.3] — 2026-10-06

### Corrections
- `bee_cli` : le scaffold ne génère plus de projets qui ne compilent pas — le modèle `new` nommait un paquet qui ne se résout pas (`bee-rust` ; publié sous `bee_rust`) et le modèle de contrôleur n'avait pas `#[async_trait]` et utilisait le chemin racine privé de `RouterError` ; un nouveau test gardé par `BEE_CLI_E2E` génère un projet et lui fait passer `cargo check` de bout en bout
- docs : l'exemple de contrôleur de l'api est désormais sous sa forme compilable (dépendance `async-trait` propre, `bee_rust::bee_router::context::RouterError`) ; l'orthographe `#[bee(crate = "bee_rust::bee_orm")]` pour les projets dépendant uniquement de `bee_rust`, la syntaxe de route `{name}` avec la règle du préfixe vide de `ns` et l'usage de `create()` sont documentés ; un fragment coréen présent dans la doc api bengalie a été corrigé
- `examples/shortlink` : les détails d'erreur internes partent dans le log au lieu du corps de la réponse, la validation d'URL rejette les hôtes vides et les caractères de contrôle, et `create_link` / `add_tag` utilisent maintenant `Model::create()` (l'insert `Click` sur le chemin chaud 302 reste `insert()`)

## [1.2.2] — 2026-10-06

### Ajouts
- `bee_orm` : `Model::create() -> Result<Self>` — insère et renvoie l'instance complète, clé primaire attribuée par la base incluse (`INSERT … RETURNING *` sur sqlite / postgres, `LAST_INSERT_ID()` + relecture sur mysql) ; `insert()` reste inchangé et renvoie toujours les lignes affectées
- `bee_orm` : l'attribut `#[bee(crate = "…")]` — indique à la derive le chemin du crate ORM (par ex. `#[bee(crate = "bee_rust::bee_orm")]` quand seul le métacrate `bee_rust` est une dépendance) ; sans lui l'expansion reste identique octet pour octet
- configuration de build docs.rs complétée pour les huit crates aux modules publics conditionnés par feature (`all-features = true`) — la doc des features backend et la table des attributs de la derive étaient jusqu'ici invisibles sur docs.rs

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
