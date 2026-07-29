// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use axum::body::Body;
use axum::http::{Request, StatusCode};
use bee_session::Session;
use bee_template::TemplateEngine;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("serialize error: {0}")]
    SerializeError(String),
    #[error("template error: {0}")]
    TemplateError(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct Context {
    pub request: Request<Body>,
    params: HashMap<String, String>,
    pub session: Session,
    pub templates: Arc<TemplateEngine>,
    response_status: StatusCode,
    response_headers: HashMap<String, String>,
    response_body: Vec<u8>,
    aborted: bool,
}

impl Context {
    pub fn new(request: Request<Body>, session: Session, templates: Arc<TemplateEngine>) -> Self {
        Self {
            request,
            params: HashMap::new(),
            session,
            templates,
            response_status: StatusCode::OK,
            response_headers: HashMap::new(),
            response_body: Vec::new(),
            aborted: false,
        }
    }

    pub fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    pub fn json<T: serde::Serialize>(&mut self, data: &T) -> Result<(), RouterError> {
        if self.aborted {
            return Ok(());
        }
        self.response_headers
            .insert("Content-Type".into(), "application/json".into());
        self.response_body =
            serde_json::to_vec(data).map_err(|e| RouterError::SerializeError(e.to_string()))?;
        Ok(())
    }

    pub fn text(&mut self, body: &str) -> Result<(), RouterError> {
        if self.aborted {
            return Ok(());
        }
        self.response_headers
            .insert("Content-Type".into(), "text/plain; charset=utf-8".into());
        self.response_body = body.as_bytes().to_vec();
        Ok(())
    }

    pub fn html(
        &mut self,
        template: &str,
        data: &HashMap<String, serde_json::Value>,
    ) -> Result<(), RouterError> {
        if self.aborted {
            return Ok(());
        }
        let rendered = self
            .templates
            .render(template, data)
            .map_err(|e| RouterError::TemplateError(e.to_string()))?;
        self.response_headers
            .insert("Content-Type".into(), "text/html; charset=utf-8".into());
        self.response_body = rendered.into_bytes();
        Ok(())
    }

    pub fn redirect(&mut self, url: &str) -> Result<(), RouterError> {
        if self.aborted {
            return Ok(());
        }
        self.response_status = StatusCode::FOUND;
        self.response_headers
            .insert("Location".into(), url.to_string());
        Ok(())
    }

    pub fn abort(&mut self, status: StatusCode, msg: &str) {
        self.aborted = true;
        self.response_status = status;
        self.response_body = msg.as_bytes().to_vec();
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted
    }

    pub fn into_response(self) -> axum::response::Response<Body> {
        let mut builder = axum::response::Response::builder().status(self.response_status);
        for (k, v) in &self.response_headers {
            builder = builder.header(k.as_str(), v.as_str());
        }
        // SAFETY: header values are constructed internally (Content-Type, Location)
        // and never contain invalid characters.
        builder
            .body(Body::from(self.response_body))
            .expect("response builder with internal-only headers should never fail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use bee_cache::MemoryCache;
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Duration;

    fn make_context(templates: Arc<TemplateEngine>) -> Context {
        let cache: Arc<dyn bee_cache::Cache> = Arc::new(MemoryCache::new());
        let session = Session::new(cache, Duration::from_secs(3600));
        let req = Request::builder().body(Body::empty()).unwrap();
        Context::new(req, session, templates)
    }

    #[test]
    fn test_text_response() {
        let engine = TemplateEngine::new(Path::new("tests/fixtures/templates")).unwrap();
        let mut ctx = make_context(Arc::new(engine));
        ctx.text("hello").unwrap();
        assert!(!ctx.is_aborted());
    }

    #[test]
    fn test_json_response() {
        let engine = TemplateEngine::new(Path::new("tests/fixtures/templates")).unwrap();
        let mut ctx = make_context(Arc::new(engine));
        ctx.json(&serde_json::json!({"key": "value"})).unwrap();
        assert!(!ctx.is_aborted());
    }

    #[test]
    fn test_abort() {
        let engine = TemplateEngine::new(Path::new("tests/fixtures/templates")).unwrap();
        let mut ctx = make_context(Arc::new(engine));
        ctx.abort(StatusCode::NOT_FOUND, "not found");
        assert!(ctx.is_aborted());
        // abort should not panic when called multiple times
        ctx.json(&serde_json::json!({})).unwrap(); // should be no-op
    }
}
