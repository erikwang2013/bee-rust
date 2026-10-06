pub mod handlers;
pub mod models;

pub use bee_rust;
pub use models::{Click, Link, Tag};

use std::sync::Arc;

use bee_rust::bee_cache::{Cache, MemoryCache};
use bee_rust::bee_orm::{migrate, pool::sqlite::Pool};
use bee_rust::bee_router::Router;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<Pool>,
    pub cache: Arc<dyn Cache>,
}

/// Create the schema, wire state and return a ready-to-serve axum router.
pub async fn build_app(
    db_path: &str,
) -> Result<axum::Router, Box<dyn std::error::Error + Send + Sync>> {
    let pool = Arc::new(Pool::connect(db_path, 8)?);
    // m2m target first: the join table of links references tags.
    migrate::sync::<Tag, _>(pool.as_ref()).await?;
    migrate::sync::<Link, _>(pool.as_ref()).await?;
    migrate::sync::<Click, _>(pool.as_ref()).await?;

    let state = AppState { pool, cache: Arc::new(MemoryCache::new()) };

    Ok(Router::new()
        .ns("/api/links", |g| {
            g.post("", handlers::create_link)
                .get("", handlers::list_links)
                .delete("/{code}", handlers::delete_link)
                .post("/{code}/tags", handlers::add_tag)
        })
        .ns("", |g| g.get("/{code}", handlers::redirect_link))
        .with_state(state))
}
