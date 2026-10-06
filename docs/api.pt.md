<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Referência da API Beerust

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Voltar ao [README](README.pt.md).

Este documento é a referência da API do framework Beerust: o uso de cada módulo, exemplos de código e descrição das interfaces.

## Núcleo Web (bee_router)

### Definindo um controlador

```rust
use bee_rust::prelude::*;

// Define um controlador
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Registro de rotas

```rust
// Registro de rotas
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### API do Context

**O Context oferece:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — saída de resposta
- `ctx.redirect()` — redirecionamento
- `ctx.abort()` — interromper a requisição
- `ctx.session` — acesso à sessão
- `ctx.params` — parâmetros de caminho

## Detecção de segurança (feature `security`)

Filtro de detecção de ataques baseado em [security-rust](https://crates.io/crates/security-rust), cobrindo 27 tipos de ataque, incluindo XSS, injeção de SQL, injeção de comandos e SSRF:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // todos os 27 detectores ativados
```

Habilite no `Cargo.toml`:
```toml
bee_rust = { features = ["security"] }
```

## ORM (bee_orm)

```rust
use bee_orm::pool::sqlite::Pool;
use bee_orm::{Model, Value, migrate, rel};

#[derive(Model)]
#[bee(table = "users")]
struct User {
    #[bee(pk, auto)]
    id: i64,
    #[bee(column = "user_name")]
    name: String,
    age: Option<i32>,
    active: bool,
    #[bee(auto_now_add)]
    created_at: i64,
    #[bee(soft_delete)]
    deleted: bool,
    #[bee(ignore)]
    cache: Vec<u8>,
}

#[derive(Model)]
#[bee(table = "posts")]
struct Post {
    #[bee(pk, auto)]
    id: i64,
    #[bee(fk = User)]
    user_id: Option<i64>,
    title: String,
    #[bee(soft_delete)]
    deleted: bool,
}

async fn demo() -> Result<(), bee_orm::OrmError> {
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql têm a mesma forma

    // Esquema — migrate::sync gera DDL não destrutivo a partir dos metadados do modelo
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — a chave primária `auto` é atribuída pelo banco
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — filtros encadeados + execução: all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // Relações — children para has_many, belongs_to para o pai; com FK aplicada, exclua o filho antes do pai
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete é lógico (inverte a flag); with_deleted() / hard_delete() contornam
    Post::query().filter_eq("title", "hello")?.hard_delete(&pool).await?;
    if let Some(mut u) = author {
        u.age = Some(31);
        u.update(&pool).await?;
        u.delete(&pool).await?;
        let live = User::query().count(&pool).await?;
        let every = User::query().with_deleted().count(&pool).await?;
        u.hard_delete(&pool).await?;
    }
    User::query().filter_eq("user_name", "bob")?.update(&pool, &[("user_name", "bobby".into())]).await?;

    // Transações — mantenha uma conexão de `get()`; o `CheckedConn` do sqlite é síncrono
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> As migrações já estão disponíveis: `bee_orm::migrate` (`create_table` / `add_missing_columns` / `sync`) gera o DDL conforme o dialeto — cria tabelas e adiciona colunas ausentes, nunca remove nem altera. As leituras de relações cobrem `belongs_to`, has_many (`children*`) e many-to-many (`m2m`). `#[bee(fk = …)]` emite DDL de chave estrangeira: postgres e sqlite (build bundled) aplicam a referência, e o mysql pode ativar chaves estrangeiras em nível de tabela via opt-in (abaixo).

### Many-to-many (`m2m`)

Declarado no nível do struct com `#[bee(m2m(Target))]`; a tabela de junção é criada por `create_table` / `sync` (sincronize primeiro o modelo alvo, depois o declarante):

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // tabela de junção user_tag, colunas user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // primeiro o modelo alvo
migrate::sync::<User, _>(&pool).await?;      // o declarante cria a tabela de junção

// Os ids `auto` são atribuídos pelo banco: releia a instância antes de mexer nas relações
User { id: 0, name: "alice".into() }.insert(&pool).await?;
let user = User::query().filter_eq("name", "alice")?.one(&pool).await?.expect("inserted");
Tag { id: 0, name: "rust".into() }.insert(&pool).await?;
let tag = Tag::query().filter_eq("name", "rust")?.one(&pool).await?.expect("inserted");

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // attach duplicado -> erro de PK composta
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // idempotente: a segunda chamada retorna 0 linhas
```

> Renomeações: `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`; as tabelas de junção só são criadas por `create_table` / `sync` — `add_missing_columns` nunca as toca.

### Colunas JSON

Campos `serde_json::Value` mapeiam para sqlite `TEXT` / postgres `JSONB` / mysql `JSON`:

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // obrigatório
    extra: Option<serde_json::Value>,  // nullable
}
```

> Semântica de NULL: SQL `NULL` decodifica para `None`; um documento JSON `null` armazenado decodifica para `Some(Json::Null)` — os dois são diferentes.

### Chaves estrangeiras em nível de tabela no MySQL (opt-in)

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // apenas mysql
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> Ativada, tabelas novas e colunas adicionadas carregam `FOREIGN KEY` em nível de tabela (nome da constraint `{table}_{column}_fk`); linhas órfãs preexistentes fazem `ADD CONSTRAINT` falhar (sem omissão silenciosa e sem retrofit de colunas existentes); em pg / sqlite a opção é inócua (inline já aplica).

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls usa as raízes webpki embutidas; para uma config rustls própria, connect_tls_with:
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig (reexportado, mesma versão da crate)
```

### Tipos data e Decimal

Campos de data e Decimal são gateados por feature: ative `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) e `rust_decimal` (`Decimal`) no `bee_orm`; via `bee_rust` os features de encaminhamento são `orm-chrono` / `orm-rust_decimal`.

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date; DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal (igual nos três backends)
}

