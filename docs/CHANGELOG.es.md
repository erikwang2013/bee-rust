# Registro de cambios

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

> Versionado: mientras el proyecto sea 1.x, las versiones menores pueden incluir cambios que rompen la API; fija una versión exacta si haces match exhaustivo sobre `OrmError`. Véanse 1.3.0 y 1.6.0.

## [1.6.0] — 2026-10-25

### Añadido
- `bee_orm`: ergonomía de sharding — `#[bee(shard_key)]` marca el campo de enrutamiento (como máximo uno; rechazado en campos `ignore` y en campos `auto` / `auto_now_add` / `auto_now` asignados por la base de datos, porque el enrutamiento ocurre antes del insert), `Model::shard_key()` lo expone (`None` / `NULL` no es enrutable) y `ShardedPools::for_model` enruta una instancia sin repetir la clave. `ShardedPools::all_shards()` itera los pools configurados y `sync_all_parallel` migra cada pool × shard de tabla de forma concurrente (el DDL ya aplicado no se revierte).
- `bee_orm`: `OrmError::ShardError` separa el mal uso del sharding de los fallos de consulta; `OrmError` ahora es `#[non_exhaustive]` — las coincidencias externas necesitan un brazo comodín.
- `bee_orm`: `PoolStatus` entre backends con `pool_status()` en los pools sqlite, postgres y mysql (`size` / `available` son `None` en mysql, que no expone estadísticas), más `TimedDb` para callbacks de consultas lentas. Los benchmarks de Criterion cubren el hashing del router y el renderizado SQL de QuerySet.
- `bee_kv` / `bee_cache`: `RedisLock::acquire(key, ttl, wait)` reintenta el bloqueo como máximo cada 50ms hasta que expire la espera acotada; `RedisPoolStatus` / `pool_status()` ofrecen una instantánea del pool propia del crate; la feature `redis-tls` habilita conexiones `rediss://` y las credenciales ACL viajan en la URL.
- `bee_kv` / `bee_cache`: stores/caches de Sentinel (`RedisSentinelStore`, `RedisSentinelCache`) y stores/caches de Cluster (`RedisClusterStore`, `RedisClusterCache`) tras las features `redis-sentinel` / `redis-cluster`; `mget` / `mset` de clúster se ejecutan como comandos de clave única para evitar `CROSSSLOT`.
- `examples/sharding` y `examples/redis-lock`; CI compila ambos ejemplos y los benchmarks, fija MySQL 9.7.4 y mantiene el job de MariaDB.
- `bee_router`: presupuesto de latencia opcional para GET/HEAD con fallback a caché obsoleta — `Router::latency(LatencyConfig::new(cache))` usa 6 ms totales por defecto (4 ms de handler + 2 ms de reserva), cachea respuestas GET 200 públicas de hasta 64 KiB y las reproduce al agotarse el tiempo (`X-Bee-Cache: stale`, `Age`) o devuelve 504. Las GET que agotan el tiempo continúan en una tarea de fondo acotada para calentar la caché, y `LatencyStats` expone contadores y P50/P99 aproximados. Los métodos de escritura conservan el comportamiento de 30 s/408.

### Cambiado
- `OrmError` es `#[non_exhaustive]` y los fallos de sharding ahora informan `ShardError` en lugar de `QueryError`; las coincidencias aguas abajo necesitan un brazo comodín.

