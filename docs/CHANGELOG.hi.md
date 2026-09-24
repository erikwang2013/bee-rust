# परिवर्तन-लॉग

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

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
