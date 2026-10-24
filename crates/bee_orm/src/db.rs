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

    /// The physical table behind a logical name; the identity by default.
    ///
    /// A sharded backend overrides this to pick the table's shard. Statements
    /// that address a model table go through here before formatting; the
    /// returned name is spliced into SQL unquoted, so it must match
    /// `[A-Za-z_][A-Za-z0-9_]*`.
    fn resolve_table(&self, logical: &str) -> Result<String> {
        Ok(logical.to_owned())
    }

    /// Insert and, where the backend can, return the stored row.
    ///
    /// The seam behind [`Model::create`](crate::Model::create), which needs
    /// the row as stored to surface database-assigned values (an
    /// auto-increment primary key, column defaults). The default runs the
    /// statement through [`execute`](Db::execute) and returns `Ok(None)` —
    /// mocks and hand-written impls keep compiling, and `create` falls back
    /// to selecting by primary key. `sql` is a plain `INSERT`; `table` is the
    /// resolved physical table name and `pk_column` is the model's static
    /// primary-key column, for backends whose read-back needs them.
    /// `Ok(None)` also covers a backend that ran the statement but cannot
    /// produce the row.
    async fn insert_returning(
        &self,
        _table: &str,
        _pk_column: &str,
        sql: &str,
        params: &[Value],
    ) -> Result<Option<Row>> {
        self.execute(sql, params).await?;
        Ok(None)
    }
}
