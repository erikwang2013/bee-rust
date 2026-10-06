<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API リファレンス

[简体中文](api.md) · [English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

README に戻る：[README](README.ja.md)。

このドキュメントは Beerust フレームワークの API リファレンスです：各モジュールの使い方、コード例、インターフェースの説明。

## Web コア（bee_router）

### コントローラの定義

```rust
use async_trait::async_trait;  // Cargo.toml に async-trait = "0.1" を追加
use bee_rust::bee_router::context::RouterError;
use bee_rust::prelude::*;

// コントローラを定義
struct UserController;

#[async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### ルートの登録

```rust
// handler は axum handler——ステートなしなら通常の async fn
async fn list_users() -> &'static str { "[]" }
async fn create_user() -> &'static str { "created" }

// ルートの登録
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users", list_users)
          .post("/users", create_user);
    });
```

> パス構文は axum 0.8 準拠：動的セグメントは `{name}`（例：`/users/{id}`）。`:name` は build 時に panic（実測）。`ns` プレフィックスはグループ内パスとそのまま連結：`ns("/api/v1", …)` + `/users` → `/api/v1/users`。空プレフィックス（`ns("", …)`）も有効で、グループ内パスがそのまま全体パスになります。

### ステート付き handler

handler は axum handler です：共有ステートは `State` で注入し `Router::with_state` で組み立てます（`axum` は直接依存が必要——CLI スキャフォールドの Cargo.toml には含まれます）。ステートなしのルートは `build()`；handler が `State` を取る場合は `axum::serve` の前に必ず `with_state`。典型的な `Pool + Cache` の形：

```rust
use std::sync::Arc;
use axum::extract::State;
use bee_rust::bee_cache::MemoryCache;
use bee_rust::bee_orm::pool::sqlite::Pool;
use bee_rust::prelude::*;

#[derive(Model)]
#[bee(crate = "bee_rust::bee_orm", table = "users")]  // bee_rust のみに依存する場合の書き方
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

#[derive(Clone)]
struct AppState {
    pool: Pool,
    cache: Arc<dyn Cache>,
}

async fn list_users(State(state): State<AppState>) -> Result<String, String> {
    let users = User::query().all(&state.pool).await.map_err(|e| e.to_string())?;  // クエリ 1 回
    Ok(format!("{} users", users.len()))
}

let app = Router::new()
    .ns("/api/v1", |ns| ns.get("/users", list_users))
    .with_state(AppState { pool, cache: Arc::new(MemoryCache::new()) });
```

### Context API

**Context が提供するもの：**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — レスポンス出力
- `ctx.redirect()` — リダイレクト
- `ctx.abort()` — リクエストの中断
- `ctx.session` — セッションへのアクセス
- `ctx.params` — パスパラメータ

## セキュリティ検知（`security` feature）

[security-rust](https://crates.io/crates/security-rust) に基づく攻撃検知フィルタ。XSS、SQL インジェクション、コマンドインジェクション、SSRF など 27 種類の攻撃タイプをカバーします：

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // 27 個の検出器をすべて有効化
```

`Cargo.toml` で有効化：
```toml
bee_rust = { features = ["security"] }
```

## ORM（bee_orm）