// escrita: o campo vira a variante Value correspondente; leitura: a célula decodifica via serde
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // "1.50" volta com valor igual
```

> Suportado: `NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`. Fora do escopo — a macro não emite mapeamento SQL, use a saída `#[bee(sql_type = "…")]`: `NaiveTime` / `DateTime<Local>` / `FixedOffset` / `DateTime` puro, e precisão de nanossegundos.
> Armazenamento: sqlite sempre TEXT (`typeof(col)` é `text`; Decimal também — declarar DECIMAL daria afinidade NUMERIC e engoliria "1.50" como REAL 1.5); mysql usa `datetime(6)` / `timestamp(6)` com microssegundos — TIMESTAMP volta como UTC (RFC3339 com `Z`; o round trip estrito exige fuso da sessão em UTC), DATETIME segue naive sem sufixo, despachados por tipo de coluna; pg `timestamptz` normaliza para UTC.
> Teto de precisão: no pg, `numeric` acima de 29 dígitos significativos volta como `NULL` (o FromSql nativo recusa); no mysql vai por texto, o excesso só falha na decodificação no seu serde; para dinheiro prefira `#[bee(sql_type = Raw("decimal(12,2)"))]`.
> Contrato de compilação: com a feature desligada, um modelo com campo de data falha com E0277 (`Value: From<NaiveDate>` não satisfeito) — nunca uma coluna silenciosamente errada.

## Gerenciamento de configuração (bee_config)

```rust
#[derive(Config)]
#[config(file = "conf/app.conf")]
struct AppConfig {
    app_name: String,
    http_port: u16,
    run_mode: String,
}

let cfg = AppConfig::load("conf/app.conf")?;
```

## Mecanismos de armazenamento

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// backend memcached (feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> O `RedisStore` usa um `ConnectionManager`: uma conexão derrubada se reconecta sozinha (sob demanda — sem thread em segundo plano — com backoff exponencial e jitter) — o comando que esbarra na queda falha e o próximo espera a conexão nova. memcached: `incr` cria o contador primeiro (0 se ausente) e então aplica o delta; contadores são sem sinal, então decrementar fica em 0 (no Redis vai a negativo); `expire(≤0)` equivale a excluir.

**Backends de cache (bee_cache):**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache` (feature `memcache`) tem a mesma interface; TTL `Some(0)` passa por `DEL` no backend Redis (`SET … EX 0` é rejeitado) e também é tratado como exclusão no memcached (lá 0 significa nunca expirar); `incr` em valor não numérico é erro de serialização.

**Mecanismo de busca (implementado, opt-in):**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // requer a feature `elasticsearch`
let engine = Elasticsearch::new("http://localhost:9200");   // construtor síncrono (sem `?`)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> Os demais drivers funcionam igual e também são opt-in: `opensearch`, `clickhouse` (features de mesmo nome).

**Banco de dados de grafos (implementado, opt-in):**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // requer a feature `neo4j`
let db = Neo4j::new("http://localhost:7474");   // construtor síncrono (sem `?`)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> Os demais drivers funcionam igual e também são opt-in: `nebulagraph`, `arangodb` (features de mesmo nome).

**Banco de dados de séries temporais (implementado, opt-in):**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // requer a feature `influxdb`
let tsdb = InfluxDB::new("http://localhost:8086");   // construtor síncrono (sem `?`)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> Os demais drivers funcionam igual e também são opt-in: `iotdb`, `questdb` (features de mesmo nome).

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> O backend é qualquer cache que implemente `bee_cache::Cache`: `MemoryCache` / `RedisCache` / `MemcacheCache`.

## Logs

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## Templates

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## Ferramenta CLI

```bash
# Cria um projeto (gera um scaffolding executável: Cargo.toml + src/main.rs)
bee-rust new my-app

# Gera código
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Execução de desenvolvimento (--watch monitora mudanças em src/ e reinicia automaticamente)
bee-rust run
bee-rust run --watch

# Empacotamento para implantação (cargo build --release + cópia para dist/)
bee-rust pack

# Migração de banco de dados (gera o ponto de entrada na sua crate e o executa)
bee-rust migrate init    # cria src/bin/bee_migrate.rs (recusa sobrescrever um arquivo existente)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> Nota: o parâmetro `pack --target` é uma interface reservada; o processo de empacotamento atual não distingue a plataforma de destino.
