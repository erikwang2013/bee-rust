<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

Beerust es un framework web de nivel producción escrito en Rust, cuya filosofía de diseño proviene del framework Beego de Go, reexpresada con los traits, las macros y el sistema de tipos idiomáticos de Rust.

## La mascota del proyecto: Rusty

<img src="rusty.svg" width="360" alt="Rusty flotando de frente, con las alas desplegadas en un círculo y las patas delanteras abrazando una caja de herramientas de color naranja oxidado">

### Ficha del personaje

**Nombre**: Rusty (Xiu Xiu) — el color oxidado de Rust y, también, esa mancha de óxido que lleva pegada en el mono de trabajo.

**Aspecto**: una abeja ingeniera peluda. El vello corto de ámbar dorado cubre su abdomen regordete, con tres franjas de negro carbón como unos tirantes de trabajo torcidos; cuatro alas transparentes, tan finas que dejan pasar la luz, que al flotar zumban dejando una estela; los ojos compuestos son dos esferas de obsidiana como canicas de vidrio, cada una con su destello; con las patas delanteras abraza una caja de herramientas de color naranja oxidado, reluciente por el uso: su cesta de polen, que nunca lleva polen, sino dependencias.

**Personalidad**: adicta al trabajo, optimista y ligeramente perfeccionista.

- Al reparar la colmena revisa celda por celda con las antenas, tan puntillosa como quien ejecuta un linter
- Las dependencias de su cesta de polen están bloqueadas con versiones fijadas a cal y canto; ay de quien las toque
- Cuando persigue un bug se queda suspendida en el aire, con las alas tan rápidas que solo se ve un anillo de estela
- El día que publica una nueva versión, baila el baile del ocho de las abejas en la entrada de la colmena

### Especificaciones visuales

| Elemento | Especificación |
|------|------|
| Cuerpo | Ámbar dorado `#F5B301`, vello abdominal corto y denso, con un halo de brillo en el borde |
| Franjas | Negro carbón `#1F1A17`, tres, solo en el abdomen |
| Alas | Azul claro translúcido `#CFE8F5`, 40 % de opacidad, con las venas dibujadas |
| Ojos compuestos | Marrón negruzco `#2A1E16`, elípticos, con un destello en la esquina superior izquierda |
| Acentos | Naranja oxidado `#B7410E` (caja de herramientas, pajarita); el color óxido es su firma |

### Conjunto de expresiones

| Expresión | Imagen | Cuándo |
|------|------|---------|
| Llena de energía | `(^o^)` pelaje erizado, alas vibrando a alta frecuencia | cuando todos los tests están en verde |
| Flotando en depuración | `o(°▽°)o` suspendida en el aire, solo se ve la estela de las alas | persiguiendo un bug difícil |
| Vuelta con la carga | `(≧▽≦)` la caja de herramientas repleta, volando con bandazos | cuando un nuevo driver se integra con éxito |
| Cabeceando | `(-_-)zZ` agazapada en la entrada de la colmena, con las antenas caídas | cuando no hay tráfico a altas horas de la noche |
| En alerta | `(#°益°)` aguijón en alto | cuando el filtro de seguridad intercepta un ataque |

### Movimientos característicos

1. **Depuración en suspensión** — cuando encuentra un bug, se cierne sobre el código, las alas zumban hasta quedar en estela y se queda mirando fijamente hasta que terminas de corregirlo.
2. **El baile del ocho** — cada vez que publica una nueva versión, da una vuelta completa alrededor de la colmena anunciando a todo el panal: «las flores han florecido». Las abejas reales usan este baile para transmitir la ubicación de las flores; Rusty lo usa para transmitir el changelog.
3. **Acicalarse** — se limpia las antenas con las patas delanteras una y otra vez. Crees que es por estética; en realidad está comprobando si la configuración se ha recargado en caliente.

### Dónde aparece

- **Logo**: de frente y en suspensión, con las alas desplegadas en un círculo que enmarca el nombre.
- **Página 404**: flotando sobre un campo de flores vacío, dando vueltas desorientada — «¿y la miel?»
- **CLI**: una pequeña abeja en el banner de arranque; cuando corres los tests se queda agachada a un lado, supervisando.
- **Anuncios de lanzamiento**: la foto del baile del ocho.

## Objetivos de diseño

| Objetivo | Métrica |
|------|------|
| **Experiencia de desarrollo** | Desde `bee-rust new` hasta la primera petición en < 30 s |
| **Rendimiento** | Sobrecarga de la capa de controladores < 5 % (frente a axum puro), latencia de ruteo P99 < 100 µs |
| **Velocidad de compilación** | Compilación completa del meta-crate < 60 s (release), compilación incremental < 5 s |
| **Tamaño del binario** | Aplicación mínima (solo router) < 5 MB (strip + LTO) |
| **Seguridad** | 0 código `unsafe` en la lógica de negocio; todo el FFI encapsulado en crates `*-sys` independientes |
| **Compatibilidad** | MSRV Rust 1.80+, siguiendo la rama stable |

