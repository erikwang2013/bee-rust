# परिवर्तन-लॉग

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

> संस्करणन: जब तक प्रोजेक्ट 1.x है, माइनर रिलीज़ में API-तोड़ने वाले बदलाव हो सकते हैं; यदि आप `OrmError` पर exhaustive match करते हैं तो सटीक संस्करण पिन करें। 1.3.0 और 1.6.0 देखें।

## [1.6.1] — 2026-10-26

### सुरक्षा
- निर्भरता ताज़ा करने से 7 RUSTSEC कमज़ोरियाँ और 6 ऑडिट चेतावनियाँ बंद होती हैं। `rustls` 0.23.46, `rustls-webpki` 0.103.16, `webpki-roots` 1.0.11, `ctutils` 0.4.4, `tokio` 1.54.1, `lru` 0.18.5, और `yoke` / `yoke-derive` 0.8.4 प्रभावित संस्करणों का स्थान लेते हैं; `mysql_async` 0.37.1 और `redis` 1.6 / `deadpool-redis` 0.23.1 अनुरक्षण-रहित `rustls-pemfile`, `proc-macro-error2` और पुरानी `lru` श्रृंखलाओं को हटा देते हैं। `cargo audit` अब 0 कमज़ोरियाँ और 0 चेतावनियाँ रिपोर्ट करता है।

### बदला गया
- `bee_kv` / `bee_cache`: पहला Sentinel URL अब `SentinelNodeConnectionInfo` के माध्यम से मास्टर कनेक्शन के ACL क्रेडेंशियल, डेटाबेस, प्रोटोकॉल और TLS भी प्रदान करता है; ACL-संरक्षित या केवल-TLS मास्टर तब काम करता है जब उसका Sentinel URL वही सेटिंग्स ले जाता है। पूल किए गए कनेक्शन अब भी केवल PING से रीसायकल होते हैं, भूमिका-जाँच नहीं होती।
- MySQL ड्राइवर को `mysql_async` 0.37 पर अपग्रेड करने से देखने योग्य व्यवहार अपरिवर्तित रहता है और इसे MySQL 5.7.44 / 8.0.46 / 8.4.11 / 9.7.4 तथा MariaDB 10.11.19 पर सत्यापित किया गया।

## [1.6.0] — 2026-10-25