### Limitaciones
- Sentinel: las conexiones al maestro del pool se reciclan con `PING`, no se comprueba su rol, así que tras un failover un maestro degradado que siga accesible puede servir conexiones obsoletas. Mediante `Config::from_urls` de `deadpool-redis` 0.18 la conexión al maestro tampoco recibe credenciales ACL ni TLS (las credenciales/TLS de las URLs de Sentinel solo se aplican a los Sentinels); un maestro protegido por ACL o solo TLS debe conectarse directamente.
- Cluster: los comandos no están acotados por `RedisOptions::response_timeout` — `deadpool-redis` 0.18 no expone ningún setter; checkout, create y recycle conservan sus límites de 30s / 5s / 5s.
- Presupuesto de latencia: solo GET/HEAD. El plazo acota la respuesta, no el trabajo del servidor — las consultas bloqueantes de SQLite no se pueden cancelar y los comandos de Postgres/MySQL/Redis continúan. Las respuestas cacheadas son solo públicas y pueden estar obsoletas hasta el TTL; las escrituras nunca invalidan la caché; solo se cachean cuerpos de tamaño exacto ≤64 KiB; `MemoryCache` no tiene límite. Las rutas GET de long-poll/streaming deberían usar un Router sin `latency`.

## [1.5.0] — 2026-10-25

### Añadido
- `bee_orm`: compatibilidad con MySQL 5.7 y MariaDB 10.11+ — la matriz de CI ahora cubre 5.7 / 8.0 / 8.4 / 9.x más un job de MariaDB; las pruebas de integración tienen en cuenta el proveedor (MariaDB informa JSON como LONGTEXT y aplica `REFERENCES` a nivel de columna; MySQL 5.7/8.x lo analiza y lo ignora).
- `bee_orm`: `Model::create` en MySQL ahora usa los metadatos `columns()` del modelo para omitir la sonda de `information_schema` en cada insert; los modelos escritos a mano sin metadatos conservan la sonda anterior. La pista es autoritativa — la deriva entre esquema y modelo no está soportada.
- `bee_orm::shard`: `ShardedPools::{pools, pool, len, is_empty}` exponen los shards configurados para operaciones; `ShardHandle` reenvía la pista de autoincremento.
- `bee_kv` / `bee_cache`: métricas de pool `status()`; `RedisOptions` público (`max_size`, `response_timeout`) con `with_options`; `RedisLock::try_lock_with_watchdog` renueva el TTL hasta release/drop (el TTL del watchdog debe ser ≥ 30ms).
- Interno: una puerta de CI de consistencia de documentación (`scripts/check-docs.sh`), un script de renderizado de vista previa social y los fixtures de prueba compartidos extraídos de `tests/mod.rs`.

## [1.4.0] — 2026-10-25

### Añadido
- `bee_orm`: compatibilidad con MySQL 8.0 / 8.4 / 9.x, verificada por una matriz de CI. MySQL 9 aplica `REFERENCES` a nivel de columna (8.0/8.4 lo analizan y lo ignoran); crea primero las tablas referenciadas.
- `bee_orm`: `shard::{ShardedPools, HashRouter, ShardRouter}` — sharding hash con clave obligatoria para bases de datos y tablas; sin consultas ni transacciones entre shards.
- `bee_kv` / `bee_cache`: pool de conexiones Redis real (`deadpool-redis`) y `bee_kv::RedisLock` (`SET NX PX` + liberación Lua).

## [1.3.0] — 2026-10-13

