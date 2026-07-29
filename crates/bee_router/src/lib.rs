pub mod context;
pub mod filter;
pub mod router;

use async_trait::async_trait;
use context::{Context, RouterError};

#[async_trait]
pub trait Controller: Send + Sync + 'static {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError>;
    async fn prepare(&self, _ctx: &mut Context) -> Result<(), RouterError> {
        Ok(())
    }
    async fn finish(&self, _ctx: &mut Context) -> Result<(), RouterError> {
        Ok(())
    }
}

pub use filter::Filter;
pub use router::Router;