### जोड़ा गया
- `bee_orm`: शार्डिंग में सहजता — `#[bee(shard_key)]` राउटिंग फ़ील्ड को चिह्नित करता है (अधिकतम एक; `ignore` किए गए और डेटाबेस-द्वारा निर्धारित `auto` / `auto_now_add` / `auto_now` फ़ील्ड पर अस्वीकृत, क्योंकि राउटिंग insert से पहले होती है), `Model::shard_key()` उसे उजागर करता है (`None` / `NULL` राउट करने योग्य नहीं है), और `ShardedPools::for_model` कुंजी दोहराए बिना किसी इंस्टेंस को राउट करता है। `ShardedPools::all_shards()` कॉन्फ़िगर किए गए पूल्स पर पुनरावृत्ति करता है, और `sync_all_parallel` प्रत्येक पूल × टेबल शार्ड को समवर्ती रूप से माइग्रेट करता है (पहले से लागू DDL रोलबैक नहीं होता)।
- `bee_orm`: `OrmError::ShardError` शार्डिंग के दुरुपयोग को क्वेरी विफलताओं से अलग करता है; `OrmError` अब `#[non_exhaustive]` है — बाहरी match के लिए wildcard arm आवश्यक है।
- `bee_orm`: क्रॉस-बैकएंड `PoolStatus` के साथ sqlite, postgres और mysql पूल्स पर `pool_status()` (mysql पर `size` / `available` `None` हैं, क्योंकि वह कोई आँकड़े उजागर नहीं करता), और धीमी क्वेरी कॉलबैक के लिए `TimedDb`। Criterion बेंचमार्क राउटर हैशिंग और QuerySet SQL रेंडरिंग को कवर करते हैं।
- `bee_kv` / `bee_cache`: `RedisLock::acquire(key, ttl, wait)` सीमित प्रतीक्षा समाप्त होने तक अधिकतम हर 50ms पर लॉक पुनः प्रयास करता है; `RedisPoolStatus` / `pool_status()` crate-स्वामित्व वाला पूल स्नैपशॉट देते हैं; `redis-tls` feature `rediss://` कनेक्शन सक्षम करता है और ACL क्रेडेंशियल URL में जाते हैं।
- `bee_kv` / `bee_cache`: `redis-sentinel` / `redis-cluster` features के पीछे Sentinel स्टोर/कैश (`RedisSentinelStore`, `RedisSentinelCache`) और Cluster स्टोर/कैश (`RedisClusterStore`, `RedisClusterCache`); `CROSSSLOT` से बचने के लिए क्लस्टर `mget` / `mset` एकल-कुंजी कमांड के रूप में चलते हैं।
- `examples/sharding` और `examples/redis-lock`; CI दोनों उदाहरणों और बेंचमार्क को कंपाइल करता है, MySQL 9.7.4 पिन करता है, और MariaDB job बनाए रखता है।
- `bee_router`: GET/HEAD के लिए opt-in विलंबता बजट, पुराने कैश फ़ॉलबैक के साथ — `Router::latency(LatencyConfig::new(cache))` डिफ़ॉल्ट रूप से कुल 6 ms उपयोग करता है (4 ms हैंडलर + 2 ms आरक्षित), 64 KiB तक की सार्वजनिक GET 200 प्रतिक्रियाएँ कैश करता है, और टाइमआउट पर उन्हें फिर से परोसता है (`X-Bee-Cache: stale`, `Age`) या 504 लौटाता है। टाइमआउट हुए GET कैश गर्म करने के लिए एक सीमित पृष्ठभूमि कार्य में जारी रहते हैं, और `LatencyStats` काउंटर तथा अनुमानित P50/P99 उजागर करता है। राइट मेथड 30 s/408 व्यवहार बनाए रखते हैं।

### बदला गया
- `OrmError` अब `#[non_exhaustive]` है और शार्डिंग विफलताएँ अब `QueryError` के बजाय `ShardError` रिपोर्ट करती हैं; डाउनस्ट्रीम match के लिए wildcard arm आवश्यक है।

### सीमाएँ
- Sentinel: पूल किए गए मास्टर कनेक्शन `PING` से रीसायकल होते हैं, भूमिका-जाँच नहीं होती, इसलिए failover के बाद पदावनत मास्टर जो पहुँच-योग्य रहता है, पुराने कनेक्शन परोस सकता है। `deadpool-redis` 0.18 के `Config::from_urls` के ज़रिए मास्टर कनेक्शन को ACL क्रेडेंशियल या TLS भी नहीं मिलते (Sentinel URL के क्रेडेंशियल/TLS केवल Sentinels पर लागू होते हैं); ACL-संरक्षित या केवल-TLS मास्टर को सीधे कनेक्ट करना होगा।
- Cluster: कमांड `RedisOptions::response_timeout` से सीमित नहीं होते — `deadpool-redis` 0.18 कोई setter उजागर नहीं करता; checkout, create और recycle अपनी 30s / 5s / 5s सीमाएँ बनाए रखते हैं।
- विलंबता बजट: केवल GET/HEAD। समय-सीमा प्रतिक्रिया को सीमित करती है, सर्वर-साइड कार्य को नहीं — SQLite की ब्लॉकिंग क्वेरी रद्द नहीं की जा सकतीं और Postgres/MySQL/Redis कमांड चलते रहते हैं। कैश की गई प्रतिक्रियाएँ केवल सार्वजनिक हैं और TTL तक पुरानी हो सकती हैं; राइट कैश को कभी अमान्य नहीं करते; केवल सटीक आकार के ≤64 KiB बॉडी कैश होते हैं; `MemoryCache` असीमित है। long-poll/streaming GET रूट्स को `latency` के बिना Router उपयोग करना चाहिए।

