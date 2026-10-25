# Log Perubahan

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

> Pemberian versi: selama proyek masih 1.x, rilis minor dapat memuat perubahan yang memutus API; pin versi persis jika Anda melakukan match `OrmError` secara menyeluruh. Lihat 1.3.0 dan 1.6.0.

## [1.6.1] — 2026-10-26

### Keamanan
- Penyegaran dependensi menutup 7 kerentanan RUSTSEC dan 6 peringatan audit. `rustls` 0.23.46, `rustls-webpki` 0.103.16, `webpki-roots` 1.0.11, `ctutils` 0.4.4, `tokio` 1.54.1, `lru` 0.18.5, dan `yoke` / `yoke-derive` 0.8.4 menggantikan versi yang terdampak; `mysql_async` 0.37.1 dan `redis` 1.6 / `deadpool-redis` 0.23.1 menghapus rantai `rustls-pemfile`, `proc-macro-error2`, dan `lru` lama yang tidak terpelihara. `cargo audit` kini melaporkan 0 kerentanan dan 0 peringatan.

### Diubah
- `bee_kv` / `bee_cache`: URL Sentinel pertama kini juga memasok kredensial ACL, database, protokol, dan TLS koneksi master melalui `SentinelNodeConnectionInfo`; master yang dilindungi ACL atau hanya-TLS berfungsi bila URL Sentinel-nya membawa pengaturan yang sama. Koneksi yang dipool tetap hanya didaur ulang dengan PING, bukan diperiksa perannya.
- Peningkatan driver MySQL ke `mysql_async` 0.37 tidak mengubah perilaku yang teramati dan telah diverifikasi terhadap MySQL 5.7.44 / 8.0.46 / 8.4.11 / 9.7.4 dan MariaDB 10.11.19.

## [1.6.0] — 2026-10-25

### Ditambahkan
- `bee_orm`: ergonomi sharding — `#[bee(shard_key)]` menandai field routing (paling banyak satu; ditolak pada field `ignore` dan field `auto` / `auto_now_add` / `auto_now` yang ditetapkan database, karena routing terjadi sebelum insert), `Model::shard_key()` mengeksposnya (`None` / `NULL` tidak dapat dirutekan), dan `ShardedPools::for_model` merutekan instance tanpa mengulang key. `ShardedPools::all_shards()` mengiterasi pool yang dikonfigurasi, dan `sync_all_parallel` memigrasikan setiap pool × shard tabel secara bersamaan (DDL yang sudah diterapkan tidak di-rollback).
- `bee_orm`: `OrmError::ShardError` memisahkan penyalahgunaan sharding dari kegagalan kueri; `OrmError` kini `#[non_exhaustive]` — match eksternal memerlukan arm wildcard.
- `bee_orm`: `PoolStatus` lintas backend dengan `pool_status()` pada pool sqlite, postgres, dan mysql (`size` / `available` bernilai `None` di mysql yang tidak mengekspos statistik), plus `TimedDb` untuk callback kueri lambat. Benchmark Criterion mencakup hashing router dan rendering SQL QuerySet.
- `bee_kv` / `bee_cache`: `RedisLock::acquire(key, ttl, wait)` mengulang percobaan lock paling sering setiap 50ms hingga batas tunggu habis; `RedisPoolStatus` / `pool_status()` memberikan snapshot pool milik crate; feature `redis-tls` mengaktifkan koneksi `rediss://` dan kredensial ACL ikut dalam URL.
- `bee_kv` / `bee_cache`: store/cache Sentinel (`RedisSentinelStore`, `RedisSentinelCache`) dan store/cache Cluster (`RedisClusterStore`, `RedisClusterCache`) di balik feature `redis-sentinel` / `redis-cluster`; `mget` / `mset` cluster berjalan sebagai perintah single-key untuk menghindari `CROSSSLOT`.
- `examples/sharding` dan `examples/redis-lock`; CI mengompilasi kedua contoh dan benchmark, mem-pin MySQL 9.7.4, dan mempertahankan job MariaDB.
- `bee_router`: budget latensi opt-in untuk GET/HEAD dengan fallback cache basi — `Router::latency(LatencyConfig::new(cache))` memakai default total 6 ms (4 ms handler + 2 ms cadangan), meng-cache respons GET 200 publik hingga 64 KiB, dan memutarnya kembali saat timeout (`X-Bee-Cache: stale`, `Age`) atau mengembalikan 504. GET yang timeout berlanjut di tugas latar belakang terbatas untuk menghangatkan cache, dan `LatencyStats` mengekspos penghitung plus P50/P99 perkiraan. Metode tulis mempertahankan perilaku 30 s/408.

