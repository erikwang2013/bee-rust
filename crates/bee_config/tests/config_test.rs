// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use bee_config::{Config, ConfigSource};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Config)]
struct AppConfig {
    app_name: String,
    http_port: u16,
    run_mode: String,
}

#[test]
fn test_load_ini_config() {
    let cfg = AppConfig::load("tests/fixtures/test.conf").unwrap();
    assert_eq!(cfg.app_name, "test-app");
    assert_eq!(cfg.http_port, 8080);
    assert_eq!(cfg.run_mode, "dev");
}

#[test]
fn test_reload_and_watch() {
    let mut cfg = AppConfig::load("tests/fixtures/test.conf").unwrap();
    assert!(cfg.reload().is_ok());
    assert!(cfg.watch().is_ok());
}