## [1.5.0] — 2026-10-25

### जोड़ा गया
- `bee_orm`: MySQL 5.7 और MariaDB 10.11+ का समर्थन — CI मैट्रिक्स अब 5.7 / 8.0 / 8.4 / 9.x के साथ-साथ एक MariaDB job भी कवर करता है; इंटीग्रेशन टेस्ट सर्वर विक्रेता को पहचानते हैं (MariaDB JSON को LONGTEXT के रूप में रिपोर्ट करता है और कॉलम-स्तरीय `REFERENCES` लागू करता है; MySQL 5.7/8.x इसे पार्स करके अनदेखा करते हैं)।
- `bee_orm`: MySQL पर `Model::create` अब मॉडल के `columns()` मेटाडेटा का उपयोग करके प्रत्येक insert की `information_schema` जाँच छोड़ता है; मेटाडेटा-रहित हस्तलिखित मॉडल पुरानी जाँच रखते हैं। यह संकेत आधिकारिक है — schema और मॉडल का बदलाव समर्थित नहीं है।
- `bee_orm::shard`: `ShardedPools::{pools, pool, len, is_empty}` कॉन्फ़िगर किए गए शार्ड्स को संचालन के लिए उजागर करते हैं; `ShardHandle` ऑटो-इंक्रीमेंट संकेत आगे भेजता है।
- `bee_kv` / `bee_cache`: `status()` पूल मेट्रिक्स; `with_options` के साथ सार्वजनिक `RedisOptions` (`max_size`, `response_timeout`); `RedisLock::try_lock_with_watchdog` release/drop तक TTL का नवीनीकरण करता है (watchdog TTL कम से कम 30ms होना चाहिए)।
- आंतरिक: एक दस्तावेज़-संगति CI गेट (`scripts/check-docs.sh`), एक सोशल-प्रीव्यू रेंडर स्क्रिप्ट, और `tests/mod.rs` से अलग किए गए साझा टेस्ट फ़िक्सचर।

## [1.4.0] — 2026-10-25

### जोड़ा गया
- `bee_orm`: MySQL 8.0 / 8.4 / 9.x संगतता, CI मैट्रिक्स द्वारा सत्यापित। MySQL 9 कॉलम-स्तरीय `REFERENCES` लागू करता है (8.0/8.4 इसे पार्स करके अनदेखा करते हैं); पहले संदर्भित टेबल बनाएं।
- `bee_orm`: `shard::{ShardedPools, HashRouter, ShardRouter}` — डेटाबेस और टेबल के लिए key-required हैश शार्डिंग; क्रॉस-शार्ड क्वेरी/ट्रांजैक्शन नहीं।
- `bee_kv` / `bee_cache`: वास्तविक Redis कनेक्शन पूल (`deadpool-redis`) और `bee_kv::RedisLock` (`SET NX PX` + Lua रिलीज़)।

## [1.3.0] — 2026-10-13

