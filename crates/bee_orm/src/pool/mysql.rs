// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! MySQL / TiDB pool (mysql_async's built-in pool).
//!
//! # Transactions
//!
//! Transactions must run on a connection held from `Pool::get()`: statements
//! sent through the `Pool` itself may each land on a different connection.
//! `CheckedConn::begin` / `commit` / `rollback` issue `BEGIN` / `COMMIT` /
//! `ROLLBACK` over the text protocol (`query_drop`) — MySQL 8 rejects those
//! statements in the prepared-statement protocol (error 1295). Dropping a
//! `CheckedConn` returns the connection to the pool;
//! the pool resets it on the next check-out (`reset_connection`, on by
//! default), which rolls back a leftover transaction. Always end a
//! transaction with `commit()` or `rollback()` — until then it holds locks.
//!
//! `Pool` has no `status()`: mysql_async exposes no pool statistics.
//!
//! ```no_run
//! # async fn demo() -> Result<(), bee_orm::OrmError> {
//! use bee_orm::pool::mysql::Pool;
//!
//! let pool = Pool::connect("mysql://user:pass@localhost:3306/app", 16)?;
//! let rows = pool
//!     .query("SELECT name FROM users WHERE age > ?", &[bee_orm::Value::from(18)])
//!     .await?;
//! # Ok(())
//! # }
//! ```

use std::time::Duration;

use async_trait::async_trait;
use mysql_async::prelude::Queryable;
use mysql_async::{Opts, OptsBuilder, Pool as MysqlPool, PoolConstraints, PoolOpts};
use serde_json::Value as Json;

use super::{Row, conn_err, query_err};
use crate::{Db, Dialect, OrmError, Result, Value};

/// ponytail: fixed 30s — mysql_async 0.34.2 has no acquire timeout; make it
/// configurable when a caller needs to tune it.
const GET_CONN_TIMEOUT: Duration = Duration::from_secs(30);

/// A cloneable handle to a mysql_async pool.
#[derive(Clone)]
pub struct Pool {
    inner: MysqlPool,
}

impl Pool {
    /// `dsn` is a URL, e.g. `"mysql://user:pass@host:3306/db"`.
    pub fn connect(dsn: &str, max_size: u32) -> Result<Self> {
        let opts = Opts::from_url(dsn).map_err(conn_err)?;
        let constraints = PoolConstraints::new(1, max_size.max(1) as usize)
            .ok_or_else(|| OrmError::ConnectionError("invalid pool size".into()))?;
        let opts = OptsBuilder::from_opts(opts)
            .pool_opts(PoolOpts::default().with_constraints(constraints));
        Ok(Self { inner: MysqlPool::new(opts) })
    }

    /// Check out one connection. Hold it for the length of a transaction
    /// (`begin` … `commit` / `rollback`). Waits at most 30 s for a slot.
    pub async fn get(&self) -> Result<CheckedConn> {
        let conn = tokio::time::timeout(GET_CONN_TIMEOUT, self.inner.get_conn())
            .await
            .map_err(|_| {
                OrmError::ConnectionError(
                    "pool exhausted: timed out waiting for a connection (mysql, 30s)".into(),
                )
            })?
            .map_err(conn_err)?;
        Ok(CheckedConn { conn })
    }

    pub async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
        self.get().await?.query(sql, params).await
    }

    /// Returns the number of affected rows.
    pub async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64> {
        self.get().await?.execute(sql, params).await
    }
}

#[async_trait]
impl Db for Pool {
    fn dialect(&self) -> Option<Dialect> {
        Some(Dialect::Mysql)
    }

    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
        Pool::query(self, sql, params).await
    }

    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64> {
        Pool::execute(self, sql, params).await
    }
}

/// A checked-out MySQL connection, returned to the pool on drop.
pub struct CheckedConn {
    conn: mysql_async::Conn,
}