> `bee_rust` のみに依存する場合：モデルに `#[bee(crate = "bee_rust::bee_orm")]` を付ける（推奨）か、モジュール先頭で `use bee_rust::bee_orm;`——展開は既定で `bee_orm::…` パスを出力するため、どちらかで解決できます。他の `bee_orm::` パスは `bee_rust::bee_orm::` に置き換え（以下は `bee_orm` 直接依存の書き方）。

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
    let pool = Pool::connect("app.db", 8)?;      // sqlite / postgres / mysql は同じ形

    // スキーマ — migrate::sync がモデルメタデータから非破壊 DDL を生成
    migrate::sync::<User, _>(&pool).await?;
    migrate::sync::<Post, _>(&pool).await?;

    // INSERT——insert() は影響行数を返す
    let user = User { id: 0, name: "alice".into(), age: Some(30), active: true, created_at: 0, deleted: false, cache: vec![] };
    user.insert(&pool).await?;

    // SELECT — チェーン式フィルタ + 実行：all / one / count / exists / sum / avg
    let alice = User::query().filter_eq("user_name", "alice")?.one(&pool).await?.expect("inserted above");
    let adults = User::query().filter_gt("age", 18)?.order_by("id").limit(20).all(&pool).await?;
    let total = User::query().filter_lt("age", 65)?.count(&pool).await?;
    let any = User::query().filter_contains("user_name", "a")?.exists(&pool).await?;
    let avg_age = User::query().avg(&pool, "age").await?;

    // リレーション — children が has_many、belongs_to が親行；FK 強制時は子を先に削除
    Post { id: 0, user_id: Some(alice.id), title: "hello".into(), deleted: false }.insert(&pool).await?;
    let posts = rel::children::<Post, User, _>(&pool, &alice).await?;
    let author = rel::belongs_to::<User, _>(&pool, alice.id).await?;
    let matching = Post::query().filter_in("title", &[Value::Text("hello".into())])?.all(&pool).await?;

    // UPDATE / DELETE — delete はソフト削除（フラグ反転）。with_deleted() / hard_delete() で回避
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

    // トランザクション — `get()` で接続を保持；sqlite の `CheckedConn` は同期
    let conn = pool.get()?;
    conn.begin()?;
    conn.execute("INSERT INTO users (user_name, active) VALUES (?, ?)", &[Value::from("carol"), Value::from(true)])?;
    conn.commit()?;

    Ok(())
}
```

> 書き込みは 2 形態：`insert()` は影響行数（`u64`）を返し、`create()` は挿入して行を読み直し、`auto` 主キー / 既定値が入ったインスタンスを返します（読み直せない場合は `NotFound`）——主キーが必要な場面は `create()` を（下の m2m 例）。

> マイグレーションが利用可能に：`bee_orm::migrate`（`create_table` / `add_missing_columns` / `sync`）が方言ごとの DDL を生成——テーブル作成と不足カラムの追加のみで、削除・変更は行いません。リレーション読み取りは `belongs_to`、has_many（`children*`）、many-to-many（`m2m`）に対応。`#[bee(fk = …)]` は外部キー DDL を生成します：postgres と sqlite（bundled ビルド）は強制、mysql はテーブルレベルの外部キーを opt-in で有効化できます（下記）。

### 多対多（`m2m`）

構造体レベルで `#[bee(m2m(Target))]` を宣言；join テーブルは `create_table` / `sync` が作成します（先に対象モデル、次に宣言側を同期）：

```rust
use bee_orm::m2m;

#[derive(Model)]
#[bee(table = "users")]
#[bee(m2m(Tag))]                             // join テーブル user_tag、列 user_id / tag_id
struct User {
    #[bee(pk, auto)]
    id: i64,
    name: String,
}

migrate::sync::<Tag, _>(&pool).await?;       // まず対象モデル
migrate::sync::<User, _>(&pool).await?;      // 宣言側が join テーブルを作成

// create() は挿入してインスタンスを読み直す（auto 主キー入り）——そのままリレーション操作へ
let user = User { id: 0, name: "alice".into() }.create(&pool).await?;
let tag = Tag { id: 0, name: "rust".into() }.create(&pool).await?;

m2m::attach::<User, Tag, _>(&pool, &user, &tag).await?;   // attach 重複 -> 複合主キーエラー
let tags = m2m::related::<User, Tag, _>(&pool, &user).await?;
m2m::detach::<User, Tag, _>(&pool, &user, &tag).await?;   // 冪等：2 回目の呼び出しは 0 行
```

> 名称の上書き：`#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]`；join テーブルは `create_table` / `sync` でのみ作成され、`add_missing_columns` は触れません。

### JSON 列

`serde_json::Value` フィールドは sqlite `TEXT` / postgres `JSONB` / mysql `JSON` にマップされます：

