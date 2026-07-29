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

/// Session filter — loads session from request before controller
pub struct SessionFilter;

impl Filter for SessionFilter {}
