<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# BeeRust

[简体中文](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [English](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.md) · [한국어](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ko.md) · [Русский](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ru.md) · [Deutsch](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.de.md) · [Français](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.fr.md) · [Español](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.es.md) · [Português](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.pt.md) · [हिन्दी](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.hi.md) · [العربية](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ar.md) · [বাংলা](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.bn.md) · [Bahasa Indonesia](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.id.md) · [日本語](https://github.com/erikwang2013/bee-rust/blob/main/docs/crates-readme.ja.md)

Rust 製の本番向け Web フレームワーク。Go の Beego の設計哲学を、Rust らしい trait / macro / 型システムで表現し直したものです。

MVC コントローラ · 名前空間ルーティング · フィルタチェーン · ORM · ストレージエンジンの統一 trait 抽象

## インストール

```bash
cargo add bee_rust
```

または `Cargo.toml` に記述：

```toml
[dependencies]
bee_rust = "1.1.5"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

既定の `full` feature は `router` + `orm` + `kv` + `config` + `logs` + `cache` + `session` + `security` + `template` を有効にします。

## クイックスタート

```rust
async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> bee_rust::Result<()> {
    let _log_handle = bee_rust::init()?;

    let router = bee_rust::bee_router::Router::new()
        .ns("/api/v1", |ns| ns.get("/health", health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, router.build()).await?;
    Ok(())
}
```

```bash
cargo run
curl http://localhost:8080/api/v1/health    # OK
```

完全に動作するプロジェクトは [examples/hello](https://github.com/erikwang2013/bee-rust/tree/main/examples/hello) にあります（E2E テスト付き）。

## Feature フラグ

`search`、`graph`、`tsdb` は既定の `full` に**含まれません**。明示的に有効化してください。

| Feature | 読み込まれるもの | 説明 |
|---------|-----------|-------|
| `full` *(既定)* | router, orm, kv, config, logs, cache, session, security, template | オールインワン |
| `router` | bee_router, bee_session, bee_template, bee_config, bee_logs | Web コア：ルーティング + コントローラ + フィルタチェーン |
| `orm` | bee_orm, bee_config, bee_cache | `#[derive(Model)]` + QuerySet |
| `kv` | bee_kv | KV 抽象 |
| `cache` | bee_cache, bee_config | キャッシュ抽象 |
| `session` | bee_session, bee_cache | マルチバックエンドセッション |
| `config` | bee_config | INI / YAML / ENV + ホットリロード |
| `logs` | bee_logs | レベル別ログ + tracing |
| `template` | bee_template | tera テンプレート |
| `security` | bee_router/security | 27 種の攻撃検知フィルタ |
| `search` | bee_search | Elasticsearch / OpenSearch / ClickHouse |
| `graph` | bee_graph | Neo4j / NebulaGraph / ArangoDB |
| `tsdb` | bee_tsdb | InfluxDB / Apache IoTDB / QuestDB |

最小構成（ORM もデータベースドライバも不要）：

```toml
bee_rust = { version = "1.1.5", default-features = false, features = ["router", "logs", "config"] }
```

## サブクレート

本フレームワークは独立して使えるクレート群で構成され、`bee_rust` はそれらを必要に応じて再エクスポートする単一の入口です。

| Crate | 機能 | Beego 対応 |
|-------|--------------|-------------------|
| `bee_rust` | メタクレート、単一の入口 | — |
| `bee_router` | ルーティング + コントローラ + `Context` + フィルタ | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + マイグレーション | `client/orm` |
| `bee_config` | 設定 + ホットリロード | `client/config` |
| `bee_logs` | ログ | `logs` |
| `bee_cache` | キャッシュ抽象 | `client/cache` |
| `bee_session` | セッション管理 | `server/web/session` |
| `bee_template` | テンプレートレンダリング | — （拡張） |
| `bee_kv` | KV/キャッシュ抽象 | `client/cache` （拡張） |
| `bee_search` | 検索 / 分析エンジン | — （新規） |
| `bee_graph` | グラフデータベース | — （新規） |
| `bee_tsdb` | 時系列データベース | — （新規） |
| `bee_cli` | スキャフォールディング / コード生成 / 開発ランナー | `bee` ツール |

これらは個別に依存することもできます。たとえば Elasticsearch クライアントだけを使う場合：

```toml
bee_search = { version = "1.1.5", features = ["elasticsearch"] }
```

## 対応データベース

| 種別 | データベース | Crate | Feature |
|------|-----------|-------|---------|
| リレーショナル | SQLite / PostgreSQL / MySQL / TiDB | `bee_orm` | `sqlite` / `postgres` / `mysql` |
| KV / キャッシュ | Redis / Memcached | `bee_kv` · `bee_cache` | `redis` / `memcache` |
| 検索 / 分析 | Elasticsearch / OpenSearch / ClickHouse | `bee_search` | `elasticsearch` / `opensearch` / `clickhouse` |
| グラフ | Neo4j / NebulaGraph / ArangoDB | `bee_graph` | `neo4j` / `nebulagraph` / `arangodb` |
| 時系列 | InfluxDB / Apache IoTDB / QuestDB | `bee_tsdb` | `influxdb` / `iotdb` / `questdb` |

## 動作環境

- Rust 1.80+ (MSRV)
- edition 2024

## リンク

- **API ドキュメント**: [docs.rs/bee_rust](https://docs.rs/bee_rust)
- **リポジトリ**: [github.com/erikwang2013/bee-rust](https://github.com/erikwang2013/bee-rust)
- **完全な API リファレンス（13 言語）**: [docs/](https://github.com/erikwang2013/bee-rust/tree/main/docs)

## ライセンス

Apache-2.0