### Corregido
- `bee_tsdb` / `bee_graph`: activar una sola feature de backend no compilaba — el helper compartido `http_client` vivía dentro de los módulos `influxdb` / `neo4j` (gateados por feature), así que una compilación solo con `iotdb` / `questdb` / `arangodb` / `nebulagraph` fallaba con unresolved import; ahora vive en el `crate::http` de cada crate (gateado sobre exactamente los backends que lo usan), y cada feature de backend individual compila por sí sola — la matriz de features se volvió a pasar punto por punto
- `bee_router`: la feature opcional `security` pasa a **security-rust 4.0.0** — `Scanner` / `DetectionResult` / `AttackCategory` / `Severity` conservan la forma de 1.x, sin cambios de código en bee_router; la superficie de capacidades crece: 32 detectores integrados (nuevos: `format_string`, `log4shell`, HPP, `formula_injection`, reDoS), los detectores pasan a gradarse en fuerte/débil (las señales débiles quedan fijadas en `Severity::Low`) y se añade `Scanner::assess`. El filtro sigue bloqueando ante cualquier detección, así que **las señales débiles (p. ej. un parámetro de consulta repetido) ahora también se bloquean** — ya no hay punto de inyección de un scanner propio; para dejarlas pasar, construya su propio `Filter` en torno a `SecurityFilter::scanner()`. El grafo de dependencias ahora lleva un único security-rust 4.0.0 en lugar de 1.x junto a 4.x; documentación sincronizada
- duplicados fusionados: `bee_tsdb` / `bee_graph` / `bee_search` conservan una sola copia de los helpers de conexión y mapeo de errores (el nombre del backend ahora lo pasa quien llama, textos de error sin cambios; `aggregation_fn` se traslada a la lib de `bee_tsdb`), `backtick` / `esc` de `bee_graph` se unifican en su módulo `http`, `bee_orm` extrae `dedup_pk_values` / `index_rows_by` y la cláusula de clave foránea de `migrate`, `bee_session::refresh` reutiliza `load` y `watch_path` de `bee_config` absorbe la ruta de llamada restante de `wait_for_change`; los mensajes de error de `bee_search` recuperan el nombre del backend y el stub de pruebas de `bee_kv` pasa a ser privado de sus pruebas
- API pública eliminada (de ahí la versión menor): `Context::set_params` / `param`, `SecurityFilter::with_scanner`, `Traversal::edge_labels` → `edge_label: Option<String>` (los tres backends ya tomaban solo la primera etiqueta — comportamiento sin cambios), `GraphError::EdgeNotFound`, `ConfigError::ParseError` / `TypeMismatch`, `bee_config::watch_path_timeout` y `bee_logs::Logger::async_` (un interruptor que `init()` nunca leyó) — todos sin llamadas
- dependencias muertas eliminadas: `bee_router` (`tracing` / `http` / `bee_config` / `bee_logs`), `bee_session` (`async-trait`), `bee_config_macro` (`proc-macro2`) y los `dev` `serde` duplicados en `bee_tsdb` / `bee_graph` / `bee_search`

## [1.2.3] — 2026-10-06

### Corregido
- `bee_cli`: el scaffold ya no genera proyectos que no compilan — la plantilla de `new` nombraba un paquete que no resuelve (`bee-rust`; se publica como `bee_rust`) y a la plantilla de controlador le faltaba `#[async_trait]` y usaba la ruta raíz privada de `RouterError`; una nueva prueba con puerta `BEE_CLI_E2E` genera un proyecto y le pasa `cargo check` de extremo a extremo
- docs: el ejemplo de controlador de la api ya está en su forma compilable (dependencia propia `async-trait`, `bee_rust::bee_router::context::RouterError`); quedan documentados el uso de `#[bee(crate = "bee_rust::bee_orm")]` para proyectos solo con `bee_rust`, la sintaxis de ruta `{name}` con la regla de prefijo vacío de `ns` y el uso de `create()`; se corrigió un fragmento en coreano del documento api en bengalí
- `examples/shortlink`: los detalles de error internos van al log en vez del cuerpo de la respuesta, la validación de URL rechaza hosts vacíos y caracteres de control, y `create_link` / `add_tag` usan ahora `Model::create()` (el insert de `Click` en la ruta caliente 302 conserva `insert()`)

## [1.2.2] — 2026-10-06

### Añadido
- `bee_orm`: `Model::create() -> Result<Self>` — inserta y devuelve la instancia completa, con la clave primaria asignada por la base de datos ya rellena (`INSERT … RETURNING *` en sqlite / postgres, `LAST_INSERT_ID()` + relectura en mysql); `insert()` no cambia y sigue devolviendo filas afectadas
- `bee_orm`: el atributo `#[bee(crate = "…")]` — indica a la derive la ruta del crate ORM (p. ej. `#[bee(crate = "bee_rust::bee_orm")]` cuando solo el metacrate `bee_rust` es dependencia); omitirlo mantiene la expansión byte a byte idéntica
- configuración de build de docs.rs completada para los ocho crates con módulos públicos tras feature (`all-features = true`) — la documentación de features de backend y la tabla de atributos del derive eran invisibles en docs.rs