### Diubah
- `OrmError` adalah `#[non_exhaustive]` dan kegagalan sharding kini melaporkan `ShardError` alih-alih `QueryError`; match di hilir memerlukan arm wildcard.

### Batasan
- Sentinel: koneksi master yang dipool didaur ulang dengan `PING`, bukan diperiksa perannya, sehingga setelah failover master yang diturunkan namun tetap terjangkau dapat melayani koneksi usang. Melalui `Config::from_urls` dari `deadpool-redis` 0.18 koneksi master juga tidak mendapat kredensial ACL atau TLS (kredensial/TLS dari URL Sentinel hanya berlaku untuk Sentinel); master yang dilindungi ACL atau hanya-TLS harus terhubung langsung.
- Cluster: perintah tidak dibatasi oleh `RedisOptions::response_timeout` — `deadpool-redis` 0.18 tidak mengekspos setter; checkout, create, dan recycle mempertahankan batas 30s / 5s / 5s masing-masing.
- Batas latensi: hanya GET/HEAD. Tenggat membatasi respons, bukan pekerjaan sisi server — kueri blocking SQLite tidak dapat dibatalkan dan perintah Postgres/MySQL/Redis terus berjalan. Respons yang di-cache hanya publik dan dapat basi hingga TTL; penulisan tidak pernah membatalkan cache; hanya body berukuran tepat ≤64 KiB yang di-cache; `MemoryCache` tidak terbatas. Rute GET long-poll/streaming sebaiknya memakai Router tanpa `latency`.

## [1.5.0] — 2026-10-25

### Ditambahkan
- `bee_orm`: dukungan MySQL 5.7 dan MariaDB 10.11+ — matriks CI kini mencakup 5.7 / 8.0 / 8.4 / 9.x plus job MariaDB; pengujian integrasi mengenali vendor server (MariaDB melaporkan JSON sebagai LONGTEXT dan menegakkan `REFERENCES` tingkat kolom; MySQL 5.7/8.x mem-parse dan mengabaikannya).
- `bee_orm`: `Model::create` di MySQL kini memakai metadata `columns()` milik model untuk melewati probe `information_schema` pada setiap insert; model tulisan tangan tanpa metadata tetap memakai probe lama. Petunjuk ini bersifat otoritatif — penyimpangan schema/model tidak didukung.
- `bee_orm::shard`: `ShardedPools::{pools, pool, len, is_empty}` mengekspos shard yang dikonfigurasi untuk operasi; `ShardHandle` meneruskan petunjuk auto-increment.
- `bee_kv` / `bee_cache`: metrik pool `status()`; `RedisOptions` publik (`max_size`, `response_timeout`) dengan `with_options`; `RedisLock::try_lock_with_watchdog` memperbarui TTL hingga release/drop (TTL watchdog minimal 30ms).
- Internal: gate CI konsistensi dokumentasi (`scripts/check-docs.sh`), skrip render pratinjau sosial, dan fixture pengujian bersama yang dipisahkan dari `tests/mod.rs`.

## [1.4.0] — 2026-10-25

### Ditambahkan
- `bee_orm`: kompatibilitas MySQL 8.0 / 8.4 / 9.x, diverifikasi oleh matriks CI. MySQL 9 menegakkan `REFERENCES` tingkat kolom (8.0/8.4 mem-parse dan mengabaikannya); buat tabel yang direferensikan terlebih dahulu.
- `bee_orm`: `shard::{ShardedPools, HashRouter, ShardRouter}` — sharding hash wajib kunci untuk basis data dan tabel; tidak ada kueri atau transaksi lintas shard.
- `bee_kv` / `bee_cache`: pool koneksi Redis sungguhan (`deadpool-redis`) dan `bee_kv::RedisLock` (`SET NX PX` + pelepasan Lua).

## [1.3.0] — 2026-10-13