### ठीक किया गया
- `bee_tsdb` / `bee_graph`: अकेले किसी एक backend feature को चालू करने पर कंपाइल नहीं होता था — साझा `http_client` हेल्पर feature-गेटेड `influxdb` / `neo4j` मॉड्यूल के अंदर रहता था, इसलिए केवल `iotdb` / `questdb` / `arangodb` / `nebulagraph` के साथ बिल्ड unresolved import पर गिरता था; अब यह हर crate के अपने `crate::http` में है (cfg ठीक उन्हीं backend पर जो इसे इस्तेमाल करते हैं), और हर एकल-backend feature अकेले कंपाइल होती है — feature मैट्रिक्स एक-एक करके दोबारा चलाया गया
- `bee_router`: वैकल्पिक `security` feature अब **security-rust 4.0.0** पर — `Scanner` / `DetectionResult` / `AttackCategory` / `Severity` का आकार 1.x जैसा ही है, bee_router के सोर्स में कोई बदलाव नहीं चाहिए; क्षमता-सतह बढ़ी है: अंतर्निहित डिटेक्टर 32 (नए: `format_string`, `log4shell`, HPP, `formula_injection`, reDoS), डिटेक्टर अब मज़बूत/कमज़ोर श्रेणियों में (कमज़ोर सिग्नल `Severity::Low` पर तय), और नया `Scanner::assess`। फ़िल्टर अब भी "कोई भी हिट मिले तो रोकें" पर फैसला करता है, इसलिए **कमज़ोर सिग्नल (जैसे दोहराया गया query पैरामीटर) भी अब रोके जाएंगे** — कस्टम scanner डालने का रास्ता अब नहीं है; इन्हें जाने देना हो तो `SecurityFilter::scanner()` के इर्द-गिर्द अपना `Filter` बनाइए। निर्भरता ग्राफ़ में "1.x + 4.x" की जगह अब एक ही security-rust 4.0.0 है; दस्तावेज़ भी अद्यतन
- दोहराव समाहित: `bee_tsdb` / `bee_graph` / `bee_search` में कनेक्शन व error-mapping हेल्पर की एक-एक ही प्रति रही (backend का नाम अब कॉलर देता है, त्रुटि-संदेश अपरिवर्तित; `aggregation_fn` `bee_tsdb` की lib में गया), `bee_graph` के `backtick` / `esc` उसके `http` मॉड्यूल में आ गए, `bee_orm` ने `dedup_pk_values` / `index_rows_by` और `migrate` का foreign-key खंड निकाला, `bee_session::refresh` अब `load` का पुनरुपयोग करता है, और `bee_config` के `watch_path` ने बची हुई `wait_for_change` कॉल-पथ समेट ली; `bee_search` के त्रुटि-संदेशों में backend का नाम वापस आया, और `bee_kv` का टेस्ट stub अब उसके टेस्टों तक ही सीमित है
- सार्वजनिक API हटाए गए (इसीलिए माइनर वर्शन): `Context::set_params` / `param`, `SecurityFilter::with_scanner`, `Traversal::edge_labels` → `edge_label: Option<String>` (तीनों backend पहले से केवल पहला label लेते थे — व्यवहार अपरिवर्तित), `GraphError::EdgeNotFound`, `ConfigError::ParseError` / `TypeMismatch`, `bee_config::watch_path_timeout`, और `bee_logs::Logger::async_` (ऐसा स्विच जिसे `init()` कभी पढ़ता ही नहीं था) — सभी के शून्य कॉलर
- मृत निर्भरताएँ हटीं: `bee_router` (`tracing` / `http` / `bee_config` / `bee_logs`), `bee_session` (`async-trait`), `bee_config_macro` (`proc-macro2`), और `bee_tsdb` / `bee_graph` / `bee_search` में [dependencies] से दोहराए dev `serde`

## [1.2.3] — 2026-10-06

### ठीक किया गया
- `bee_cli`: स्कैफ़ोल्ड अब ऐसे प्रोजेक्ट नहीं बनाता जो कंपाइल न हों — `new` टेम्पलेट में पैकेज का नाम गलत था (`bee-rust`, जो रिज़ॉल्व नहीं होता; प्रकाशित नाम `bee_rust`) और कंट्रोलर टेम्पलेट में `#[async_trait]` नहीं था तथा निजी `RouterError` रूट पथ इस्तेमाल हो रहा था; नया `BEE_CLI_E2E`-गेटेड टेस्ट प्रोजेक्ट बनाकर उस पर `cargo check` चलाकर इसे शुरू से अंत तक पिन करता है
- दस्तावेज़: api का कंट्रोलर उदाहरण अब कंपाइल-योग्य रूप में है (अपनी `async-trait` निर्भरता, `bee_rust::bee_router::context::RouterError`); केवल `bee_rust` पर निर्भर प्रोजेक्ट के लिए `#[bee(crate = "bee_rust::bee_orm")]` लिखावट, `{name}` रूट सिंटैक्स के साथ `ns` खाली-प्रीफ़िक्स नियम, और `create()` का उपयोग दर्ज किया गया; बंगाली api दस्तावेज़ में घुसे एक कोरियाई अंश को ठीक किया गया
- `examples/shortlink`: आंतरिक त्रुटि विवरण अब लॉग में जाते हैं (प्रतिक्रिया में सामान्य संदेश), URL सत्यापन खाली होस्ट और नियंत्रण अक्षर अस्वीकार करता है, और `create_link` / `add_tag` अब `Model::create()` उपयोग करते हैं (302 हॉट पाथ का `Click` इंसर्ट `insert()` ही रहता है)

