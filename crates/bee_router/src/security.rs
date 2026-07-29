// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use axum::http::StatusCode;
use security_rust::{DetectionResult, Scanner};

use crate::context::Context;
use crate::context::RouterError;
use crate::filter::Filter;

/// A [`Filter`] that scans incoming requests for malicious payloads using
/// the `security-rust` detection engine.
///
/// By default, 27 detectors cover XSS, SQL injection, command injection,
/// SSRF, path traversal, and more.  If any attack is detected, the request
/// is aborted with HTTP 400 and a message listing each matched attack type.
///
/// # Body scanning
///
/// The current implementation scans the URI query string and the `Cookie`,
/// `User-Agent`, and `Referer` headers.  Full request-body scanning requires
/// buffering the body before the controller runs and is planned for a future
/// release.
pub struct SecurityFilter {
    scanner: Scanner,
}

impl SecurityFilter {
    /// Create a filter with all 27 detectors enabled.
    pub fn new() -> Self {
        Self {
            scanner: Scanner::default(),
        }
    }

    /// Create a filter backed by a custom-configured [`Scanner`] (e.g. one
    /// built via [`Scanner::builder()`]).
    pub fn with_scanner(scanner: Scanner) -> Self {
        Self { scanner }
    }

    /// Access the underlying scanner for direct use (e.g. scanning
    /// individual strings in a controller).
    pub fn scanner(&self) -> &Scanner {
        &self.scanner
    }
}

impl Default for SecurityFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter for SecurityFilter {
    fn before(&self, ctx: &mut Context) -> Result<(), RouterError> {
        // ── Scan the URI query string ──────────────────────────────
        if let Some(query) = ctx.request.uri().query() {
            let results = self.scanner.scan(query);
            if !results.is_empty() {
                ctx.abort(
                    StatusCode::BAD_REQUEST,
                    &format_attack_message("query string", &results),
                );
                return Ok(());
            }
        }

        // ── Scan relevant request headers ──────────────────────────
        let headers = ctx.request.headers();
        let header_names = ["cookie", "user-agent", "referer"];

        for name in header_names {
            if let Some(value) = headers.get(name).and_then(|v| v.to_str().ok()) {
                let results = self.scanner.scan(value);
                if !results.is_empty() {
                    ctx.abort(
                        StatusCode::BAD_REQUEST,
                        &format_attack_message(&format!("header `{name}`"), &results),
                    );
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn after(&self, _ctx: &mut Context) -> Result<(), RouterError> {
        Ok(())
    }
}

/// Format a human-readable abort message from a list of detection results.
fn format_attack_message(source: &str, results: &[DetectionResult]) -> String {
    let attacks: Vec<String> = results
        .iter()
        .map(|r| format!("{} ({})", r.attack_type, r.severity))
        .collect();
    format!("attack detected in {}: {}", source, attacks.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_detects_xss() {
        let filter = SecurityFilter::new();
        let results = filter.scanner.scan("<script>alert(1)</script>");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.attack_type == "xss"));
    }

    #[test]
    fn test_scanner_detects_sql_injection() {
        let filter = SecurityFilter::new();
        let results = filter.scanner.scan("' OR '1'='1");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.attack_type == "sql_injection"));
    }

    #[test]
    fn test_scanner_detects_command_injection() {
        let filter = SecurityFilter::new();
        let results = filter.scanner.scan("$(rm -rf /)");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.attack_type == "command_injection"));
    }

    #[test]
    fn test_clean_input_passes() {
        let filter = SecurityFilter::new();
        let results = filter.scanner.scan("hello world");
        assert!(results.is_empty());
    }

    #[test]
    fn test_format_attack_message() {
        let results = vec![DetectionResult {
            attack_type: "xss".into(),
            category: security_rust::AttackCategory::Injection,
            severity: security_rust::Severity::Critical,
            matched_pattern: "<script>".into(),
            offset: 0,
            message: "XSS attack detected".into(),
        }];
        let msg = format_attack_message("query string", &results);
        assert!(msg.contains("xss"));
        assert!(msg.contains("CRITICAL"));
        assert!(msg.contains("query string"));
    }

    #[test]
    fn test_scanner_detects_path_traversal() {
        let filter = SecurityFilter::new();
        let results = filter.scanner.scan("../../../etc/passwd");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.attack_type == "path_traversal"));
    }
}
