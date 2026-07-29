// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use crate::context::{Context, RouterError};

/// Middleware-like filter with `before` and `after` hooks that run around
/// controller execution. Implementors can inspect or modify the [`Context`]
/// and abort the request early.
pub trait Filter: Send + Sync {
    fn before(&self, _ctx: &mut Context) -> Result<(), RouterError> {
        Ok(())
    }
    fn after(&self, _ctx: &mut Context) -> Result<(), RouterError> {
        Ok(())
    }
}

/// Session filter placeholder.
///
/// Intended to extract a session ID from the request (cookie / header),
/// load the corresponding [`Session`] from cache, and replace the
/// ephemeral session in [`Context`].  Currently a no-op — new sessions
/// are created per-request by [`Context::new`].
///
/// [`Session`]: bee_session::Session
pub struct SessionFilter;

impl Filter for SessionFilter {}
