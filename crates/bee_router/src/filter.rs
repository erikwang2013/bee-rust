use crate::context::{Context, RouterError};

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
