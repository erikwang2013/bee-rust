use bee_config::ConfigSource;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
struct TestConfig {
    app_name: String,
    http_port: u16,
    run_mode: String,
}

impl ConfigSource for TestConfig {
    fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, bee_config::ConfigError> {
        todo!()
    }
    fn reload(&mut self) -> Result<(), bee_config::ConfigError> {
        todo!()
    }
    fn watch(&self) -> Result<(), bee_config::ConfigError> {
        todo!()
    }
}

#[test]
fn test_trait_exists() {
    // Confirms the trait compiles and can be implemented
}