## Principios de diseño

1. **Filosofía de Beego, expresión en Rust** — MVC, namespaces y cadena de filtros implementados con traits + macros
2. **Mejora progresiva** — el núcleo mínimo solo depende de axum + tokio; todo lo demás va detrás de feature flags
3. **Explícito antes que implícito** — el registro de rutas, el mapeo de modelos y el orden de los middlewares se declaran explícitamente en código
4. **Abstracciones de costo cero** — despacho estático con traits y macros expandidas en tiempo de compilación, sin sobrecarga de funciones virtuales
5. **Independencia de los motores de almacenamiento** — el trait de cada motor puede sustituirse por otra implementación sin afectar a la lógica de negocio de las capas superiores
6. **Observabilidad integrada** — instrumentación de tracing y métricas en todo el framework; logs estructurados activados por defecto

## Diseño de la arquitectura

### Topología de crates

```
bee_rust/           # Meta-crate, re-export + feature flags
bee_router/         # Enrutado + controlador + Context + cadena de filtros
bee_orm/            # ORM — trait Model + QuerySet
bee_kv/             # Abstracción unificada KV/Cache — Redis / Memcached
bee_search/         # Motor de búsqueda/análisis — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # Base de datos de grafos — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # Base de datos de series temporales — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # Gestión de configuración — INI/YAML/ENV + recarga en caliente
bee_cache/          # Abstracción de caché — Memory / Redis / Memcache
bee_session/        # Sesión — backends Memory, Redis (implementado) / Cookie, Database (planificado)
bee_logs/           # Registros — logs multinivel + integración con tracing
bee_template/       # Renderizado de plantillas — basado en tera
bee_cli/            # CLI — andamiaje/generación de código/ejecución en desarrollo/empaquetado (migrate init/run incluido)
```

### Diagrama de arquitectura

```
                          ┌───────────────────────┐
                          │  bee_rust  (meta)      │
                          │  re-export + features  │
                          └───────────┬───────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              │                       │                       │
     ┌────────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐
     │    Web Layer     │    │   Data Layer    │    │   Tool Layer    │
     └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
              │                       │                       │
  ┌───────────┼───────────┐  ┌───────┼───────┐  ┌───────────┼───────────┐
  │ bee_router            │  │ bee_orm        │  │ bee_cli               │
  │  - route register     │  │  - Model/Query  │  │  - scaffolding        │
  │  - controller trait   │  │  - QuerySet    │  │  - hot reload          │
  │  - filter chain       │  │  - Connection   │  │  - code generation    │
  │  - param extract      │  │                 │  │                       │
  ├────────────────────────┤  ├────────────────┤  ├───────────────────────┤
  │ bee_template           │  │ bee_config     │  │ bee_logs              │
  │  - template render     │  │  - INI/YAML/ENV│  │  - multi-level log    │
  │  - HTML/JSON           │  │  - hot reload   │  │  - tracing integrate  │
  ├────────────────────────┤  ├────────────────┤  └───────────────────────┘
  │ bee_session            │  │ bee_cache      │
  │  - session management  │  │  - cache trait  │
  │  - Memory/Redis        │  │  - Mem/Redis    │
  └────────────────────────┘  └────────────────┘

  ┌─────────────────────────────────────────────────────────┐
  │                   Storage Engine Layer                   │
  ├──────────────────┬──────────────────────────────────────┤
  │ bee_kv           │  Redis / Memcached                   │
  │ bee_search       │  Elasticsearch + OpenSearch + ClickHouse │
  │ bee_graph        │  Neo4j + NebulaGraph + ArangoDB      │
  │ bee_tsdb         │  InfluxDB + Apache IoTDB + QuestDB   │
  └──────────────────┴──────────────────────────────────────┘
```

### Dependencias entre crates

```
bee_config (sin dependencias)
bee_logs   (sin dependencias)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (sin dependencias)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → todos los crates anteriores (re-export)
```

### Bases de datos compatibles