## [1.2.1] — 2026-10-06

### Añadido
- `bee_orm`: campos de fecha y Decimal detrás de features opt-in — `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) y `rust_decimal` (`Decimal`) mapean a los tipos SQL `Date` / `DateTime` / `DateTimeTz` / `Decimal` en los tres backends; `bee_rust` los reenvía como `orm-chrono` / `orm-rust_decimal` (no están en `full`)
- `bee_orm`: límites honestos de esos tipos — sqlite guarda como TEXT (`typeof` = `text`; declarar DECIMAL daría afinidad NUMERIC y convertiría `"1.50"` en REAL 1.5 sin avisar), mysql conserva microsegundos en `datetime(6)` / `timestamp(6)` (TIMESTAMP se lee como UTC, DATETIME sigue naive) y en pg un `numeric` con más de 29 dígitos significativos se lee `NULL`; `NaiveTime` / `DateTime<Local>` / `FixedOffset` / nanosegundos quedan fuera y requieren `#[bee(sql_type = "…")]`, y con el feature apagado el modelo no compila (E0277)

### Corregido
- `bee_orm`: el aviso de relación m2m faltante ahora escribe el ident del tipo (`#[bee(m2m(Author))]`) y no el nombre de la tabla — la sugerencia compila también con la tabla renombrada
- CI: `rust-toolchain` ganó el input explícito `toolchain: stable`; los dos archivos de test que rompían `clippy --all-targets` sin features recibieron gates cfg a nivel de archivo (bee_cache / bee_kv) y el doctest m2m de `bee_orm` pasó a ser agnóstico del backend; el test end-to-end de `migrate` del CLI ahora construye su crate scratch en línea (offline solo pasaba con caché local caliente) y corre en CI con `BEE_CLI_E2E=1`

## [1.2.0] — 2026-10-06

