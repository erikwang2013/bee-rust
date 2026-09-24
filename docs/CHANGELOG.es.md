# Registro de cambios

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