```rust
#[derive(Model)]
#[bee(table = "docs")]
struct Doc {
    #[bee(pk, auto)]
    id: i64,
    body: serde_json::Value,           // 必須
    extra: Option<serde_json::Value>,  // nullable
}
```

> NULL の意味論：SQL `NULL` は `None` に、保存された JSON `null` ドキュメントは `Some(Json::Null)` にデコードされます——両者は異なります。

### MySQL テーブルレベルの外部キー（opt-in）

```rust
use bee_orm::{MigrateOptions, migrate};

let opts = MigrateOptions { table_level_fk: true };       // mysql のみ
migrate::sync_with::<User, _>(&pool, opts).await?;
```

> 有効にすると新規テーブルと追加カラムにテーブルレベルの `FOREIGN KEY` が付きます（制約名 `{table}_{column}_fk`）；既存の孤児行があると `ADD CONSTRAINT` が失敗します（黙ってスキップせず、既存カラムの retrofit も行いません）；pg / sqlite ではこのオプションは無効です（inline が既に強制）。

### postgres TLS

```rust
use bee_orm::pool::postgres::Pool;

// connect_tls は同梱の webpki ルートを使用；独自の rustls 設定には connect_tls_with：
let pool = Pool::connect_tls_with(dsn, 8, my_rustls_config)?;
// my_rustls_config: bee_orm::rustls::ClientConfig（再エクスポート、crate と同じバージョン）
```

### 日付と Decimal 型

日付と Decimal フィールドは feature で制御します：`bee_orm` で `chrono`（`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`）と `rust_decimal`（`Decimal`）を有効化；`bee_rust` 経由では転送 feature `orm-chrono` / `orm-rust_decimal` が対応します。

```rust
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Model)]
#[bee(table = "events")]
struct Event {
    #[bee(pk, auto)]
    id: i64,
    at: NaiveDateTime,      // NaiveDate -> Date、DateTime<Utc> -> DateTimeTz
    amount: Decimal,        // -> Decimal（3 バックエンド共通）
}

// 書き込み：フィールドは対応する Value 変体に；読み込み：セルは serde で復元
let at = "2026-10-06T12:00:00".parse::<NaiveDateTime>().expect("valid");
let amount = "1.50".parse::<Decimal>().expect("valid");
Event { id: 0, at, amount }.insert(&pool).await?;
let back = Event::query().all(&pool).await?;   // 「1.50」は数値として等しく往復
```

> 対応：`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>` / `Decimal`。範囲外——マクロは SQL マッピングを出しません。`#[bee(sql_type = "…")]` が逃げ道です：`NaiveTime` / `DateTime<Local>` / `FixedOffset` / 素の `DateTime`、およびナノ秒精度。
> 保存形式：sqlite は常に TEXT（`typeof(col)` は `text`。Decimal も同様——DECIMAL 宣言は NUMERIC アフィニティになり「1.50」を REAL 1.5 に吞みます）；mysql は `datetime(6)` / `timestamp(6)` でマイクロ秒保持——TIMESTAMP は UTC として読み戻し（RFC3339 の `Z` 付き。厳密な往復にはセッションタイムゾーン UTC が前提）、DATETIME は naive のまま接尾辞なし、列型で分派；pg の `timestamptz` は UTC に正規化。
> 精度の天井：pg の `numeric` は有効桁 29 を超えると `NULL` で読み戻り（ネイティブ FromSql が拒否）；mysql はテキスト経由なので超過分はデコード時に serde でエラー；金額には `#[bee(sql_type = Raw("decimal(12,2)"))]` を推奨。
> コンパイル時契約：feature を切ると日付フィールドのモデルは E0277（`Value: From<NaiveDate>` 未充足）で失敗——黙って誤った列にはなりません。

## 設定管理（bee_config）

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

## ストレージエンジン

**KV/Cache：**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", "value").await?;
kv.expire("key", 60).await?;
let val = kv.get("key").await?;

