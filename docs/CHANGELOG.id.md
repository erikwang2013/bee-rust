# Log Perubahan

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
