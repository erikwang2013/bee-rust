// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
// Process-level registry of load paths, keyed by config type.
// One path per type: the last load() wins. reload()/watch() use it
// because they receive no path argument.
use std::any::TypeId;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};

use crate::ConfigError;

static PATHS: OnceLock<Mutex<HashMap<TypeId, PathBuf>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<TypeId, PathBuf>> {
    PATHS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Records the path `T` was loaded from.
pub fn record<T: 'static>(path: &Path) {
    registry()
        .lock()
        .unwrap()
        .insert(TypeId::of::<T>(), path.to_path_buf());
}

/// Returns the path `T` was last loaded from, or `ConfigError::NotFound`.
pub fn path_of<T: 'static>() -> Result<PathBuf, ConfigError> {
    registry()
        .lock()
        .unwrap()
        .get(&TypeId::of::<T>())
        .cloned()
        .ok_or_else(|| ConfigError::NotFound("no load path recorded for this config type".into()))
}

/// Blocks until `path` changes once, then returns. Callers should call
/// `reload()` afterwards. Changes that happen before the watcher is
/// registered are not observed.
pub fn watch_path(path: &Path) -> Result<(), ConfigError> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())
        .map_err(|e| ConfigError::Io(std::io::Error::other(e.to_string())))?;
    watcher
        .watch(path, RecursiveMode::NonRecursive)
        .map_err(|e| ConfigError::Io(std::io::Error::other(e.to_string())))?;
    match rx.recv() {
        Ok(_) => Ok(()),
        Err(_) => Err(ConfigError::Io(std::io::Error::other(
            "watch channel closed",
        ))),
    }
}