### Añadido
- `bee_orm`: el ORM ya se ejecuta de extremo a extremo — parámetros tipados `Value` (Null / Bool / Int / Float / Text / Bytes), `#[derive(Model)]` con `#[bee(table / column / pk / auto / ignore)]`, `insert` / `update` / `delete` en el modelo y ejecución de QuerySet (`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm`: pools de conexiones para los tres backends (`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)`, `get()` → `CheckedConn`, `query` / `execute`, `status()` y transacciones con `begin` / `commit` / `rollback`; una conexión descartada a mitad de transacción se revierte en sqlite y postgres
- `bee_orm`: nueva feature `postgres-tls` (PostgreSQL por TLS, raíces Mozilla incluidas) y pools reforzados — tiempos de espera de 30 s / 10 s y caché de sentencias preparadas en el pool postgres
- `bee_orm`: ciclo de vida del modelo — marcas de tiempo con `#[bee(auto_now_add)]` / `#[bee(auto_now)]`, borrado lógico con `#[bee(soft_delete)]` (delete cambia la marca; `with_deleted()` / `hard_delete()` la eluden), hooks (before/after de insert/update/delete), `Model::insert_many` (lotes de 999 parámetros, no transaccional) y `QuerySet::filter_in` con agregados `sum` / `avg` / `min` / `max`
- `bee_orm`: migraciones no destructivas — `migrate::{create_table, add_missing_columns, sync}` generan DDL según el dialecto (sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`): crean tablas y añaden columnas faltantes sin eliminar ni modificar nunca
- `bee_orm`: relaciones por clave foránea — `#[bee(fk = Target)]` (junto a `#[bee(sql_type = "…")]`) emite DDL de FK y `rel::{fk_column_to, belongs_to, children, children_for}` las leen; el cumplimiento depende del backend: postgres y sqlite (build bundled) lo aplican, mysql ignora `REFERENCES` inline (hueco para el round 5+)
- `bee_orm`: relaciones many-to-many — `#[bee(m2m(Target))]` declara la relación, `m2m::{attach, detach, related, related_for, related_ids}` la leen y escriben, y `create_table` / `sync` crean la tabla puente (`add_missing_columns` nunca la toca)
- `bee_orm`: columnas JSON — los campos `serde_json::Value` se mapean a `TEXT` / `JSONB` / `JSON` según el backend, con semántica de NULL honesta: SQL `NULL` → `None`, un documento JSON `null` almacenado → `Some(Json::Null)`
- `bee_orm`: claves foráneas a nivel de tabla en MySQL (opt-in) — `MigrateOptions { table_level_fk }` con `sync_with` / `create_table_with` / `add_missing_columns_with`; desactivadas por defecto, y las filas huérfanas preexistentes hacen fallar `ADD CONSTRAINT` en vez de omitirse en silencio
- `bee_orm`: `Pool::connect_tls_with` para una `rustls::ClientConfig` propia, más un reexport de `bee_orm::rustls` para que la versión siempre coincida; `connect_tls` conserva las raíces webpki incluidas
- `bee_rust`: cuatro features de reenvío — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` reenvían un backend de `bee_orm` a través de `bee_rust` (ninguna está en `full`)
- `bee_cli`: `bee-rust migrate init` genera `src/bin/bee_migrate.rs` (se niega a sobrescribir un archivo existente) y `bee-rust migrate run` lo ejecuta con `cargo run --bin bee_migrate`
- `bee_kv` / `bee_cache`: los backends Redis sobreviven a conexiones caídas — el `ConnectionManager` subyacente reconecta bajo demanda (sin hilo en segundo plano) con backoff exponencial y jitter (el comando que topa con la caída falla; el siguiente espera la conexión nueva)
- `bee_kv` / `bee_cache`: backends memcached (`MemcacheStore` en bee_kv, `MemcacheCache` en bee_cache); `incr` crea el contador antes del delta, los contadores sin signo se quedan en 0 y TTL 0 borra la clave

### Cambiado
- Las migraciones y relaciones de bee_orm (incluido el many-to-many), el subcomando `bee-rust migrate`, los backends Redis / Memcached y las features de reenvío ORM de `bee_rust` ya no figuran como planificados en la documentación
- los drivers de búsqueda / grafos / series temporales ya no figuran en los docs como planificados ni como stub de trait, sino como implementados (features opt-in); además se corrigieron los ejemplos antiguos (nombres de tipo inexistentes `ElasticsearchEngine` / `Neo4jDB`, esquema `bolt://` erróneo)

## [1.1.5] — 2026-09-25

### Añadido
- READMEs condensados para crates.io en 11 idiomas más: `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`. El selector de idioma de la página de crates.io ahora apunta a estos archivos en lugar de a los README completos del repositorio, de modo que cada idioma recibe el mismo documento breve orientado al desarrollador.

## [1.1.4] — 2026-09-25

### Cambiado
- La página de crates.io (`docs/crates-readme.md`) ahora es bilingüe: inglés arriba, chino debajo. crates.io solo renderiza un README por crate y no ofrece selector de idioma en la página, así que el inglés queda en la primera pantalla y el texto chino permanece en la misma página.

## [1.1.3] — 2026-09-25

### Añadido
- La página de crates.io (`docs/crates-readme.md`) incluye ahora un selector de 13 idiomas que enlaza a los README del repositorio

### Corregido
- `docs/api.*`: 11 traducciones carecían del enlace al original en chino (`api.md`)
- `docs/CHANGELOG.zh.md` y `docs/CONTRIBUTING.zh.md` eran los únicos archivos de su familia sin enlace a sí mismos

## [1.1.2] — 2026-09-24

### Añadido
- `docs/crates-readme.md`: un README específico para las páginas de crates.io — instalación, ejemplo ejecutable (tomado de `examples/hello`, cubierto por sus tests E2E), tabla de feature flags e índice de sub-crates
- `scripts/publish.sh`: publica los crates del workspace uno a uno, omite los ya subidos y, ante la limitación de tasa, espera la hora de reintento que devuelve crates.io

### Corregido
- La suite de tests no compilaba (`cargo test --workspace` fallaba en main): en el test de integración de `bee_router` faltaba `Serialize` en `Submit` (se devuelve como respuesta `Json`), faltaba `Router::build()` en dos helpers y faltaba `&` en ocho llamadas a `status_line`
- Los tests del ejemplo `hello` referenciaban un nombre `CARGO_BIN_EXE` incorrecto — el target binario conserva su guion (`CARGO_BIN_EXE_hello-bee`)
- El test de autoescapado de plantillas de `hello` comprobaba `&#39;` mientras que tera emite `&#x27;`; el comportamiento de escapado en sí era correcto
- Lints de Clippy `manual_split_once` y `manual_range_contains` en `bee_cli`; un import sin usar en el test de integración de `bee_router`

### Cambiado
- Todos los crates ahora incluyen metadatos `readme` y `repository` — antes ninguna página de crates.io mostraba un README y a seis crates les faltaba `repository` por completo
- `examples/hello` está marcado como `publish = false`: permanece en el workspace como arnés de tests E2E pero ya no se publica en crates.io

## [1.0.6] — 2026-08-07

### Añadido
- Implementaciones reales de `bee_cli`: `new` (andamiaje de proyectos), `generate controller/model`, `run` con recarga en caliente mediante `--watch`, `pack` (compilación release + copia a `dist/`)
- Tests unitarios de la CLI para el andamiaje y la generación de código (7 tests nuevos)

### Corregido
- `bee_rust::init()` ahora está detrás del feature `logs`: las compilaciones con features reducidos (p. ej. `--no-default-features --features kv`) vuelven a compilar
- Lint de Clippy `unnecessary_map_or` en `bee_kv::InMemoryKvStore::exists`
- Se eliminaron de `rustfmt.toml` las opciones exclusivas de nightly que se ignoraban silenciosamente en stable; el workspace ahora pasa `cargo fmt --all --check`
- El binario `bee_cli` usa `doc = false` para eliminar la colisión de nombres en la salida de rustdoc con `bee_rust`
- El puerto del ejemplo `hello` ahora es configurable mediante la variable de entorno `PORT`

### Cambiado
- `bee-rust migrate` informa de «no implementado» y termina con código de salida distinto de cero (planificado)
- README / README.en actualizados para describir el comportamiento real de la CLI

## [1.0.4] — 2026-07-29

### Añadido
- Filtro de detección de ataques mediante `security-rust` (27 detectores)
- `SecurityFilter` con cobertura de XSS, inyección SQL, inyección de comandos y traversal de rutas
- Feature flag `security` en `bee_rust` y `bee_router`

### Cambiado
- README actualizado con la documentación de la feature de seguridad
- README actualizado con la sección de soporte mediante donaciones (WeChat Pay / Alipay)

### Corregido
- Sintaxis de identificadores raw de Tera en `bee_template` para la edición Rust 2024

## [1.0.3] — 2026-07-29

### Añadido
- Estructura inicial del workspace con 13 crates
- Enrutado MVC con el trait `Controller` y `Router`
- ORM con el builder `QuerySet` y la macro derive `Model`
- Abstracción por trait de KV/caché con backends Redis y Memory
- Gestión de sesiones con backends Memory/Redis
- Gestión de configuración con soporte INI/YAML/ENV y recarga en caliente
- Renderizado de plantillas mediante Tera
- Registros con integración de tracing
- Andamiaje de proyectos y generación de código en la CLI
- Stubs de traits para los motores de búsqueda, grafos y series temporales (drivers planificados)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
