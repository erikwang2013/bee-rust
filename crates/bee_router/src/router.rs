// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use axum::Router as AxumRouter;

pub struct Router {
    // Stub — full axum integration in a follow-up task
    routes: Vec<String>,
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
        for route in group.routes {
            self.routes
                .push(format!("{} {}", route.method, route.path));
        }
        self
    }

    pub fn build(self) -> AxumRouter {
        AxumRouter::new()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RouteGroup {
    prefix: String,
    routes: Vec<RouteDef>,
}

struct RouteDef {
    method: String,
    path: String,
}

impl RouteGroup {
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            routes: Vec::new(),
        }
    }

    pub fn get(mut self, path: &str) -> Self {
        self.routes.push(RouteDef {
            method: "GET".into(),
            path: format!("{}{}", self.prefix, path),
        });
        self
    }

    pub fn post(mut self, path: &str) -> Self {
        self.routes.push(RouteDef {
            method: "POST".into(),
            path: format!("{}{}", self.prefix, path),
        });
        self
    }

    pub fn put(mut self, path: &str) -> Self {
        self.routes.push(RouteDef {
            method: "PUT".into(),
            path: format!("{}{}", self.prefix, path),
        });
        self
    }

    pub fn delete(mut self, path: &str) -> Self {
        self.routes.push(RouteDef {
            method: "DELETE".into(),
            path: format!("{}{}", self.prefix, path),
        });
        self
    }

    #[allow(dead_code)]
    pub fn group<F>(self, _path: &str, _f: F) -> Self
    where
        F: FnOnce(RouteGroup) -> RouteGroup,
    {
        self // stub — nested groups in follow-up
    }
}
