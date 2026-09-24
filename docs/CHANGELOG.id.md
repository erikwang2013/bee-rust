# Log Perubahan

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
