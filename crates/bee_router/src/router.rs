// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use axum::Router as AxumRouter;
use axum::routing::MethodRouter;

pub struct Router {
    routes: Vec<(String, MethodRouter)>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
        }
    }

    pub fn ns<F>(mut self, prefix: &str, f: F) -> Self
    where
        F: FnOnce(RouteGroup) -> RouteGroup,
    {
        let group = RouteGroup::new(prefix.to_string());
        let group = f(group);
        self.routes.extend(group.routes);
        self
    }

    pub fn build(self) -> AxumRouter {
        let mut router = AxumRouter::new();
        for (path, method_router) in self.routes {
            router = router.route(&path, method_router);
        }
        router
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RouteGroup {
    prefix: String,
    routes: Vec<(String, MethodRouter)>,
}

impl RouteGroup {
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            routes: Vec::new(),
        }
    }

    pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, ()>,
        T: 'static,
    {
        let full = format!("{}{}", self.prefix, path);
        self.routes.push((full, axum::routing::get(handler)));
        self
    }

    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, ()>,
        T: 'static,
    {
        let full = format!("{}{}", self.prefix, path);
        self.routes.push((full, axum::routing::post(handler)));
        self
    }

    pub fn put<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, ()>,
        T: 'static,
    {
        let full = format!("{}{}", self.prefix, path);
        self.routes.push((full, axum::routing::put(handler)));
        self
    }

    pub fn delete<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, ()>,
        T: 'static,
    {
        let full = format!("{}{}", self.prefix, path);
        self.routes.push((full, axum::routing::delete(handler)));
        self
    }
}