// memcached バックエンド（feature `memcached`）
let kv = MemcacheStore::new("127.0.0.1:11211")?;
```

> `RedisStore` の内部は `ConnectionManager`：切断された接続は自動再接続されます（コマンド契機——バックグラウンドスレッドなし——指数バックオフとジッター付き）——切断に当たったコマンドはエラーになり、次のコマンドは新しい接続を待ちます。memcached：`incr` はまずカウンタを作成し（無ければ 0）、その後で増減します；カウンタは符号なしで、減算は 0 で止まります（Redis は負になります）；`expire(≤0)` は削除と同じです。

**キャッシュバックエンド（bee_cache）：**
```rust
use bee_cache::{Cache, RedisCache};

let cache = RedisCache::new("redis://localhost:6379").await?;  // feature `redis`
cache.set("k", b"v".to_vec(), Some(60)).await?;
let v = cache.get("k").await?;
```

> `MemcacheCache`（feature `memcache`）は同じインターフェース；TTL `Some(0)` は Redis バックエンドでは `DEL` を通ります（`SET … EX 0` は拒否されます）、memcached でも削除として扱われます（そちらでは 0 は無期限を意味します）；非数値への `incr` はシリアライズエラーです。

**検索エンジン（実装済み、opt-in）：**
```rust
use bee_search::SearchEngine;
use bee_search::elasticsearch::Elasticsearch;   // feature `elasticsearch` が必要
let engine = Elasticsearch::new("http://localhost:9200");   // 同期コンストラクタ（`?` 不要）
let result = engine.search("my_index", serde_json::json!({"query": {"match_all": {}}})).await?;
```

> ほかのドライバも同じ書き方で、いずれも opt-in です：`opensearch`, `clickhouse`（同名 feature）。

**グラフデータベース（実装済み、opt-in）：**
```rust
use bee_graph::{GraphDB, Vertex};
use bee_graph::neo4j::Neo4j;   // feature `neo4j` が必要
let db = Neo4j::new("http://localhost:7474");   // 同期コンストラクタ（`?` 不要）
let v = db.add_vertex(Vertex {
    id: "p1".into(),
    label: "Person".into(),
    properties: [("name".to_string(), serde_json::json!("Alice"))].into_iter().collect(),
}).await?;
```

> ほかのドライバも同じ書き方で、いずれも opt-in です：`nebulagraph`, `arangodb`（同名 feature）。

**時系列データベース（実装済み、opt-in）：**
```rust
use bee_tsdb::{Point, TimeSeriesDB};
use bee_tsdb::influxdb::InfluxDB;   // feature `influxdb` が必要
let tsdb = InfluxDB::new("http://localhost:8086");   // 同期コンストラクタ（`?` 不要）
tsdb.write_point(Point {
    measurement: "cpu".into(),
    tags: [("host".to_string(), "srv1".to_string())].into_iter().collect(),
    fields: [("value".to_string(), serde_json::json!(0.85))].into_iter().collect(),
    timestamp: chrono::Utc::now(),
}).await?;
```

> ほかのドライバも同じ書き方で、いずれも opt-in です：`iotdb`, `questdb`（同名 feature）。

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

> バックエンドは `bee_cache::Cache` を実装した任意のキャッシュです：`MemoryCache` / `RedisCache` / `MemcacheCache`。

## ログ

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## テンプレート

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI ツール

```bash
# プロジェクトを作成（実行可能なスキャフォールディングを生成：Cargo.toml + src/main.rs）
bee-rust new my-app

# コードを生成
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# 開発実行（--watch は src/ の変更を監視して自動再起動）
bee-rust run
bee-rust run --watch

# パッケージングしてデプロイ（cargo build --release + dist/ にコピー）
bee-rust pack

# データベースマイグレーション（対象クレートにエントリーポイントを生成して実行）
bee-rust migrate init    # src/bin/bee_migrate.rs を生成（既存ファイルは上書きしません）
bee-rust migrate run     # cargo run --bin bee_migrate
```

> 注記：`pack --target` パラメータは予約済みのインターフェースで、現在のパッケージング処理はターゲットプラットフォームを区別しません。