| Categoría | Base de datos | Crate correspondiente | Feature Flag |
|------|--------|-----------|-------------|
| **Relacional** | SQLite | `bee_orm` | `sqlite` (bee_rust: `orm-sqlite`) |
| | PostgreSQL | `bee_orm` | `postgres` / `postgres-tls` (bee_rust: `orm-postgres` / `orm-postgres-tls`) |
| | MySQL 5.7 / 8.0 / 8.4 / 9.x · MariaDB 10.11+ | `bee_orm` | `mysql` (bee_rust: `orm-mysql`) |
| | TiDB | `bee_orm` | `mysql` |
| **Tipos de campo** | Fecha / Decimal | `bee_orm` | `chrono` / `rust_decimal` (bee_rust: `orm-chrono` / `orm-rust_decimal`) |
| **KV / caché** | Redis | `bee_kv` / `bee_cache` | `redis` / `redis-tls` / `redis-sentinel` / `redis-cluster` |
| | Memcached | `bee_kv` / `bee_cache` | `memcached` / `memcache` |
| **Búsqueda / análisis** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **Base de datos de grafos** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **Series temporales** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### Cadena de filtros de peticiones

```
Petición → [SecurityFilter detección de ataques] → [restauración de Session] → [validación de parámetros] → [hook prepare] → [procesado handle] → [hook finish] → Respuesta
                  ↓ cualquier etapa puede interrumpirse (similar al Abort de Beego)
```

## Funcionalidades

A continuación tienes un resumen de las funcionalidades; el uso detallado de cada módulo, los ejemplos de código y la documentación de la API están en la [Referencia de la API](api.es.md).

### Núcleo web (bee_router)

Controladores MVC: el trait `Controller` + `Context`, con soporte para namespaces de rutas, registro de métodos RESTful y cadenas de filtros de peticiones.

- Salida de respuestas: `ctx.json()` / `ctx.text()` / `ctx.html()`
- Redirección / interrupción: `ctx.redirect()` / `ctx.abort()`
- Sesión y parámetros: `ctx.session` / `ctx.params`
- Presupuesto de latencia (opcional): `Router::latency(LatencyConfig::new(cache))` otorga a GET/HEAD un presupuesto total predeterminado de 6 ms (4 ms de manejador + 2 ms de reserva para el fallback de caché); las respuestas públicas se almacenan en caché y se reproducen al agotarse el tiempo con `X-Bee-Cache: stale`, o la petición devuelve 504. Las escrituras conservan el tiempo de espera de 30 s; `LatencyStats` expone P50/P99.

### Detección de seguridad (feature `security`)

