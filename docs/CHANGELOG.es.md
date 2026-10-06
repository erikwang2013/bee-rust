# Registro de cambios

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