### Diperbaiki
- `bee_tsdb` / `bee_graph`: mengaktifkan satu fitur backend saja tidak bisa dikompilasi — helper bersama `http_client` tinggal di dalam modul `influxdb` / `neo4j` yang digerbangi fitur, sehingga build hanya dengan `iotdb` / `questdb` / `arangodb` / `nebulagraph` gagal dengan unresolved import; kini ia tinggal di `crate::http` masing-masing crate (digerbangi persis pada backend yang memakainya), dan setiap fitur backend tunggal dapat dikompilasi sendiri — matriks fitur dijalankan ulang satu per satu
- `bee_router`: fitur opsional `security` pindah ke **security-rust 4.0.0** — `Scanner` / `DetectionResult` / `AttackCategory` / `Severity` tetap berbentuk sama seperti 1.x, tanpa perubahan sumber di sisi bee_router; permukaan kemampuan bertambah: 32 detektor bawaan (baru: `format_string`, `log4shell`, HPP, `formula_injection`, reDoS), detektor kini bertingkat kuat/lemah (sinyal lemah dipatok di `Severity::Low`), dan `Scanner::assess` ditambahkan. Filter tetap memblokir pada deteksi apa pun, jadi **sinyal lemah (mis. parameter query yang berulang) kini juga diblokir** — tidak ada lagi pintu untuk menyuntikkan scanner kustom; untuk meloloskannya, bangun `Filter` sendiri di sekitar `SecurityFilter::scanner()`. Graf dependensi kini membawa satu security-rust 4.0.0, bukan 1.x berdampingan dengan 4.x; dokumen disinkronkan
- duplikasi digabung: `bee_tsdb` / `bee_graph` / `bee_search` masing-masing menyisakan satu salinan helper koneksi dan pemetaan galat (nama backend kini dikirim pemanggil, teks galat tidak berubah; `aggregation_fn` pindah ke lib `bee_tsdb`), `backtick` / `esc` milik `bee_graph` menyatu ke modul `http`-nya, `bee_orm` mengekstrak `dedup_pk_values` / `index_rows_by` dan klausa foreign-key di `migrate`, `bee_session::refresh` kini memakai ulang `load`, dan `watch_path` di `bee_config` menyerap jalur pemanggilan `wait_for_change` yang tersisa; pesan galat `bee_search` kembali membawa nama backend, dan stub tes `bee_kv` kini privat untuk tesnya
- API publik dihapus (karena itu versi minor): `Context::set_params` / `param`, `SecurityFilter::with_scanner`, `Traversal::edge_labels` → `edge_label: Option<String>` (ketiga backend memang sudah hanya mengambil label pertama — perilaku tak berubah), `GraphError::EdgeNotFound`, `ConfigError::ParseError` / `TypeMismatch`, `bee_config::watch_path_timeout`, dan `bee_logs::Logger::async_` (sakelar yang tak pernah dibaca `init()`) — semuanya tanpa pemanggil
- dependensi mati dihapus: `bee_router` (`tracing` / `http` / `bee_config` / `bee_logs`), `bee_session` (`async-trait`), `bee_config_macro` (`proc-macro2`), dan `dev` `serde` yang duplikat di `bee_tsdb` / `bee_graph` / `bee_search`

## [1.2.3] — 2026-10-06

### Diperbaiki
- `bee_cli`: scaffold tidak lagi menghasilkan proyek yang gagal dikompilasi — template `new` menyebut paket yang tidak bisa diresolusi (`bee-rust`; dipublikasikan sebagai `bee_rust`) dan template controller tidak punya `#[async_trait]` serta memakai jalur akar `RouterError` yang privat; tes bergerbang `BEE_CLI_E2E` yang baru membuat proyek lalu menjalankan `cargo check` dari awal sampai akhir
- dokumen: contoh controller di api kini dalam bentuk yang bisa dikompilasi (dependensi `async-trait` sendiri, `bee_rust::bee_router::context::RouterError`); didokumentasikan pula penulisan `#[bee(crate = "bee_rust::bee_orm")]` untuk proyek yang hanya bergantung pada `bee_rust`, sintaks rute `{name}` dengan aturan prefiks kosong `ns`, dan penggunaan `create()`; satu fragmen Korea di dokumen api Bengali diperbaiki
- `examples/shortlink`: detail kesalahan internal masuk ke log (bodi respons hanya pesan umum), validasi URL menolak host kosong dan karakter kendali, dan `create_link` / `add_tag` kini memakai `Model::create()` (insert `Click` di jalur panas 302 tetap `insert()`)

## [1.2.2] — 2026-10-06

