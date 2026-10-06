<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Référence API Beerust

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Retour au [README](README.fr.md)。

Ce document est la référence API du framework Beerust : usages de chaque module, exemples de code et description des interfaces.

## Cœur Web (bee_router)

### Définir un contrôleur

```rust
use bee_rust::prelude::*;

// Définition du contrôleur
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Enregistrement des routes

```rust
// Enregistrement des routes
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### API de Context

**Context fournit :**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — sortie de réponse
- `ctx.redirect()` — redirection
- `ctx.abort()` — interruption de la requête
- `ctx.session` — accès à la session
- `ctx.params` — paramètres de chemin

## Détection de sécurité (feature `security`)

Un filtre de détection d'attaques basé sur [security-rust](https://crates.io/crates/security-rust), couvrant 27 types d'attaques dont XSS, injection SQL, injection de commandes et SSRF :

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // les 27 détecteurs sont tous activés
```

Pour l'activer dans `Cargo.toml` :
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
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql partagent la même forme

    // Schéma — migrate::sync génère un DDL non destructif à partir des métadonnées du modèle
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT — la clé primaire `auto` est attribuée par la base
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — filtres chaînés + exécution : all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // Relations — children pour has_many, belongs_to pour le parent ; sous contrainte FK, supprimez l'enfant avant le parent
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete est logique (bascule le drapeau) ; with_deleted() / hard_delete() le contournent
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

    // Transactions — gardez une connexion issue de `get()` ; le `CheckedConn` de sqlite est synchrone
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> Les migrations sont disponibles : `bee_orm::migrate` (`create_table` / `add_missing_columns` / `sync`) génère le DDL selon le dialecte — création de tables et ajout des colonnes manquantes, jamais de suppression ni de modification. Les lectures de relations couvrent `belongs_to`, has_many (`children*`) et many-to-many (`m2m`). `#[bee(fk = …)]` émet le DDL de clé étrangère : postgres et sqlite (build bundled) font respecter la référence, mysql peut activer les clés étrangères au niveau table via opt-in (ci-dessous).

### Many-to-many (`m2m`)

Déclaré au niveau du struct avec `#[bee(m2m(Target))]` ; la table de jointure est créée par `create_table` / `sync` (synchronisez d'abord le modèle cible, puis le déclarant) :

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // table de jointure user_tag, colonnes user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // le modèle cible d'abord
migrate::sync::<User, _>(&pool).await?;      // le déclarant crée la table de jointure

// Les ids `auto` sont attribués par la base : relisez l'instance avant de toucher aux relations
User { id: 0, name: "alice".into() }.insert(&pool).await?;
let user = User::query().filter_eq("name", "alice")?.one(&pool).await?.expect("inserted");
Tag { id: 0, name: "rust".into() }.insert(&pool).await?;
let tag = Tag::query().filter_eq("name", "rust")?.one(&pool).await?.expect("inserted");

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // attach en double -> erreur de PK composite
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // idempotent : un second appel renvoie 0 ligne
```

> Renommages : `#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]` ; les tables de jointure ne sont créées que par `create_table` / `sync` — `add_missing_columns` n'y touche jamais.

### Colonnes JSON

Les champs `serde_json::Value` deviennent sqlite `TEXT` / postgres `JSONB` / mysql `JSON` :

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // obligatoire
    extra: Option<serde_json::Value>,  // nullable
}
```

> Sémantique NULL : SQL `NULL` se décode en `None` ; un document JSON `null` stocké en `Some(Json::Null)` — les deux diffèrent.

### Clés étrangères au niveau table sous MySQL (opt-in)

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // mysql uniquement
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> Activée, les tables neuves et les colonnes ajoutées portent une `FOREIGN KEY` au niveau table (nom de contrainte `{table}_{column}_fk`) ; des lignes orphelines préexistantes font échouer `ADD CONSTRAINT` (pas d'omission silencieuse, pas de retrofit des colonnes existantes) ; sous pg / sqlite l'option est sans effet (inline l'applique déjà).

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls utilise les racines webpki embarquées ; pour une config rustls maison, connect_tls_with :
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig (réexporté, même version que la crate)
```

## Gestion de configuration (bee_config)

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

## Moteurs de stockage

**KV/Cache :**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// backend memcached (feature `memcached`)
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> `RedisStore` s'appuie sur un `ConnectionManager` : une connexion coupée se rétablit seule (à la demande — sans thread d'arrière-plan — avec backoff exponentiel et jitter) — la commande qui tombe sur la coupure échoue, la suivante attend la nouvelle connexion. memcached : `incr` crée d'abord le compteur (0 s'il est absent) puis applique le delta ; les compteurs sont non signés, donc décrémenter s'arrête à 0 (Redis descend en négatif) ; `expire(≤0)` équivaut à une suppression.

**Backends de cache (bee_cache) :**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache` (feature `memcache`) expose la même interface ; le TTL `Some(0)` passe par `DEL` côté Redis (`SET … EX 0` est refusé) et vaut aussi suppression côté memcached (là, 0 signifie ne jamais expirer) ; `incr` sur une valeur non numérique est une erreur de sérialisation.

**Moteur de recherche (implémenté, opt-in) :**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // nécessite la feature `elasticsearch`
let engine = Elasticsearch::new("http://localhost:9200");   // constructeur synchrone (sans `?`)
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> Les autres drivers fonctionnent de la même façon et sont opt-in eux aussi : `opensearch`, `clickhouse` (features de même nom).

**Base de données graphe (implémentée, opt-in) :**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // nécessite la feature `neo4j`
let db = Neo4j::new("http://localhost:7474");   // constructeur synchrone (sans `?`)
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> Les autres drivers fonctionnent de la même façon et sont opt-in eux aussi : `nebulagraph`, `arangodb` (features de même nom).

**Séries temporelles (implémentées, opt-in) :**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // nécessite la feature `influxdb`
let tsdb = InfluxDB::new("http://localhost:8086");   // constructeur synchrone (sans `?`)
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> Les autres drivers fonctionnent de la même façon et sont opt-in eux aussi : `iotdb`, `questdb` (features de même nom).

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> Le backend est n'importe quel cache implémentant `bee_cache::Cache` : `MemoryCache` / `RedisCache` / `MemcacheCache`.

## Journalisation

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

## Outil CLI

```bash
# Crée un projet (génère un scaffolding exécutable : Cargo.toml + src/main.rs)
bee-rust new my-app

# Génération de code
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Exécution de développement (--watch surveille src/ et redémarre automatiquement)
bee-rust run
bee-rust run --watch

# Empaquetage et déploiement (cargo build --release + copie dans dist/)
bee-rust pack

# Migration de base de données (génère le point d'entrée dans votre crate puis l'exécute)
bee-rust migrate init    # crée src/bin/bee_migrate.rs (refuse d'écraser un fichier existant)
bee-rust migrate run     # cargo run --bin bee_migrate
```

> Note : le paramètre `pack --target` est une interface réservée ; le processus d'empaquetage actuel ne distingue pas les plateformes cibles.
