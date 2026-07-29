// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
#[cfg(feature = "config")]   pub use bee_config;
#[cfg(feature = "logs")]     pub use bee_logs;
#[cfg(feature = "cache")]    pub use bee_cache;
#[cfg(feature = "template")] pub use bee_template;
#[cfg(feature = "kv")]       pub use bee_kv;
#[cfg(feature = "search")]   pub use bee_search;
#[cfg(feature = "graph")]    pub use bee_graph;
#[cfg(feature = "tsdb")]     pub use bee_tsdb;
#[cfg(feature = "orm")]      pub use bee_orm;
#[cfg(feature = "session")]  pub use bee_session;
#[cfg(feature = "router")]   pub use bee_router;

pub mod prelude {
    #[cfg(feature = "router")]
    pub use bee_router::{Controller, Router, Context, Filter};
    #[cfg(feature = "orm")]
    pub use bee_orm::Model;
    #[cfg(feature = "config")]
    pub use bee_config::Config;
    #[cfg(feature = "cache")]
    pub use bee_cache::Cache;
    #[cfg(feature = "session")]
    pub use bee_session::Session;
    #[cfg(feature = "logs")]
    pub use bee_logs::{Logger, Output};
    #[cfg(feature = "template")]
    pub use bee_template::context;
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// One-click startup: initializes the logger and returns a handle.
///
/// Keep the returned `LogHandle` alive for the program lifetime.
pub fn init() -> Result<bee_logs::LogHandle> {
    Ok(bee_logs::Logger::new().init()?)
}