Un filtro de detección de ataques basado en [security-rust](https://crates.io/crates/security-rust) que cubre 27 tipos de ataques, como XSS, inyección SQL, inyección de comandos y SSRF; se activa con una sola línea:

```rust
let security = SecurityFilter::new();  // Los 27 detectores activados
```

### ORM (bee_orm)

La macro derivada `#[derive(Model)]` (con `#[bee(table / column / pk / auto / ignore / auto_now_add / soft_delete / shard_key)]`) + consultas encadenadas y ejecución de QuerySet (`all` / `one` / `count` / `exists` / `update` / `delete` / `filter_in`, agregados `sum` / `avg` / `min` / `max`) + migraciones no destructivas (`migrate::create_table` / `add_missing_columns` / `sync`) + lecturas de relaciones (`belongs_to` / has_many `children*` / many-to-many `m2m`) + columnas JSON (`serde_json::Value`) + pools de conexiones para los tres backends (con transacciones y timeouts), con soporte para SQLite, PostgreSQL, MySQL y TiDB; sharding de base de datos/tabla con clave obligatoria (`ShardedPools` + `HashRouter`: `#[bee(shard_key)]` / `for_model` enrutan directamente desde una instancia; la clave de shard debe proporcionarla el llamador — nunca un campo auto/marca de tiempo — y NULL no es enrutable; cada consulta debe llevar una clave de shard, y no se admiten consultas ni transacciones entre shards). Se admiten MySQL 5.7–9.x y MariaDB 10.11+; `sync_all_parallel` migra todos los shards de forma concurrente, y `all_shards()` / `pools()` exponen los pools configurados para operaciones. Un `PoolStatus` / `pool_status()` compartido y el envoltorio `TimedDb` para consultas lentas cubren todos los backends; los fallos de sharding tienen su propio `OrmError::ShardError`, y `OrmError` es `#[non_exhaustive]`.

### Gestión de configuración (bee_config)

La macro derivada `#[derive(Config)]`, con soporte para cargar INI / YAML / ENV y recarga en caliente.

### Motores de almacenamiento

KV / caché (Redis / Memcached), motores de búsqueda (Elasticsearch / OpenSearch / ClickHouse), bases de datos de grafos (Neo4j / NebulaGraph / ArangoDB) y bases de datos de series temporales (InfluxDB / IoTDB / QuestDB) comparten abstracciones unificadas por trait; los drivers se compilan según los feature flags. Los backends Redis usan un pool de conexiones real (`deadpool-redis`), y `bee_kv::RedisLock` ofrece un bloqueo distribuido (adquisición `SET NX PX` + liberación en Lua, espera acotada en `acquire` y renovación automática con `try_lock_with_watchdog`). Los pools de Redis exponen `status()` / `RedisPoolStatus` y `RedisOptions` con un tiempo de espera de respuesta configurable; `redis-tls` habilita `rediss://` y credenciales ACL en la URL; `redis-sentinel` / `redis-cluster` añaden backends KV/caché Sentinel y Cluster. La primera URL de Sentinel también aporta las credenciales ACL, la base de datos, el protocolo y TLS de la conexión al maestro; las conexiones del pool se siguen reciclando con PING sin comprobar el rol, por lo que un antiguo maestro degradado que siga siendo accesible puede continuar sirviendo. Los comandos de Cluster no están limitados por el tiempo de espera de respuesta.

### Sesión, registros y plantillas

- Sesión: Memory, Redis (implementado) / Cookie, Database (planificados)
- Registros: logs multinivel + integración con tracing
- Plantillas: renderizado basado en tera

### Herramientas CLI

```bash
bee-rust new my-app            # Crea el andamiaje del proyecto
bee-rust generate controller user
bee-rust run --watch           # Ejecución en desarrollo (recarga en caliente)
bee-rust pack                  # Empaquetado y despliegue
```

## Pasos de uso

### Requisitos del entorno

- Rust 1.80+
- Cargo

### Instalación

```bash
# Clona el proyecto
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# Compila
cargo build --workspace

# Ejecuta los tests
cargo test --workspace
```

### Inicio rápido

```bash
# Crea un nuevo proyecto con la CLI
cargo run -p bee_cli -- new hello
cd hello

# Ejecuta el servidor de desarrollo
cargo run
```

### Usar en tu proyecto

```toml
[dependencies]
bee_rust = { version = "1.6.1", features = ["full"] }
```

## Notas técnicas

### Stack tecnológico

| Capa | Tecnología |
|----|------|
| Base HTTP | axum 0.8 + tower 0.5 |
| Runtime asíncrono | tokio 1.x |
| Serialización | serde + serde_json |
| Motor de plantillas | tera 1.x |
| Capa de logs | tracing + tracing-subscriber |
| CLI | clap 4 |
| Parseo de configuración | toml / serde_yaml / INI propio |
| Manejo de errores | thiserror |
| Macros procedurales | syn + quote + proc-macro2 |

### Patrones de diseño

| Patrón | Aplicación |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Abstracción por trait** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **Macros derivadas** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature gates** | Implementaciones de drivers compiladas bajo demanda (redis, memcached, elasticsearch, etc.) |
| **Cadena de filtros** | Cadena de filtros de peticiones, a la altura del Filter de Beego |

### Listado de crates

| Crate | Función | Equivalente en Beego |
|-------|------|-----------|
| `bee_rust` | Meta-crate, punto de entrada unificado | — |
| `bee_router` | Enrutado + controlador + Context + filtros | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet | `client/orm` |
| `bee_kv` | Abstracción unificada KV/Cache | `client/cache` (extensión) |
| `bee_search` | Motor de búsqueda/análisis | — (nuevo) |
| `bee_graph` | Base de datos de grafos | — (nuevo) |
| `bee_tsdb` | Base de datos de series temporales | — (nuevo) |
| `bee_config` | Gestión de configuración + recarga en caliente | `client/config` |
| `bee_cache` | Abstracción de caché | `client/cache` |
| `bee_session` | Gestión de sesiones (Memory/Redis implementado; Cookie/Database planificado) | `server/web/session` |
| `bee_logs` | Registros | `logs` |
| `bee_template` | Renderizado de plantillas | — (mejorado) |
| `bee_cli` | Herramientas CLI | herramienta `bee` |

### Cobertura de tests

Los 475 tests del repositorio pasan:

| Crate | N.º de tests |
|-------|--------|
| bee_config | 8 |
| bee_cache | 27 |
| bee_template | 2 |
| bee_logs | 3 |
| bee_kv | 46 |
| bee_search | 21 |
| bee_graph | 19 |
| bee_tsdb | 19 |
| bee_orm | 211 |
| bee_orm_macro | 26 |
| bee_session | 2 |
| bee_router | 61 |
| bee_cli | 30 |

## Apoya el proyecto

Si este proyecto te resulta útil, ¡escanea el código QR para hacer una donación y apoyarlo! Gracias.

**WeChat Pay**

<img src="weixinpay.png" width="160" height="175" alt="WeChat Pay">

**Alipay**

<img src="alipay.png" width="160" height="175" alt="Alipay">

### Licencia

Apache-2.0
