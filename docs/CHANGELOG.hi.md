# परिवर्तन-लॉग

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.2.0] — 2026-10-06

### जोड़ा गया
- `bee_orm`: ORM अब शुरू से अंत तक चलता है — टाइप्ड `Value` पैरामीटर (Null / Bool / Int / Float / Text / Bytes), `#[bee(table / column / pk / auto / ignore)]` के साथ `#[derive(Model)]`, मॉडल पर `insert` / `update` / `delete`, और `QuerySet` निष्पादन (`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm`: तीनों बैकएंड के लिए कनेक्शन पूल (`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)`, `get()` → `CheckedConn`, `query` / `execute`, `status()`, और `begin` / `commit` / `rollback` से ट्रांज़ैक्शन; ट्रांज़ैक्शन के बीच छोड़ा गया कनेक्शन sqlite और postgres पर रोलबैक हो जाता है
- `bee_orm`: नया `postgres-tls` फ़ीचर (TLS पर PostgreSQL, अंतर्निहित Mozilla रूट) और पूल सुदृढ़ीकरण — 30 सेकंड प्रतीक्षा / 10 सेकंड निर्माण टाइमआउट और postgres पूल में प्रीपेयर्ड-स्टेटमेंट कैश
- `bee_orm`: मॉडल जीवनचक्र — `#[bee(auto_now_add)]` / `#[bee(auto_now)]` स्वतः टाइमस्टैम्प, `#[bee(soft_delete)]` सॉफ़्ट डिलीट (delete फ़्लैग पलट देता है; `with_deleted()` / `hard_delete()` इसे बायपास करते हैं), insert/update/delete हुक, `Model::insert_many` (999 पैरामीटर के बैच, गैर-ट्रांज़ैक्शनल), और `QuerySet::filter_in` तथा `sum` / `avg` / `min` / `max` एग्रीगेट
- `bee_orm`: गैर-विनाशकारी माइग्रेशन — `migrate::{create_table, add_missing_columns, sync}` डायलेक्ट के अनुसार DDL (sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`) बनाते हैं: तालिकाएँ बनाते और छूटे कॉलम जोड़ते हैं, कभी हटाते या बदलते नहीं
- `bee_orm`: विदेशी-कुंजी संबंध — `#[bee(fk = Target)]` (`#[bee(sql_type = "…")]` के साथ) FK DDL बनाता है और `rel::{fk_column_to, belongs_to, children, children_for}` उन्हें पढ़ते हैं; प्रवर्तन बैकएंड पर निर्भर है: postgres और sqlite (bundled बिल्ड) लागू करते हैं, mysql inline `REFERENCES` को अनदेखा करता है (round 5+ का अंतराल)
- `bee_orm`: अनेक-से-अनेक संबंध — `#[bee(m2m(Target))]` संबंध घोषित करता है, `m2m::{attach, detach, related, related_for, related_ids}` पढ़ते-लिखते हैं, और join टेबल `create_table` / `sync` बनाते हैं (`add_missing_columns` उसे नहीं छूता)
- `bee_orm`: JSON कॉलम — `serde_json::Value` फ़ील्ड बैकएंड के अनुसार `TEXT` / `JSONB` / `JSON` में मैप होते हैं, ईमानदार NULL अर्थशास्त्र के साथ: SQL `NULL` → `None`, सहेजा गया JSON `null` दस्तावेज़ → `Some(Json::Null)`
- `bee_orm`: MySQL टेबल-स्तरीय फ़ॉरेन की (opt-in) — `MigrateOptions { table_level_fk }` के साथ `sync_with` / `create_table_with` / `add_missing_columns_with`; डिफ़ॉल्ट बंद, और पहले से मौजूद अनाथ पंक्तियाँ चुपचाप छोड़े जाने के बजाय `ADD CONSTRAINT` विफल करती हैं
- `bee_orm`: कस्टम `rustls::ClientConfig` के लिए `Pool::connect_tls_with`, साथ में `bee_orm::rustls` पुनः-निर्यात ताकि संस्करण हमेशा मेल खाए; `connect_tls` बंडल किए webpki रूट रखता है
- `bee_rust`: चार अग्रेषण feature — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` `bee_rust` के माध्यम से `bee_orm` बैकएंड अग्रेषित करते हैं (कोई भी `full` में नहीं)
- `bee_cli`: `bee-rust migrate init` `src/bin/bee_migrate.rs` बनाता है (मौजूद फ़ाइल ओवरराइट नहीं करता), `bee-rust migrate run` उसे `cargo run --bin bee_migrate` से चलाता है
- `bee_kv` / `bee_cache`: Redis बैकएंड टूटे कनेक्शन से उबरते हैं — भीतरी `ConnectionManager` मांग पर (पृष्ठभूमि थ्रेड के बिना) घातांकीय बैकऑफ़ और जिटर के साथ पुनः जुड़ता है (टूटन पर पड़ने वाला कमांड विफल, अगला नए कनेक्शन की प्रतीक्षा करता है)
- `bee_kv` / `bee_cache`: memcached बैकएंड (bee_kv में `MemcacheStore`, bee_cache में `MemcacheCache`); `incr` डेल्टा से पहले काउंटर बनाता है, अहस्ताक्षरित काउंटर 0 पर रुकते हैं, और TTL 0 कुंजी हटा देता है

### बदला गया
- दस्तावेज़ों में bee_orm के माइग्रेशन, संबंध (many-to-many सहित), `bee-rust migrate` सबकमांड, Redis / Memcached बैकएंड और `bee_rust` की ORM अग्रेषण feature अब योजनाबद्ध नहीं हैं
- दस्तावेज़ों में search / graph / tsdb ड्राइवर अब योजनाबद्ध या trait-stub के बजाय कार्यान्वित (opt-in feature) के रूप में दर्ज हैं, और पुराने उदाहरणों के टाइप नाम व कनेक्शन तरीका सुधारे गए (न मौजूद `ElasticsearchEngine` / `Neo4jDB`, गलत `bolt://`)

## [1.1.5] — 2026-09-25

### जोड़ा गया
- 11 और भाषाओं के लिए संक्षिप्त crates.io README: `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`। crates.io पृष्ठ का भाषा स्विचर अब रिपॉज़िटरी के पूर्ण README के बजाय इन फ़ाइलों की ओर इंगित करता है, इसलिए हर भाषा को वही संक्षिप्त डेवलपर-केंद्रित दस्तावेज़ मिलता है।

## [1.1.4] — 2026-09-25

### बदला गया
- crates.io पृष्ठ (`docs/crates-readme.md`) अब द्विभाषी है: ऊपर अंग्रेज़ी, नीचे चीनी। crates.io प्रति क्रेट केवल एक README रेंडर करता है और पृष्ठ के भीतर भाषा चयन की सुविधा नहीं देता, इसलिए अंग्रेज़ी पहली स्क्रीन पर आ गई है और चीनी पाठ उसी पृष्ठ पर बना हुआ है।

## [1.1.3] — 2026-09-25

### जोड़ा गया
- crates.io पृष्ठ (`docs/crates-readme.md`) में अब 13 भाषाओं का स्विचर जोड़ा गया है, जो रिपॉज़िटरी के README से जोड़ता है

### ठीक किया गया
- `docs/api.*`: 11 अनुवादों में चीनी मूल (`api.md`) का लिंक अनुपस्थित था
- `docs/CHANGELOG.zh.md` और `docs/CONTRIBUTING.zh.md` अपने-अपने समूह में एकमात्र ऐसी फ़ाइलें थीं जिनमें स्व-लिंक नहीं था

## [1.1.2] — 2026-09-24

### जोड़ा गया
- `docs/crates-readme.md`: crates.io पृष्ठों के लिए समर्पित README — इंस्टॉल, चलाने योग्य उदाहरण (`examples/hello` से लिया गया, जिसके E2E टेस्ट इसे कवर करते हैं), feature फ़्लैग तालिका और सब-क्रेट सूची
- `scripts/publish.sh`: वर्कस्पेस के क्रेट एक-एक करके प्रकाशित करता है, पहले से अपलोड किए गए को छोड़ देता है, और रेट लिमिट होने पर crates.io द्वारा लौटाए गए पुनःप्रयास समय तक प्रतीक्षा करता है

### ठीक किया गया
- टेस्ट सूट कंपाइल नहीं होता था (main पर `cargo test --workspace` विफल): `bee_router` के इंटीग्रेशन टेस्ट में `Submit` पर `Serialize` अनुपस्थित था (यह `Json` प्रतिक्रिया के रूप में लौटाया जाता है), दो हेल्पर पर `Router::build()` अनुपस्थित था, और आठ `status_line` कॉल में `&` अनुपस्थित था
- `hello` उदाहरण के टेस्ट गलत `CARGO_BIN_EXE` नाम का संदर्भ दे रहे थे — बाइनरी टारगेट अपना हाइफ़न रखता है (`CARGO_BIN_EXE_hello-bee`)
- `hello` का टेम्पलेट ऑटोएस्केप टेस्ट `&#39;` जाँच रहा था जबकि tera `&#x27;` उत्पन्न करता है; एस्केपिंग व्यवहार स्वयं सही था
- `bee_cli` में Clippy `manual_split_once` और `manual_range_contains` लिंट; `bee_router` के इंटीग्रेशन टेस्ट में अप्रयुक्त import

### बदला गया
- अब सभी क्रेट में `readme` और `repository` मेटाडेटा है — पहले किसी भी crates.io पृष्ठ पर README रेंडर नहीं होता था और छह क्रेट में `repository` पूरी तरह अनुपस्थित था
- `examples/hello` को `publish = false` चिह्नित किया गया: यह E2E टेस्ट हार्नेस के रूप में वर्कस्पेस में रहता है, पर crates.io पर प्रकाशित नहीं होता

## [1.0.6] — 2026-08-07

### जोड़ा गया
- `bee_cli` के वास्तविक कार्यान्वयन: `new` (प्रोजेक्ट स्कैफोल्डिंग), `generate controller/model`, `--watch` हॉट रीलोड के साथ `run`, `pack` (release बिल्ड + `dist/` में कॉपी)
- स्कैफोल्डिंग और कोड जनरेशन के लिए CLI यूनिट परीक्षण (7 नए परीक्षण)

### ठीक किया गया
- `bee_rust::init()` अब `logs` feature के पीछे गेट किया गया — घटे हुए feature बिल्ड (जैसे `--no-default-features --features kv`) फिर से संकलित होते हैं
- `bee_kv::InMemoryKvStore::exists` में Clippy `unnecessary_map_or` लिंट
- `rustfmt.toml` से केवल-nightly विकल्प हटाए गए जो stable पर चुपचाप अनदेखा किए जाते थे; वर्कस्पेस अब `cargo fmt --all --check` पास करता है
- `bee_cli` बाइनरी पर `doc = false` ताकि `bee_rust` के साथ rustdoc आउटपुट फ़ाइलनाम टकराव दूर हो
- `hello` उदाहरण का पोर्ट अब `PORT` env var से कॉन्फ़िगर किया जा सकता है

### बदला गया
- `bee-rust migrate` "not implemented" रिपोर्ट करता है और गैर-शून्य कोड के साथ बाहर निकलता है (योजनाबद्ध)
- README / README.en को वास्तविक CLI व्यवहार बताने के लिए अद्यतन किया गया

## [1.0.4] — 2026-07-29

### जोड़ा गया
- `security-rust` के माध्यम से सुरक्षा हमला-पता लगाने वाला फ़िल्टर (27 डिटेक्टर)
- XSS, SQL इंजेक्शन, कमांड इंजेक्शन, पाथ ट्रैवर्सल कवरेज के साथ `SecurityFilter`
- `bee_rust` और `bee_router` में `security` feature flag

### बदला गया
- सुरक्षा feature दस्तावेज़ीकरण के साथ README अद्यतन
- भुगतान समर्थन अनुभाग (WeChat Pay / Alipay) के साथ README अद्यतन

### ठीक किया गया
- Rust 2024 edition के लिए `bee_template` Tera रॉ आइडेंटिफ़ायर सिंटैक्स

## [1.0.3] — 2026-07-29

### जोड़ा गया
- 13 crates के साथ प्रारंभिक वर्कस्पेस संरचना
- `Controller` trait और `Router` के साथ MVC रूटिंग
- `QuerySet` बिल्डर और `Model` डिराइव मैक्रो के साथ ORM
- Redis और Memory बैकएंड के साथ KV/Cache trait अमूर्तता
- Memory/Redis बैकएंड के साथ सत्र प्रबंधन
- INI/YAML/ENV समर्थन और हॉट-रीलोड के साथ कॉन्फ़िगरेशन प्रबंधन
- Tera के माध्यम से टेम्पलेट रेंडरिंग
- tracing एकीकरण के साथ लॉगिंग
- CLI स्कैफोल्डिंग और कोड जनरेशन
- खोज, ग्राफ़, टाइम-सीरीज़ इंजन trait stubs (ड्राइवर योजनाबद्ध)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
