//! Black-box end-to-end tests: each test starts the app on an ephemeral port
//! against a fresh sqlite file in the system temp dir.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

static SEQ: AtomicU64 = AtomicU64::new(0);

struct TestApp {
    base: String,
    db: PathBuf,
    client: reqwest::Client,
}

impl TestApp {
    async fn spawn() -> Self {
        let db = std::env::temp_dir().join(format!(
            "shortlink_e2e_{}_{}.db",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        for suffix in ["", "-journal", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", db.display()));
        }

        let app = shortlink::build_app(db.to_str().unwrap()).await.expect("build_app");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Self {
            base,
            db,
            // Redirects must not be followed: the 302 itself is under test.
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
        }
    }

    async fn create(&self, body: Value) -> reqwest::Response {
        self.client.post(format!("{}/api/links", self.base)).json(&body).send().await.unwrap()
    }

    async fn list(&self) -> Vec<Value> {
        let r = self.client.get(format!("{}/api/links", self.base)).send().await.unwrap();
        assert_eq!(r.status(), 200);
        r.json().await.unwrap()
    }
}

// A1: empty database, empty list. A2: create returns id/code/url/created_at.
#[tokio::test]
async fn a1_a2_create_and_empty_list() {
    let app = TestApp::spawn().await;

    assert!(app.list().await.is_empty());

    let r = app.create(json!({"url": "https://example.com/page"})).await;
    assert_eq!(r.status(), 201);
    let v: Value = r.json().await.unwrap();

    assert!(v["id"].as_i64().unwrap() > 0);
    let code = v["code"].as_str().unwrap();
    assert_eq!(code.len(), 8, "auto code is 8 chars: {code}");
    assert!(
        code.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()),
        "auto code is [a-z0-9]: {code}"
    );
    assert_eq!(v["url"], "https://example.com/page");

    let created = v["created_at"].as_i64().expect("created_at is an integer");
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    assert!((now - created).abs() < 60, "created_at looks like unix seconds: {created} vs {now}");
}

// A3: url and code validation.
#[tokio::test]
async fn a3_validation() {
    let app = TestApp::spawn().await;

    for bad in ["javascript:alert(1)", "//evil.com", "notaurl", "ftp://example.com"] {
        let r = app.create(json!({"url": bad})).await;
        assert_eq!(r.status(), 400, "url {bad:?} must be rejected");
    }

    let long_url = format!("https://example.com/{}", "a".repeat(2048));
    assert!(long_url.len() > 2048);
    assert_eq!(app.create(json!({"url": long_url})).await.status(), 400);

    assert_eq!(
        app.create(json!({"url": "https://example.com", "code": "AB!"})).await.status(),
        400
    );
    assert_eq!(
        app.create(json!({"url": "https://example.com", "code": "abc"})).await.status(),
        400,
        "code shorter than 4 chars"
    );

    // scheme check is case-insensitive
    assert_eq!(app.create(json!({"url": "HTTPS://Example.com/X"})).await.status(), 201);
}

// A4: duplicate code, including after a soft delete.
#[tokio::test]
async fn a4_duplicate_code() {
    let app = TestApp::spawn().await;

    let r = app.create(json!({"url": "https://example.com/1", "code": "dup1"})).await;
    assert_eq!(r.status(), 201);

    assert_eq!(
        app.create(json!({"url": "https://example.com/2", "code": "dup1"})).await.status(),
        409
    );

    let r = app.client.delete(format!("{}/api/links/dup1", app.base)).send().await.unwrap();
    assert_eq!(r.status(), 204);

    // the soft-deleted row still holds the code
    assert_eq!(
        app.create(json!({"url": "https://example.com/3", "code": "dup1"})).await.status(),
        409
    );
}

// A5: redirect including the cached path, unknown code. A6: clicks counted per hit.
#[tokio::test]
async fn a5_a6_redirect_and_clicks() {
    let app = TestApp::spawn().await;
    let target = "https://example.com/target";
    assert_eq!(app.create(json!({"url": target, "code": "abc123"})).await.status(), 201);

    for _ in 0..2 {
        let r = app.client.get(format!("{}/abc123", app.base)).send().await.unwrap();
        assert_eq!(r.status(), 302);
        assert_eq!(r.headers().get("location").unwrap(), target);
    }

    assert_eq!(app.client.get(format!("{}/nope404", app.base)).send().await.unwrap().status(), 404);

    let rows = app.list().await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["code"], "abc123");
    assert_eq!(rows[0]["clicks"], 2, "one click per 302, cached or not");
}

// A7: tagging, find-or-create, idempotency, validation. A8: list shape.
#[tokio::test]
async fn a7_a8_tags_and_list_shape() {
    let app = TestApp::spawn().await;
    assert_eq!(
        app.create(json!({"url": "https://example.com/tagged", "code": "tag1"})).await.status(),
        201
    );

    let tag_url = format!("{}/api/links/tag1/tags", app.base);
    let r = app.client.post(&tag_url).json(&json!({"name": "rust"})).send().await.unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.json::<Value>().await.unwrap(), json!({"tags": ["rust"]}));

    // idempotent: same tag again
    let r = app.client.post(&tag_url).json(&json!({"name": "rust"})).send().await.unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.json::<Value>().await.unwrap(), json!({"tags": ["rust"]}));

    let r = app.client.post(&tag_url).json(&json!({"name": "web"})).send().await.unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.json::<Value>().await.unwrap(), json!({"tags": ["rust", "web"]}));

    // name is trimmed, then must be 1..=64 chars
    let r = app.client.post(&tag_url).json(&json!({"name": "   "})).send().await.unwrap();
    assert_eq!(r.status(), 400);
    let r = app.client.post(&tag_url).json(&json!({"name": "x".repeat(65)})).send().await.unwrap();
    assert_eq!(r.status(), 400);

    // unknown link
    let r = app
        .client
        .post(format!("{}/api/links/ghost1/tags", app.base))
        .json(&json!({"name": "rust"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 404);

    let rows = app.list().await;
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert!(row["id"].as_i64().unwrap() > 0);
    assert_eq!(row["code"], "tag1");
    assert_eq!(row["url"], "https://example.com/tagged");
    assert!(row["created_at"].as_i64().is_some());
    assert_eq!(row["clicks"], 0);
    assert_eq!(row["tags"], json!(["rust", "web"]));
}

// A9: soft delete — 204, cache invalidated immediately, row still in the database.
#[tokio::test]
async fn a9_soft_delete() {
    let app = TestApp::spawn().await;
    assert_eq!(
        app.create(json!({"url": "https://example.com/gone", "code": "del1"})).await.status(),
        201
    );

    // populate the cache first, so the 404 below proves the key was dropped
    assert_eq!(app.client.get(format!("{}/del1", app.base)).send().await.unwrap().status(), 302);

    let r = app.client.delete(format!("{}/api/links/del1", app.base)).send().await.unwrap();
    assert_eq!(r.status(), 204);
    assert_eq!(app.client.get(format!("{}/del1", app.base)).send().await.unwrap().status(), 404);
    assert!(app.list().await.is_empty());

    // the row survives as a soft-deleted row (checked with the ORM on the same file)
    let pool =
        shortlink::bee_rust::bee_orm::pool::sqlite::Pool::connect(app.db.to_str().unwrap(), 1)
            .unwrap();
    let count = shortlink::Link::query().with_deleted().count(&pool).await.unwrap();
    assert_eq!(count, 1, "soft-deleted row must still exist");
}