impl CheckedConn {
    /// `&mut` because mysql_async's `Conn` methods require it.
    pub async fn query(&mut self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
        let rows: Vec<mysql_async::Row> =
            self.conn.exec(sql, bind(params)).await.map_err(query_err)?;
        Ok(rows.into_iter().map(row_to_json).collect())
    }

    /// `&mut` because mysql_async's `Conn` methods require it.
    pub async fn execute(&mut self, sql: &str, params: &[Value]) -> Result<u64> {
        self.conn.exec_drop(sql, bind(params)).await.map_err(query_err)?;
        Ok(self.conn.affected_rows())
    }

    /// `BEGIN` on this connection. See the module docs on transactions.
    ///
    /// The three transaction-control statements go through the text protocol
    /// (`query_drop`), not `execute`: MySQL 8 rejects `BEGIN` / `COMMIT` /
    /// `ROLLBACK` in the prepared-statement protocol with error 1295.
    pub async fn begin(&mut self) -> Result<()> {
        self.conn.query_drop("BEGIN").await.map_err(query_err)
    }

    /// `COMMIT` the transaction started with [`CheckedConn::begin`].
    pub async fn commit(&mut self) -> Result<()> {
        self.conn.query_drop("COMMIT").await.map_err(query_err)
    }

    /// `ROLLBACK` the transaction started with [`CheckedConn::begin`].
    pub async fn rollback(&mut self) -> Result<()> {
        self.conn.query_drop("ROLLBACK").await.map_err(query_err)
    }
}

fn bind(params: &[Value]) -> Vec<mysql_async::Value> {
    params.iter().map(to_mysql).collect()
}

/// `Bool` → `Int(0|1)`, `Float` → `Double` (lossless), `Text` → `Bytes`,
/// `Json` → its serialized bytes (a `JSON` column accepts them).
fn to_mysql(value: &Value) -> mysql_async::Value {
    use mysql_async::Value as My;
    match value {
        Value::Null => My::NULL,
        Value::Bool(v) => My::Int(i64::from(*v)),
        Value::Int(v) => My::Int(*v),
        Value::Float(v) => My::Double(*v),
        Value::Text(v) => My::Bytes(v.as_bytes().to_vec()),
        Value::Bytes(v) => My::Bytes(v.clone()),
        Value::Json(v) => My::Bytes(v.to_string().into_bytes()),
    }
}

fn row_to_json(row: mysql_async::Row) -> Row {
    let names: Vec<String> = row.columns_ref().iter().map(|c| c.name_str().to_string()).collect();
    let mut out = Row::new();
    for (name, v) in names.into_iter().zip(row.unwrap()) {
        out.insert(name, value(v));
    }
    out
}

/// ponytail: date/time columns come back as null (no chrono dependency), and
/// bytes decode as UTF-8 text.
fn value(v: mysql_async::Value) -> Json {
    use mysql_async::Value as V;
    match v {
        V::NULL => Json::Null,
        V::Int(i) => Json::from(i),
        V::UInt(u) => Json::from(u),
        V::Float(f) => Json::from(f),
        V::Double(d) => Json::from(d),
        V::Bytes(b) => Json::from(String::from_utf8_lossy(&b).into_owned()),
        V::Date(..) | V::Time(..) => Json::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mysql_async::Value as My;

    #[test]
    fn bool_maps_to_zero_or_one() {
        assert_eq!(to_mysql(&Value::Bool(true)), My::Int(1));
        assert_eq!(to_mysql(&Value::Bool(false)), My::Int(0));
    }

    #[test]
    fn other_variants_map_one_to_one() {
        assert_eq!(to_mysql(&Value::Null), My::NULL);
        assert_eq!(to_mysql(&Value::Int(-7)), My::Int(-7));
        assert_eq!(to_mysql(&Value::Float(1.5)), My::Double(1.5));
        assert_eq!(to_mysql(&Value::Text("hi".into())), My::Bytes(b"hi".to_vec()));
        assert_eq!(to_mysql(&Value::Bytes(vec![1, 2])), My::Bytes(vec![1, 2]));
    }
}
