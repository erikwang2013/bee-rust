// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! The [`Db`] trait: the execution seam every query builder and model method
//! talks to, so callers can accept `&dyn Db` regardless of backend.
//!
//! Implementations are provided by each `pool` backend. Bind values always go
//! through the driver's prepared-statement interface; the SQL string never
//! contains user data.

use async_trait::async_trait;

use crate::{Dialect, Result, Row, Value};

#[async_trait]
pub trait Db: Send + Sync {
    /// Run a query and collect every row.
    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>>;

    /// Run a statement, returning the number of affected rows.
    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64>;

    /// The backend's SQL dialect, which [`migrate`](crate::migrate) needs for
    /// DDL. `None` by default — mock and hand-written impls keep compiling;
    /// the migration functions fail loudly on `None` rather than guessing.
    fn dialect(&self) -> Option<Dialect> {
        None
    }
}
