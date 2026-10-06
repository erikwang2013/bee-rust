# 変更履歴

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.2.2] — 2026-10-06

### 追加
- `bee_orm`：`Model::create() -> Result<Self>`——挿入して完全なインスタンスを返す（データベースが割り当てた主キー入り。sqlite / postgres は `INSERT … RETURNING *`、mysql は `LAST_INSERT_ID()` + 読み直し）；`insert()` は変更なしで影響行数を返す
- `bee_orm`：`#[bee(crate = "…")]` 属性——derive が使う ORM クレートのパスを指定（`bee_rust` メタクレートだけに依存する場合は `#[bee(crate = "bee_rust::bee_orm")]`）。省略時の展開はバイト単位で不変
- feature で公開モジュールが切り替わる 8 つのクレートに docs.rs ビルド設定（`all-features = true`）を追加——バックエンド feature のドキュメントページと derive 属性表は docs.rs 上で見えていませんでした

## [1.2.1] — 2026-10-06

### 追加
- `bee_orm`：opt-in feature で制御する日付 / Decimal フィールド——`chrono`（`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`）と `rust_decimal`（`Decimal`）が三バックエンドで `Date` / `DateTime` / `DateTimeTz` / `Decimal` SQL 型にマップ；`bee_rust` は `orm-chrono` / `orm-rust_decimal` で転送（`full` には含まれない）
- `bee_orm`：これらの型の正直な境界——sqlite は TEXT 保存（`typeof` = `text`。DECIMAL 宣言は NUMERIC アフィニティを得て `"1.50"` を黙って REAL 1.5 にします）、mysql は `datetime(6)` / `timestamp(6)` のマイクロ秒を保持（TIMESTAMP は UTC として読み戻し、DATETIME は naive のまま）、pg `numeric` は有効桁 29 超で `NULL` の読み戻し；`NaiveTime` / `DateTime<Local>` / `FixedOffset` / ナノ秒は範囲外で `#[bee(sql_type = "…")]` が必要、feature を切るとモデルはコンパイル不可（E0277）

### 修正
- `bee_orm`：m2m の関係未宣言ヒントは表名ではなく型 ident（`#[bee(m2m(Author))]`）を綴るようになり、改名した表でもそのままコンパイルできます
- CI：`rust-toolchain` に明示的な `toolchain: stable` 入力を追加；feature なし構成で `clippy --all-targets` を壊していた 2 つのテストファイルにファイル単位 cfg ゲート（bee_cache / bee_kv）、`bee_orm` の m2m doctest はバックエンド非依存に；CLI `migrate` の E2E テストは内側ビルドをオンライン化（オフラインは温まったローカルキャッシュでのみ通っていました）し、`BEE_CLI_E2E=1` で CI に接続

## [1.2.0] — 2026-10-06

