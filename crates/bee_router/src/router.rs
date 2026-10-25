// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use std::time::Duration;

use axum::Router as AxumRouter;
use axum::middleware::from_fn_with_state;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::latency::{LatencyConfig, middleware as latency_middleware};

/// A declarative router that collects named routes and builds an
/// [`axum::Router`]. Routes are organised into [`RouteGroup`]s via
/// the [`ns`](Router::ns) method.
///
/// The state type `S` is the router's shared state, passed to handlers
/// through axum extractors such as [`axum::extract::State`]. Stateless
/// usage keeps the default `S = ()`.
pub struct Router<S = ()> {
    routes: Vec<(String, MethodRouter<S>)>,
    latency: Option<LatencyConfig>,
}

impl<S> Router<S> {
    pub fn new() -> Self {
        Self { routes: Vec::new(), latency: None }
    }

    pub fn ns<F>(mut self, prefix: &str, f: F) -> Self
    where
        F: FnOnce(RouteGroup<S>) -> RouteGroup<S>,
    {
        let group = RouteGroup::new(prefix.to_string());
        let group = f(group);
        self.routes.extend(group.routes);
        self
    }

    /// Enable an opt-in latency budget for `GET`/`HEAD` requests.
    ///
    /// The handler gets [`LatencyConfig::deadline`] minus the configured
    /// fallback reserve; on expiry a public cached response is replayed, or
    /// the request returns `504`. Other methods (including all writes) keep
    /// the router's existing 30-second timeout.
    ///
    /// The deadline bounds the response, not background work: SQLite blocking
    /// queries cannot be cancelled, and Postgres/MySQL/Redis commands continue
    /// server-side. Timed-out GET handlers can continue in a bounded
    /// background task to warm the cache.
    pub fn latency(mut self, config: LatencyConfig) -> Self {
        self.latency = Some(config);
        self
    }

    /// Build the router without supplying state. Handlers using
    /// [`axum::extract::State`] require [`Router::with_state`] instead.
    ///
    /// Every route gets a 2 MiB body limit and a 30 s request timeout.
    /// With [`Router::latency`], `GET`/`HEAD` additionally get the configured
    /// deadline and stale-cache fallback.
    pub fn build(self) -> AxumRouter<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        let mut router = AxumRouter::new();
        for (path, method_router) in self.routes {
            router = router.route(&path, method_router);
        }
        let router = router.layer(
            // One stack per `Router::layer` call: the error handling must be
            // inside it so the composed service's error is `Infallible`.
            tower::ServiceBuilder::new()
                .layer(axum::error_handling::HandleErrorLayer::new(
                    |err: axum::BoxError| async move {
                        if err.is::<tower::timeout::error::Elapsed>() {
                            (axum::http::StatusCode::REQUEST_TIMEOUT, "request timed out")
                                .into_response()
                        } else {
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
                        }
                    },
                ))
                .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(30)))
                .layer(axum::extract::DefaultBodyLimit::max(2 * 1024 * 1024)),
        );
        // Added last, so it is the outermost layer: the latency deadline wins
        // over the 30s timeout for GET/HEAD. A timed-out handler moved to the
        // background still runs inside the inner 30s timeout.
        match self.latency {
            Some(config) => router.layer(from_fn_with_state(config, latency_middleware)),
            None => router,
        }
    }

    /// Build the router and attach `state`, yielding a fully constructed
    /// [`AxumRouter<()>`](axum::Router) ready to serve.
    pub fn with_state(self, state: S) -> AxumRouter<()>
    where
        S: Clone + Send + Sync + 'static,
    {
        self.build().with_state(state)
    }
}

impl<S> Default for Router<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// A grouping of routes sharing a common URL prefix, created via
/// [`Router::ns`]. Supports get, post, put, and delete.
pub struct RouteGroup<S = ()> {
    prefix: String,
    routes: Vec<(String, MethodRouter<S>)>,
}

impl<S> RouteGroup<S> {
    pub fn new(prefix: String) -> Self {
        Self { prefix, routes: Vec::new() }
    }

    pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        let full = format!("{}{}", self.prefix, path);
        self.routes.push((full, axum::routing::get(handler)));
        self
    }

    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        let full = format!("{}{}", self.prefix, path);
        self.routes.push((full, axum::routing::post(handler)));
        self
    }

    pub fn put<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        let full = format!("{}{}", self.prefix, path);
        self.routes.push((full, axum::routing::put(handler)));
        self
    }

    pub fn delete<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        let full = format!("{}{}", self.prefix, path);
        self.routes.push((full, axum::routing::delete(handler)));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::{Query, State};
    use axum::http::{Request, StatusCode};
    use serde::Deserialize;
    use tower::ServiceExt;

    #[derive(Deserialize)]
    struct HelloParams {
        name: Option<String>,
    }

    async fn hello(Query(params): Query<HelloParams>) -> String {
        format!("hello {}", params.name.unwrap_or_else(|| "world".into()))
    }

    #[tokio::test]
    async fn stateless_handler_with_query_extractor() {
        let app = Router::new().ns("/api", |ns| ns.get("/hello", hello));
        let res = app
            .build()
            .oneshot(Request::builder().uri("/api/hello?name=rust").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), 1024).await.unwrap();
        assert_eq!(&body[..], b"hello rust");
    }

    #[derive(Clone)]
    struct AppState {
        counter: usize,
    }

    async fn count(State(state): State<AppState>) -> String {
        format!("count={}", state.counter)
    }

    #[tokio::test]
    async fn handler_with_state_extractor() {
        let app = Router::new()
            .ns("/api", |ns| ns.get("/count", count))
            .with_state(AppState { counter: 42 });
        let res = app
            .oneshot(Request::builder().uri("/api/count").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), 1024).await.unwrap();
        assert_eq!(&body[..], b"count=42");
    }
}