### Ditambahkan
- `bee_orm`: `Model::create() -> Result<Self>` — menyisipkan dan mengembalikan instance utuh, lengkap dengan primary key yang ditetapkan basis data (`INSERT … RETURNING *` di sqlite / postgres, `LAST_INSERT_ID()` + baca ulang di mysql); `insert()` tetap sama dan tetap mengembalikan jumlah baris terpengaruh
- `bee_orm`: atribut `#[bee(crate = "…")]` — menunjuk derive ke jalur crate ORM (mis. `#[bee(crate = "bee_rust::bee_orm")]` saat hanya metacrate `bee_rust` yang menjadi dependensi); tanpa atribut ini ekspansinya identik byte per byte
- konfigurasi build docs.rs dilengkapi untuk delapan crate dengan modul publik di balik feature (`all-features = true`) — halaman dokumen feature backend dan tabel atribut derive sebelumnya tidak terlihat di docs.rs

## [1.2.1] — 2026-10-06

### Ditambahkan
- `bee_orm`: field tanggal / Decimal di balik feature opt-in — `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) dan `rust_decimal` (`Decimal`) memetakan ke tipe SQL `Date` / `DateTime` / `DateTimeTz` / `Decimal` di ketiga backend; `bee_rust` meneruskannya sebagai `orm-chrono` / `orm-rust_decimal` (tidak termasuk `full`)
- `bee_orm`: batas jujur tipe-tipe ini — sqlite menyimpan sebagai TEXT (`typeof` = `text`; deklarasi DECIMAL akan mendapat afinitas NUMERIC dan diam-diam mengubah `"1.50"` jadi REAL 1.5), mysql mempertahankan mikrodetik `datetime(6)` / `timestamp(6)` (TIMESTAMP dibaca sebagai UTC, DATETIME tetap naive), dan pg `numeric` di atas 29 digit signifikan dibaca `NULL`; `NaiveTime` / `DateTime<Local>` / `FixedOffset` / nanodetik di luar cakupan dan butuh `#[bee(sql_type = "…")]`, dan dengan feature mati model gagal dikompilasi (E0277)

### Diperbaiki
- `bee_orm`: petunjuk relasi m2m yang hilang kini menuliskan ident tipe (`#[bee(m2m(Author))]`) alih-alih nama tabel, jadi saran tetap bisa dikompilasi untuk tabel yang diganti nama
- CI: `rust-toolchain` mendapat input eksplisit `toolchain: stable`; dua berkas tes yang merusak `clippy --all-targets` tanpa features mendapat gate cfg tingkat berkas (bee_cache / bee_kv) dan doctest m2m `bee_orm` menjadi agnostik backend; tes end-to-end `migrate` CLI kini membangun scratch crate secara online (offline hanya lolos dengan cache lokal hangat) dan berjalan di CI lewat `BEE_CLI_E2E=1`

## [1.2.0] — 2026-10-06

