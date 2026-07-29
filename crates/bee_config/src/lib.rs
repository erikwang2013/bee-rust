pub mod error;
pub mod ini;

use std::path::Path;

pub use error::ConfigError;

pub use bee_config_macro::Config;

pub trait ConfigSource: Sized {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError>;
    fn reload(&mut self) -> Result<(), ConfigError>;
    fn watch(&self) -> Result<(), ConfigError>;
}