## [1.2.2] — 2026-10-06

### जोड़ा गया
- `bee_orm`: `Model::create() -> Result<Self>` — इंसर्ट करके पूरा इंस्टेंस लौटाता है, डेटाबेस द्वारा दी गई प्राइमरी कुंजी सहित (sqlite / postgres में `INSERT … RETURNING *`, mysql में `LAST_INSERT_ID()` + दोबारा पढ़ना); `insert()` अपरिवर्तित है और प्रभावित पंक्तियाँ लौटाता रहता है
- `bee_orm`: `#[bee(crate = "…")]` विशेषता — derive के लिए ORM क्रेट का पथ निर्दिष्ट करें (केवल `bee_rust` मेटा-क्रेट पर निर्भर हों तो `#[bee(crate = "bee_rust::bee_orm")]`); न देने पर विस्तार बाइट-दर-बाइट वही रहता है
- feature-नियंत्रित सार्वजनिक मॉड्यूल वाले आठ crates में docs.rs बिल्ड कॉन्फ़िगरेशन पूरा किया (`all-features = true`) — बैकएंड feature के दस्तावेज़ पृष्ठ और derive विशेषता सारणी docs.rs पर पहले नहीं दिखती थीं

## [1.2.1] — 2026-10-06

### जोड़ा गया
- `bee_orm`: opt-in features के पीछे टाइप्ड दिनांक / Decimal फ़ील्ड — `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) और `rust_decimal` (`Decimal`) तीनों बैकएंड पर `Date` / `DateTime` / `DateTimeTz` / `Decimal` SQL प्रकारों में मैप होते हैं; `bee_rust` इन्हें `orm-chrono` / `orm-rust_decimal` के रूप में अग्रेषित करता है (`full` में नहीं)
- `bee_orm`: इन प्रकारों की स्पष्ट सीमाएँ — sqlite TEXT में रखता है (`typeof` = `text`; DECIMAL घोषित करने पर NUMERIC एफ़िनिटी मिलती और `"1.50"` चुपचाप REAL 1.5 बन जाता), mysql `datetime(6)` / `timestamp(6)` का माइक्रोसेकंड रखता है (TIMESTAMP UTC के रूप में पढ़ा जाता है, DATETIME naive रहता है), और pg `numeric` 29 सार्थक अंकों से ऊपर `NULL` पढ़ा जाता है; `NaiveTime` / `DateTime<Local>` / `FixedOffset` / नैनोसेकंड दायरे से बाहर हैं और `#[bee(sql_type = "…")]` चाहिए, और feature बंद होने पर मॉडल कंपाइल नहीं होता (E0277)

### ठीक किया गया
- `bee_orm`: m2m संबंध-अनुपस्थिति का संकेत अब टेबल नाम की जगह टाइप ident (`#[bee(m2m(Author))]`) लिखता है, इसलिए नाम बदली टेबल पर भी सुझाव कंपाइल होता है
- CI: `rust-toolchain` में स्पष्ट `toolchain: stable` इनपुट जोड़ा गया; बिना features में `clippy --all-targets` तोड़ने वाली दो टेस्ट फ़ाइलों को फ़ाइल-स्तरीय cfg गेट मिले (bee_cache / bee_kv) और `bee_orm` का m2m doctest बैकएंड-निरपेक्ष बना; CLI `migrate` का एंड-टू-एंड टेस्ट अब अपना scratch crate ऑनलाइन बनाता है (ऑफ़लाइन केवल गर्म लोकल कैश पर चलता था) और CI में `BEE_CLI_E2E=1` से जुड़ा

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
