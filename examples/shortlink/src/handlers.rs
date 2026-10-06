use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use bee_rust::bee_cache::CacheError;
use bee_rust::bee_orm::{Model, m2m};
use serde_json::json;

use crate::{AppState, Click, Link, Tag};

const CACHE_TTL_SECS: u64 = 60;
const ALPHABET: &[u8; 36] = b"abcdefghijklmnopqrstuvwxyz0123456789";

// ---------- helpers ----------

/// Small error type: an axum Response is 128+ bytes, too big for every Result's Err slot.
pub struct HttpError(StatusCode, String);

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error": self.1}))).into_response()
    }
}

fn not_found() -> HttpError {
    HttpError(StatusCode::NOT_FOUND, "not found".into())
}

fn bad_request(msg: impl Into<String>) -> HttpError {
    HttpError(StatusCode::BAD_REQUEST, msg.into())
}

fn oops(e: impl std::fmt::Display) -> HttpError {
    HttpError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn cache_key(code: &str) -> String {
    format!("shortlink:code:{code}")
}

fn is_http_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

fn is_valid_code(code: &str) -> bool {
    (4..=32).contains(&code.len())
        && code.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

static SEQ: AtomicU64 = AtomicU64::new(0);

fn random_code() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64;
    let mut x = nanos ^ SEQ.fetch_add(1, Ordering::Relaxed).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (0..8)
        .map(|_| {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ALPHABET[(x >> 33) as usize % ALPHABET.len()] as char
        })
        .collect()
}

async fn generate_unique_code(state: &AppState) -> Result<String, HttpError> {
    for _ in 0..8 {
        let code = random_code();
        let taken = Link::query()
            .with_deleted()
            .filter_eq("code", code.clone())
            .map_err(oops)?
            .exists(state.pool.as_ref())
            .await
            .map_err(oops)?;
        if !taken {
            return Ok(code);
        }
    }
    Err(oops("could not allocate a unique code"))
}

// ---------- handlers ----------

/// POST /api/links — body {url, code?}
pub async fn create_link(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, HttpError> {
    let Some(url) = body.get("url").and_then(|v| v.as_str()) else {
        return Err(bad_request("url is required"));
    };
    if url.len() > 2048 || !is_http_url(url) {
        return Err(bad_request("url must be an absolute http(s) URL of at most 2048 chars"));
    }

    let code = match body.get("code").and_then(|v| v.as_str()) {
        Some(c) => {
            if !is_valid_code(c) {
                return Err(bad_request("code must match ^[a-z0-9]{4,32}$"));
            }
            c.to_string()
        }
        None => generate_unique_code(&state).await?,
    };

    // A soft-deleted row still holds its code: duplicates are checked with_deleted.
    let taken = Link::query()
        .with_deleted()
        .filter_eq("code", code.clone())
        .map_err(oops)?
        .exists(state.pool.as_ref())
        .await
        .map_err(oops)?;
    if taken {
        return Err(HttpError(StatusCode::CONFLICT, "code already exists".into()));
    }

    let link =
        Link { id: 0, code: code.clone(), url: url.to_string(), created_at: 0, deleted: false };
    link.insert(state.pool.as_ref()).await.map_err(oops)?;

    // auto pk: the database assigns the id, so read the row back.
    let link = Link::query()
        .with_deleted()
        .filter_eq("code", code)
        .map_err(oops)?
        .one(state.pool.as_ref())
        .await
        .map_err(oops)?
        .ok_or_else(|| oops("inserted row not found"))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({"id": link.id, "code": link.code, "url": link.url, "created_at": link.created_at})),
    )
        .into_response())
}

/// GET /api/links — live rows, id ascending.
pub async fn list_links(State(state): State<AppState>) -> Result<Response, HttpError> {
    let links = Link::query().order_by("id").all(state.pool.as_ref()).await.map_err(oops)?;

    let mut out = Vec::with_capacity(links.len());
    for link in links {
        // ponytail: N+1 queries (clicks + tags per row) — fine at example scale.
        let clicks = Click::query()
            .filter_eq("link_id", link.id)
            .map_err(oops)?
            .count(state.pool.as_ref())
            .await
            .map_err(oops)?;
        let mut tags =
            m2m::related::<Link, Tag, _>(state.pool.as_ref(), &link).await.map_err(oops)?;
        tags.sort_by_key(|t| t.id);
        out.push(json!({
            "id": link.id,
            "code": link.code,
            "url": link.url,
            "created_at": link.created_at,
            "clicks": clicks,
            "tags": tags.iter().map(|t| t.name.clone()).collect::<Vec<_>>(),
        }));
    }
    Ok(Json(out).into_response())
}