### 追加
- `bee_orm`：ORM がエンドツーエンドで動作——型付き `Value` パラメータ（Null / Bool / Int / Float / Text / Bytes）、`#[bee(table / column / pk / auto / ignore)]` 対応の `#[derive(Model)]`、モデルの `insert` / `update` / `delete`、そして `QuerySet` の実行（`all` / `one` / `count` / `exists` / `update` / `delete`）
- `bee_orm`：3 バックエンドすべてのコネクションプール（`pool::{sqlite, postgres, mysql}::Pool`）——`connect(dsn, max_size)`、`get()` → `CheckedConn`、`query` / `execute`、`status()`、`begin` / `commit` / `rollback` によるトランザクション。トランザクション途中で破棄された接続は sqlite と postgres でロールバックされます
- `bee_orm`：新 feature `postgres-tls`（TLS 経由の PostgreSQL、Mozilla ルート証明書同梱）とプールの強化——30 秒待機 / 10 秒作成のタイムアウトと、postgres プールのプリペアドステートメントキャッシュ
- `bee_orm`：モデルのライフサイクル——`#[bee(auto_now_add)]` / `#[bee(auto_now)]` による自動タイムスタンプ、`#[bee(soft_delete)]` によるソフト削除（delete はフラグを反転、`with_deleted()` / `hard_delete()` で回避可）、insert/update/delete フック、`Model::insert_many`（999 パラメータ単位の分割、非トランザクション）、そして `QuerySet::filter_in` と `sum` / `avg` / `min` / `max` 集計
- `bee_orm`：非破壊マイグレーション——`migrate::{create_table, add_missing_columns, sync}` が方言別 DDL（sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`）を生成し、テーブル作成と不足カラムの追加のみを行い、削除や変更はしません
- `bee_orm`：外部キーによるリレーション——`#[bee(fk = Target)]`（`#[bee(sql_type = "…")]` と併用）が FK DDL を生成し、`rel::{fk_column_to, belongs_to, children, children_for}` で読み取れます。強制はバックエンド依存：postgres と sqlite（bundled ビルド）は強制し、mysql は inline `REFERENCES` を無視します（round 5+ のギャップ）
- `bee_orm`：多対多リレーション——`#[bee(m2m(Target))]` で宣言し、`m2m::{attach, detach, related, related_for, related_ids}` で読み書き。join テーブルは `create_table` / `sync` が自動生成（`add_missing_columns` は触れません）
- `bee_orm`：JSON 列——`serde_json::Value` フィールドはバックエンドごとに `TEXT` / `JSONB` / `JSON` へマップ。NULL の意味論は正直：SQL `NULL` → `None`、保存された JSON `null` ドキュメント → `Some(Json::Null)`
- `bee_orm`：MySQL テーブルレベルの外部キー opt-in——`MigrateOptions { table_level_fk }` と `sync_with` / `create_table_with` / `add_missing_columns_with`；既定はオフで、既存の孤児行は黙ってスキップされず `ADD CONSTRAINT` が失敗します
- `bee_orm`：`Pool::connect_tls_with` で独自の `rustls::ClientConfig` を指定可能。`bee_orm::rustls` を再エクスポートしバージョン一致を保証（`connect_tls` は同梱 webpki ルートのまま）
- `bee_rust`：4 つの転送 feature——`orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` が `bee_rust` 経由で `bee_orm` バックエンドを転送（いずれも `full` に含まれません）
- `bee_cli`：`bee-rust migrate init` が `src/bin/bee_migrate.rs` を生成（既存ファイルは上書きしません）、`bee-rust migrate run` が `cargo run --bin bee_migrate` で実行
- `bee_kv` / `bee_cache`：Redis バックエンドは切断から復帰——`ConnectionManager` がオンデマンドで（バックグラウンドスレッドなし）指数バックオフとジッター付きで再接続（切断に当たったコマンドはエラー、次は新しい接続を待機）
- `bee_kv` / `bee_cache`：memcached バックエンド（bee_kv の `MemcacheStore`、bee_cache の `MemcacheCache`）；`incr` はカウンタを先に作成してから増減、符号なしカウンタは 0 で止まり、TTL 0 は削除

### 変更
- bee_orm のマイグレーション、リレーション（many-to-many を含む）、`bee-rust migrate` サブコマンド、Redis / Memcached バックエンド、`bee_rust` の ORM 転送 feature は、ドキュメント上もはや「計画中」ではありません
- ドキュメント上、search / graph / tsdb ドライバは「計画中 / trait のスタブ」から実装済み（opt-in feature）へ改め、旧例の型名と接続方式も修正（存在しない `ElasticsearchEngine` / `Neo4jDB`、誤った `bolt://`）

## [1.1.5] — 2026-09-25

### 追加
- 残り 11 言語の簡易版 crates.io README：`docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`。crates.io ページ上部の言語切り替えは、リポジトリの完全版 README ではなくこれらのファイルを指すようになり、どの言語でも同じ開発者向けの短い文書が得られます。

## [1.1.4] — 2026-09-25

### 変更
- crates.io ページ（`docs/crates-readme.md`）を中英二言語に変更：英語が上、中国語が下。crates.io は 1 クレートにつき 1 つの README しか描画できず、ページ内の言語切り替えもないため、英語を最初の画面に置きつつ中国語も同じページに残しています。

## [1.1.3] — 2026-09-25

### 追加
- crates.io ページ（`docs/crates-readme.md`）に 13 言語の切り替えリンクを追加（リポジトリ内の各言語 README へ）

