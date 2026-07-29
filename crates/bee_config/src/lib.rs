pub mod error;
pub use error::ConfigError;
use std::path::Path;

pub trait ConfigSource: Sized {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError>;
    fn reload(&mut self) -> Result<(), ConfigError>;
    fn watch(&self) -> Result<(), ConfigError>;
}