### Ditambahkan
- `bee_orm`: ORM kini berjalan menyeluruh — parameter `Value` bertipe (Null / Bool / Int / Float / Text / Bytes), `#[derive(Model)]` dengan `#[bee(table / column / pk / auto / ignore)]`, `insert` / `update` / `delete` pada model, dan eksekusi `QuerySet` (`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm`: connection pool untuk ketiga backend (`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)`, `get()` → `CheckedConn`, `query` / `execute`, `status()`, serta transaksi lewat `begin` / `commit` / `rollback`; koneksi yang dilepas di tengah transaksi di-rollback pada sqlite dan postgres
- `bee_orm`: fitur baru `postgres-tls` (PostgreSQL lewat TLS, root Mozilla bawaan) dan penguatan pool — timeout 30 detik menunggu / 10 detik membuat koneksi serta cache prepared statement di pool postgres
- `bee_orm`: siklus hidup model — stempel waktu `#[bee(auto_now_add)]` / `#[bee(auto_now)]`, soft delete `#[bee(soft_delete)]` (delete membalik flag; `with_deleted()` / `hard_delete()` melewatinya), hook insert/update/delete, `Model::insert_many` (potongan 999 parameter, non-transaksional), serta `QuerySet::filter_in` dan agregat `sum` / `avg` / `min` / `max`
- `bee_orm`: migrasi non-destruktif — `migrate::{create_table, add_missing_columns, sync}` menghasilkan DDL sesuai dialek (sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`): membuat tabel dan menambah kolom yang belum ada, tidak pernah menghapus atau mengubah
- `bee_orm`: relasi kunci asing — `#[bee(fk = Target)]` (dengan `#[bee(sql_type = "…")]`) menghasilkan DDL FK dan `rel::{fk_column_to, belongs_to, children, children_for}` membacanya; penegakan bergantung backend: postgres dan sqlite (build bundled) menegakkannya, mysql mengabaikan `REFERENCES` inline (celah untuk round 5+)
- `bee_orm`: relasi banyak-ke-banyak — `#[bee(m2m(Target))]` mendeklarasikan relasi, `m2m::{attach, detach, related, related_for, related_ids}` membacanya, dan `create_table` / `sync` membuat tabel join (`add_missing_columns` tidak menyentuhnya)
- `bee_orm`: kolom JSON — field `serde_json::Value` dipetakan ke `TEXT` / `JSONB` / `JSON` sesuai backend, dengan semantik NULL yang jujur: SQL `NULL` → `None`, dokumen JSON `null` tersimpan → `Some(Json::Null)`
- `bee_orm`: kunci asing tingkat tabel MySQL (opt-in) — `MigrateOptions { table_level_fk }` dengan `sync_with` / `create_table_with` / `add_missing_columns_with`; mati secara default, dan baris yatim yang sudah ada membuat `ADD CONSTRAINT` gagal alih-alih dilewati senyap
- `bee_orm`: `Pool::connect_tls_with` untuk `rustls::ClientConfig` kustom, plus reekspor `bee_orm::rustls` agar versinya selalu cocok; `connect_tls` tetap memakai root webpki bawaan
- `bee_rust`: empat feature penerusan — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` meneruskan backend `bee_orm` melalui `bee_rust` (tak satu pun di `full`)
- `bee_cli`: `bee-rust migrate init` membuat `src/bin/bee_migrate.rs` (menolak menimpa file yang ada), dan `bee-rust migrate run` menjalankannya lewat `cargo run --bin bee_migrate`
- `bee_kv` / `bee_cache`: backend Redis bertahan dari koneksi terputus — `ConnectionManager` di dalamnya menyambung ulang sesuai permintaan (tanpa utas latar belakang) dengan backoff eksponensial dan jitter (perintah yang kena putus error, berikutnya menunggu koneksi baru)
- `bee_kv` / `bee_cache`: backend memcached (`MemcacheStore` di bee_kv, `MemcacheCache` di bee_cache); `incr` membuat counter sebelum delta, counter tak bertanda berhenti di 0, dan TTL 0 menghapus kunci

### Diubah
- Migrasi dan relasi bee_orm (termasuk many-to-many), subperintah `bee-rust migrate`, backend Redis / Memcached, dan feature penerusan ORM `bee_rust` tidak lagi ditandai direncanakan di dokumentasi
- driver search / graph / tsdb tidak lagi ditandai di dokumentasi sebagai direncanakan atau trait-stub, melainkan terimplementasi (feature opt-in), dan contoh lama diperbaiki (nama tipe yang tidak ada `ElasticsearchEngine` / `Neo4jDB`, skema `bolt://` yang salah)

## [1.1.5] — 2026-09-25

### Ditambahkan
- README crates.io ringkas untuk 11 bahasa lainnya: `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`. Pengalih bahasa di halaman crates.io kini menunjuk ke berkas-berkas ini, bukan ke README lengkap di repositori, sehingga setiap bahasa mendapat dokumen singkat yang sama untuk pengembang.

## [1.1.4] — 2026-09-25

### Diubah
- Halaman crates.io (`docs/crates-readme.md`) kini dwibahasa: bahasa Inggris di atas, bahasa Tionghoa di bawah. crates.io hanya merender satu README per crate dan tidak menyediakan pengalih bahasa di dalam halaman, sehingga bahasa Inggris tampil di layar pertama sementara teks Tionghoa tetap di halaman yang sama.

## [1.1.3] — 2026-09-25

### Ditambahkan
- Halaman crates.io (`docs/crates-readme.md`) kini memiliki pemilih 13 bahasa yang menautkan ke README di repositori

### Diperbaiki
- `docs/api.*`: 11 terjemahan tidak memiliki tautan ke versi asli Tionghoa (`api.md`)
- `docs/CHANGELOG.zh.md` dan `docs/CONTRIBUTING.zh.md` adalah satu-satunya berkas di kelompoknya yang tanpa tautan ke dirinya sendiri

## [1.1.2] — 2026-09-24

### Ditambahkan
- `docs/crates-readme.md`: README khusus untuk halaman crates.io — instalasi, contoh yang bisa dijalankan (diambil dari `examples/hello`, yang dicakup oleh tes E2E-nya), tabel feature flag, dan daftar sub-crate
- `scripts/publish.sh`: menerbitkan crate workspace satu per satu, melewati yang sudah diunggah, dan saat kena rate limit menunggu waktu coba-ulang yang dikembalikan crates.io

### Diperbaiki
- Test suite tidak bisa dikompilasi (`cargo test --workspace` gagal di main): pada tes integrasi `bee_router`, `Submit` kehilangan `Serialize` (dikembalikan sebagai respons `Json`), dua helper kehilangan `Router::build()`, dan delapan pemanggilan `status_line` kehilangan `&`
- Tes contoh `hello` merujuk nama `CARGO_BIN_EXE` yang salah — target biner mempertahankan tanda hubungnya (`CARGO_BIN_EXE_hello-bee`)
- Tes autoescape template `hello` memeriksa `&#39;` padahal tera menghasilkan `&#x27;`; perilaku escape-nya sendiri sudah benar
- Lint Clippy `manual_split_once` dan `manual_range_contains` di `bee_cli`; import tak terpakai di tes integrasi `bee_router`

### Diubah
- Semua crate kini membawa metadata `readme` dan `repository` — sebelumnya tidak ada halaman crates.io yang merender README dan enam crate sama sekali tidak punya `repository`
- `examples/hello` ditandai `publish = false`: tetap berada di workspace sebagai harness tes E2E tetapi tidak lagi diterbitkan ke crates.io

## [1.0.6] — 2026-08-07

### Ditambahkan
- Implementasi nyata `bee_cli`: `new` (scaffolding proyek), `generate controller/model`, `run` dengan hot reload `--watch`, `pack` (build release + salin ke `dist/`)
- Pengujian unit CLI untuk scaffolding dan generasi kode (7 pengujian baru)

### Diperbaiki
- `bee_rust::init()` kini dikurung di balik fitur `logs` — build dengan fitur tereduksi (mis. `--no-default-features --features kv`) dapat dikompilasi lagi
- Lint Clippy `unnecessary_map_or` di `bee_kv::InMemoryKvStore::exists`
- `rustfmt.toml` menghapus opsi khusus nightly yang sebelumnya diabaikan diam-diam di stable; workspace kini lolos `cargo fmt --all --check`
- Biner `bee_cli` diatur `doc = false` untuk menghilangkan bentrok nama berkas output rustdoc dengan `bee_rust`
- Port contoh `hello` kini dapat dikonfigurasi melalui variabel env `PORT`

### Diubah
- `bee-rust migrate` melaporkan "not implemented" dan keluar dengan kode non-nol (direncanakan)
- README / README.en diperbarui untuk menjelaskan perilaku CLI yang sebenarnya

## [1.0.4] — 2026-07-29

### Ditambahkan
- Filter deteksi serangan keamanan melalui `security-rust` (27 detektor)
- `SecurityFilter` dengan cakupan XSS, injeksi SQL, injeksi perintah, path traversal
- Flag fitur `security` di `bee_rust` dan `bee_router`

### Diubah
- README diperbarui dengan dokumentasi fitur keamanan
- README diperbarui dengan bagian dukungan donasi (WeChat Pay / Alipay)

### Diperbaiki
- Sintaks raw identifier Tera di `bee_template` untuk edisi Rust 2024

## [1.0.3] — 2026-07-29

### Ditambahkan
- Struktur workspace awal dengan 13 crate
- Routing MVC dengan trait `Controller` dan `Router`
- ORM dengan builder `QuerySet` dan macro turunan `Model`
- Abstraksi trait KV/Cache dengan backend Redis dan Memory
- Manajemen sesi dengan backend Memory/Redis
- Manajemen konfigurasi dengan dukungan INI/YAML/ENV dan hot-reload
- Rendering template melalui Tera
- Logging dengan integrasi tracing
- Scaffolding CLI dan generasi kode
- Trait stub untuk mesin pencarian, graf, dan deret waktu (driver direncanakan)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