### 修正
- `docs/api.*`：11 の翻訳に中国語原版（`api.md`）へのリンクが欠けていた
- `docs/CHANGELOG.zh.md` と `docs/CONTRIBUTING.zh.md` は、それぞれのドキュメント群で唯一セルフリンクを持たないファイルだった

## [1.1.2] — 2026-09-24

### 追加
- `docs/crates-readme.md`：crates.io ページ専用の README——インストール、実行可能なサンプル（`examples/hello` から取得、E2E テストでカバー）、feature フラグ表、サブクレート一覧
- `scripts/publish.sh`：ワークスペースのクレートを 1 つずつ公開し、アップロード済みのものはスキップ、レート制限時は crates.io が返す再試行時刻まで待機

### 修正
- テストスイートがコンパイルできなかった（`cargo test --workspace` が main で失敗）：`bee_router` の統合テストで `Submit` に `Serialize` が欠落（`Json` レスポンスとして返すため）、2 つのヘルパーに `Router::build()` が欠落、8 箇所の `status_line` 呼び出しに `&` が欠落
- `hello` サンプルのテストが誤った `CARGO_BIN_EXE` 名を参照——バイナリターゲットはハイフンを保持する（`CARGO_BIN_EXE_hello-bee`）
- `hello` のテンプレート自動エスケープテストが `&#39;` を検証していたが、tera は `&#x27;` を出力する。エスケープ動作自体は正しかった
- `bee_cli` の Clippy `manual_split_once` / `manual_range_contains` lint、`bee_router` 統合テストの未使用 import

### 変更
- すべてのクレートが `readme` と `repository` メタデータを持つように——以前は crates.io ページに README が表示されず、6 つのクレートは `repository` を完全に欠いていた
- `examples/hello` を `publish = false` に設定：E2E テストハーネスとしてワークスペースに残るが、crates.io には公開されない

## [1.0.6] — 2026-08-07

### 追加
- `bee_cli` の実装：`new`（プロジェクトのスキャフォールディング）、`generate controller/model`、`--watch` によるホットリロード付き `run`、`pack`（release ビルド + `dist/` へのコピー）
- スキャフォールディングとコード生成の CLI ユニットテスト（新規 7 テスト）

### 修正
- `bee_rust::init()` を `logs` feature の背後にゲート——feature を絞ったビルド（例：`--no-default-features --features kv`）が再びコンパイル可能に
- `bee_kv::InMemoryKvStore::exists` の Clippy `unnecessary_map_or` lint
- `rustfmt.toml` から stable で黙って無視されていた nightly 専用オプションを削除。ワークスペースは `cargo fmt --all --check` をパスするように
- `bee_cli` バイナリに `doc = false` を設定し、`bee_rust` との rustdoc 出力ファイル名の衝突を解消
- `hello` サンプルのポートが `PORT` 環境変数で設定可能に

### 変更
- `bee-rust migrate` が「未実装」を報告し、非ゼロで終了するように（計画中）
- README / README.en を実際の CLI の動作に合わせて更新

## [1.0.4] — 2026-07-29

### 追加
- `security-rust` によるセキュリティ攻撃検知フィルタ（27 検出器）
- XSS、SQL インジェクション、コマンドインジェクション、パストラバーサルをカバーする `SecurityFilter`
- `bee_rust` と `bee_router` に `security` feature フラグ

### 変更
- README にセキュリティ feature のドキュメントを追加
- README に投げ銭サポートのセクションを追加（WeChat Pay / Alipay）

### 修正
- Rust 2024 edition 向けの `bee_template` の Tera 生識別子構文

## [1.0.3] — 2026-07-29

### 追加
- 13 個の crate からなる初期ワークスペース構造
- `Controller` trait と `Router` による MVC ルーティング
- `QuerySet` ビルダーと `Model` 派生マクロを備えた ORM
- Redis と Memory バックエンドを備えた KV/Cache の trait 抽象
- Memory/Redis バックエンドによるセッション管理
- INI/YAML/ENV 対応とホットリロードを備えた設定管理
- Tera によるテンプレートレンダリング
- tracing 統合によるログ
- CLI のスキャフォールディングとコード生成
- 検索、グラフ、時系列エンジンの trait スタブ（ドライバは計画中）

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