/// GET /:code — 302 to the target, recording one click per hit.
pub async fn redirect_link(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Response, HttpError> {
    let key = cache_key(&code);
    let (link_id, url) = match state.cache.get(&key).await.map_err(oops)? {
        Some(bytes) => {
            let cached: serde_json::Value = serde_json::from_slice(&bytes).map_err(oops)?;
            let id =
                cached.get("id").and_then(|v| v.as_i64()).ok_or_else(|| oops("bad cache entry"))?;
            let url = cached
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| oops("bad cache entry"))?;
            (id, url.to_string())
        }
        None => {
            let Some(link) = Link::query()
                .filter_eq("code", code.clone())
                .map_err(oops)?
                .one(state.pool.as_ref())
                .await
                .map_err(oops)?
            else {
                return Err(not_found());
            };
            // the cached value carries the link id so a cache hit can still record a click.
            let value =
                serde_json::to_vec(&json!({"id": link.id, "url": link.url})).map_err(oops)?;
            state.cache.set(&key, value, Some(CACHE_TTL_SECS)).await.map_err(oops)?;
            (link.id, link.url)
        }
    };

    Click { id: 0, link_id, created_at: 0 }.insert(state.pool.as_ref()).await.map_err(oops)?;

    Ok((StatusCode::FOUND, [(header::LOCATION, url)]).into_response())
}

/// DELETE /api/links/{code} — soft delete, drop the cache entry.
pub async fn delete_link(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Response, HttpError> {
    let Some(link) = Link::query()
        .filter_eq("code", code.clone())
        .map_err(oops)?
        .one(state.pool.as_ref())
        .await
        .map_err(oops)?
    else {
        return Err(not_found());
    };

    link.delete(state.pool.as_ref()).await.map_err(oops)?;

    if let Err(e) = state.cache.delete(&cache_key(&code)).await
        && !matches!(e, CacheError::NotFound)
    {
        return Err(oops(e));
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// POST /api/links/{code}/tags — body {name}, idempotent attach.
pub async fn add_tag(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, HttpError> {
    let name = body.get("name").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
    if name.is_empty() || name.chars().count() > 64 {
        return Err(bad_request("name must be 1..=64 characters"));
    }

    let Some(link) = Link::query()
        .filter_eq("code", code.clone())
        .map_err(oops)?
        .one(state.pool.as_ref())
        .await
        .map_err(oops)?
    else {
        return Err(not_found());
    };

    // ponytail: find-or-create races can duplicate a tag name; fine at example scale.
    let tag = match Tag::query()
        .filter_eq("name", name.to_string())
        .map_err(oops)?
        .one(state.pool.as_ref())
        .await
        .map_err(oops)?
    {
        Some(tag) => tag,
        None => {
            Tag { id: 0, name: name.to_string() }
                .insert(state.pool.as_ref())
                .await
                .map_err(oops)?;
            Tag::query()
                .filter_eq("name", name.to_string())
                .map_err(oops)?
                .one(state.pool.as_ref())
                .await
                .map_err(oops)?
                .ok_or_else(|| oops("inserted tag not found"))?
        }
    };

    let attached = m2m::related::<Link, Tag, _>(state.pool.as_ref(), &link).await.map_err(oops)?;
    if !attached.iter().any(|t| t.id == tag.id) {
        m2m::attach::<Link, Tag, _>(state.pool.as_ref(), &link, &tag).await.map_err(oops)?;
    }

    let mut tags = m2m::related::<Link, Tag, _>(state.pool.as_ref(), &link).await.map_err(oops)?;
    tags.sort_by_key(|t| t.id);
    Ok(Json(json!({"tags": tags.iter().map(|t| t.name.clone()).collect::<Vec<_>>()}))
        .into_response())
}
