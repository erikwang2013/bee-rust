<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# bee_orm — Core ORM Completion (frozen interface spec)

**Status:** FROZEN. Implementers do not ask questions; if something here is
ambiguous, follow the letter of this document. Deviations must be reported to
`reviewer`, not improvised.
**Baseline:** `bee_orm` currently ships an uncommitted connection pool and a
SELECT-only `QuerySet`. This spec adds typed parameters, execution, models and
transactions.
**Write ownership (hard rule):**

| Writer | May write |
|---|---|
| `coder-orm` | `crates/bee_orm/**` (src + Cargo.toml only) |
| `coder-macro` | `crates/bee_orm_macro/**` |
| `tester` | `crates/bee_orm/tests/**` |

Nobody touches `crates/bee_rust/**`, the workspace `Cargo.toml`, or git state.

## Not in scope

Relations (FK / 1-1 / m2m / joins), migrations or schema sync, closure-style
transactions with automatic rollback, hooks (before/after save), timestamps and
soft delete, aggregates / `GROUP BY` / `HAVING`, `insert` returning a
DB-generated key. Do not add them, do not scaffold for them.

---

## 1. Frozen public surface

`bee_orm::` exports after this round: `Model`, `Value`, `FromValue`, `Db`,
`QuerySet`, `Row`, `OrmError`, `Result<T>`, `decode`, `pool::*` (unchanged
names), `bee_orm_macro::Model` (re-exported).

```rust
pub type Result<T> = std::result::Result<T, OrmError>;   // NEW alias in lib.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Value { Null, Bool(bool), Int(i64), Float(f64), Text(String), Bytes(Vec<u8>) }

#[async_trait]
pub trait Db: Send + Sync {
    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>>;
    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64>;
}

#[async_trait]
pub trait Model: Send + Sync + 'static {
    fn table_name() -> &'static str;
    fn pk_column() -> &'static str;
    fn from_row(row: &Row) -> Result<Self>
    where
        Self: Sized;                    // Result<T, _> carries an implicit T: Sized
    fn insert_values(&self) -> Vec<(&'static str, Value)>;
    fn pk_value(&self) -> Value;
    fn update_values(&self) -> Vec<(&'static str, Value)>;
    async fn insert<D: Db + ?Sized>(&self, db: &D) -> Result<u64> { /* default */ }
    async fn update<D: Db + ?Sized>(&self, db: &D) -> Result<u64> { /* default */ }
    async fn delete<D: Db + ?Sized>(&self, db: &D) -> Result<u64> { /* default */ }
}

impl<T: Model> QuerySet<T> {
    pub fn new(table: impl Into<String>) -> Self;                       // unchanged
    pub fn filter(self, condition: impl Into<String>) -> Self;          // unchanged
    pub fn filter_eq(self, field: impl Into<String>, value: impl Into<Value>) -> Result<Self>;
    pub fn filter_gt(self, field: impl Into<String>, value: impl Into<Value>) -> Result<Self>;
    pub fn filter_lt(self, field: impl Into<String>, value: impl Into<Value>) -> Result<Self>;
    pub fn filter_contains(self, field: impl Into<String>, value: impl Into<String>) -> Result<Self>;
    pub fn params(&self) -> &[Value];                                   // was &[String]
    pub fn order_by(self, clause: impl Into<String>) -> Self;           // unchanged
    pub fn limit(self, n: usize) -> Self;                               // unchanged
    pub fn offset(self, n: usize) -> Self;                              // unchanged
    pub fn to_sql(&self) -> String;                                     // unchanged output, debug only
    pub async fn all<D: Db + ?Sized>(&self, db: &D) -> Result<Vec<T>>;
    pub async fn one<D: Db + ?Sized>(&self, db: &D) -> Result<Option<T>>;
    pub async fn count<D: Db + ?Sized>(&self, db: &D) -> Result<i64>;
    pub async fn exists<D: Db + ?Sized>(&self, db: &D) -> Result<bool>;
    pub async fn update<D: Db + ?Sized>(&self, db: &D, sets: &[(&str, Value)]) -> Result<u64>;
    pub async fn delete<D: Db + ?Sized>(&self, db: &D) -> Result<u64>;
}

impl Pool {                                             // each backend, same shape
    pub async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>>;
    pub async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64>;
    // get() unchanged
}
#[async_trait] impl Db for Pool { /* delegates to the inherent methods */ }

impl CheckedConn {
    pub async fn begin(&self)    -> Result<()>;   // mysql: &mut self
    pub async fn commit(&self)   -> Result<()>;
    pub async fn rollback(&self) -> Result<()>;   // sqlite: all three are sync (&self)
}
```

Explicit generic `D: Db + ?Sized` instead of `db: &impl Db` because `?Sized`
keeps `&dyn Db` usable as a call-site type (verified: the trait is object safe
after `#[async_trait]` desugaring).

## 2. File layout and responsibilities

```
crates/bee_orm/src/
├── lib.rs          module decls + re-exports; OrmError; Result alias          (~90 lines)
├── value.rs        Value, From impls, FromValue, decode                       (~140)
├── db.rs           Db trait                                                   (~35)
├── model.rs        Model trait + insert/update/delete defaults                (~120)
├── queryset.rs     QuerySet (moved out of lib.rs) + execution + validate_field(~230)
└── pool/
    ├── mod.rs      Row alias, error helpers, docs (params are Value now)      (~45)
    ├── sqlite.rs   Pool/CheckedConn + rusqlite::ToSql for Value + tx          (~140)
    ├── postgres.rs Pool/CheckedConn + ToSql for Value + narrowing + tx        (~200)
    └── mysql.rs    Pool/CheckedConn + Value→mysql_async mapping + tx          (~130)
```

`lib.rs` re-exports so the macro paths stay frozen:
`pub use model::Model; pub use db::Db; pub use queryset::QuerySet; pub use value::{Value, FromValue, decode}; pub use pool::Row;`

Every file keeps `// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz`
as line 1, English doc comments, rustfmt defaults (`max_width = 100`), < 500
lines. No `Cargo.toml` change is needed anywhere — `tokio_postgres::types::private::BytesMut`
is available (verified in postgres-types 0.2.14: `#[doc(hidden)] pub mod private`);
using that path (rather than adding a `bytes` dependency) also guarantees the
exact `BytesMut` type the trait demands.

## 3. `value.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value { Null, Bool(bool), Int(i64), Float(f64), Text(String), Bytes(Vec<u8>) }

macro_rules! value_from_int {
    ($($ty:ty),* $(,)?) => { $(
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self { Value::Int(value as i64) }
        }
    )* };
}
value_from_int!(i8, i16, i32, i64, u8, u16, u32);   // u64/usize excluded: not lossless
impl From<f32> for Value { /* Value::Float(value as f64) */ }
impl From<f64> for Value { /* Value::Float(value) */ }
impl From<bool> for Value { /* Value::Bool(value) */ }
impl From<String> for Value { /* Value::Text(value) */ }
impl From<&str> for Value { /* Value::Text(value.to_owned()) */ }
impl From<&String> for Value { /* Value::Text(value.clone()) */ }
impl From<Vec<u8>> for Value { /* Value::Bytes(value) */ }
impl From<&[u8]> for Value { /* Value::Bytes(value.to_vec()) */ }
/// `None` binds as [`Value::Null`].
impl<T: Into<Value>> From<Option<T>> for Value { /* Some → inner.into(), None → Value::Null */ }

pub trait FromValue: Sized {
    fn from_value(value: serde_json::Value) -> Result<Self>;
}
impl<T: serde::de::DeserializeOwned> FromValue for T {
    fn from_value(value: serde_json::Value) -> Result<Self> {
        serde_json::from_value(value).map_err(|e| OrmError::QueryError(e.to_string()))
    }
}

/// Decode `column` from `row`; a missing column decodes as JSON `null`.
///
/// SQLite stores booleans as the integers 0/1 and JSON has no integer↔boolean
/// coercion, so a failed decode is retried once with that single normalisation
/// (0/1 number ↔ bool). Every other mismatch reports the original error.
pub fn decode<T: FromValue>(row: &Row, column: &str) -> Result<T> {
    let value = row.get(column).cloned().unwrap_or(serde_json::Value::Null);
    let decoded = T::from_value(value.clone()).or_else(|first| match bool_normalised(&value) {
        Some(normalised) => T::from_value(normalised).map_err(|_| first),
        None => Err(first),
    });
    decoded.map_err(|e| OrmError::QueryError(format!("column `{column}`: {e}")))
}

/// `0`/`1` as a boolean, or a boolean as `0`/`1`; `None` for anything else.
fn bool_normalised(value: &serde_json::Value) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::Number(n) => match n.as_i64() {
            Some(0) => Some(serde_json::Value::Bool(false)),
            Some(1) => Some(serde_json::Value::Bool(true)),
            _ => None,
        },
        serde_json::Value::Bool(b) => Some(serde_json::Value::from(i64::from(*b))),
        _ => None,
    }
}
```

Verified by running it: `serde_json::from_value::<bool>(json!(1))` and
`::<Option<bool>>(json!(1))` both error with "invalid type: integer `1`,
expected a boolean". Without the retry a `bool` field could never decode from
SQLite (which stores 0/1) — the one backend the offline tests can exercise.

`FromValue` exists so a column type outside serde has a decode seam; the
blanket impl covers every `Deserialize` type, so models need no serde derives.
All of the above compiles as written (checked with rustc). Verified traps:
`impl<T: Into<Value>> From<Option<T>> for Value` has no coherence conflict with
the concrete impls, and `f32::try_from(f64)` **does not exist** in std — the
PostgreSQL module must use the `as` + loss check shown in section 6.

`From<u64>`/`From<usize>` are deliberately absent: `as i64` would silently
wrap. A caller with a `u64` must decide the fallback itself.

## 4. `db.rs`

Doc note to include: *bind values always go through the driver's
prepared-statement interface; the SQL string never contains user data.*

```rust
use async_trait::async_trait;
use crate::{Result, Row, Value};

#[async_trait]
pub trait Db: Send + Sync {
    /// Run a query and collect every row.
    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>>;
    /// Run a statement, returning the number of affected rows.
    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64>;
}
```

Each backend adds, in its own module:

```rust
#[async_trait::async_trait]
impl Db for Pool {
    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
        Pool::query(self, sql, params).await          // explicit: never recurse
    }
    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64> {
        Pool::execute(self, sql, params).await
    }
}
```

The inherent `Pool::query` / `Pool::execute` stay (they carry the docs and the
sqlite `spawn_blocking` behaviour); the trait impl is a 2-line delegation.

## 5. `model.rs`

Imports: `use async_trait::async_trait; use crate::{Db, OrmError, Result, Row, Value};`
(CI runs `clippy -D warnings`, so no unused imports anywhere.)

```rust
#[async_trait]
pub trait Model: Send + Sync + 'static {
    fn table_name() -> &'static str;
    fn pk_column() -> &'static str;
    fn from_row(row: &Row) -> Result<Self>
    where
        Self: Sized;                    // Result<T, _> carries an implicit T: Sized
    fn insert_values(&self) -> Vec<(&'static str, Value)>;
    fn pk_value(&self) -> Value;
    fn update_values(&self) -> Vec<(&'static str, Value)>;

    /// `INSERT` this instance; returns affected rows.
    async fn insert<D: Db + ?Sized>(&self, db: &D) -> Result<u64> {
        let values = self.insert_values();
        if values.is_empty() {
            return Err(no_columns("insert", Self::table_name()));
        }
        let columns: Vec<&str> = values.iter().map(|(column, _)| *column).collect();
        let placeholders = vec!["?"; values.len()].join(", ");
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({placeholders})",
            Self::table_name(),
            columns.join(", ")
        );
        let params: Vec<Value> = values.into_iter().map(|(_, value)| value).collect();
        db.execute(&sql, &params).await
    }

    /// `UPDATE` this instance, addressed by primary key.
    async fn update<D: Db + ?Sized>(&self, db: &D) -> Result<u64> {
        let values = self.update_values();
        if values.is_empty() {
            return Err(no_columns("update", Self::table_name()));
        }
        let assignments: Vec<String> =
            values.iter().map(|(column, _)| format!("{column} = ?")).collect();
        let sql = format!(
            "UPDATE {} SET {} WHERE {} = ?",
            Self::table_name(),
            assignments.join(", "),
            Self::pk_column()
        );
        let mut params: Vec<Value> = values.into_iter().map(|(_, value)| value).collect();
        params.push(self.pk_value());
        db.execute(&sql, &params).await
    }

    /// `DELETE` this instance, addressed by primary key.
    async fn delete<D: Db + ?Sized>(&self, db: &D) -> Result<u64> {
        let sql = format!("DELETE FROM {} WHERE {} = ?", Self::table_name(), Self::pk_column());
        db.execute(&sql, &[self.pk_value()]).await
    }
}

fn no_columns(op: &str, table: &str) -> OrmError {
    OrmError::QueryError(format!("{op} on `{table}`: no columns to write"))
}
```

`Self: Send + Sync + 'static` (existing marker) is what `#[async_trait]` needs;
keep it. Column names in the generated SQL come from the macro, which validates
them (section 8), so they are safe to splice.

`where Self: Sized` on `from_row` is required, not stylistic: the trait's `Self`
is `?Sized` and `Result<T, E>` carries an implicit `T: Sized`, so the method does
not compile without it (rustc E0277, verified). Implementing it for a concrete
struct needs no repeated clause, so the macro's generated impl is unaffected.

## 6. Pool modules

### 6.1 Common (all three)

- Every `query`/`execute` signature takes `params: &[Value]`; module doc
  examples switch from `&["18".to_string()]` to `&[bee_orm::Value::from(18)]`.
- **Import collision:** `postgres.rs` and `mysql.rs` already have
  `use serde_json::Value;` for the read-side helpers. Rename that import to
  `use serde_json::Value as Json;` and add `use crate::Value;`, then spell the
  read-side helpers with `Json`. Leaving both under the name `Value` is a real
  trap: `Value::Null` exists in *both* enums and would silently mean serde_json's.
  (`sqlite.rs` is safe: its `use serde_json::Value;` is function-local and
  shadows the module-level `crate::Value` only inside that helper.)
- **Alias collision:** `crate::Result<T>` is a single-parameter alias, so any
  module importing it can no longer spell a two-parameter
  `std::result::Result`. `postgres.rs` has exactly one such use — the read-side
  helper `fn opt<T>(v: Result<Option<T>, tokio_postgres::Error>) -> Json` —
  qualify that one as `std::result::Result<...>`.
- `Row` keeps its `serde_json::Map<String, serde_json::Value>` shape and the
  read-side decode helpers keep their behaviour (only the `Json` spelling from
  the import note below changes).
- Add a transaction paragraph to the docs of `pool/mod.rs` and of each backend:

  > Transactions must run on a connection held from `Pool::get()`: statements
  > sent through the `Pool` itself may each land on a different connection.
  > `CheckedConn::begin` / `commit` / `rollback` issue `BEGIN` / `COMMIT` /
  > `ROLLBACK`. Nothing rolls back automatically: always end a transaction with
  > `commit()` or `rollback()` before the `CheckedConn` goes out of scope — an
  > open transaction returned to the pool can leak into the next check-out on
  > backends whose pools do not reset connections.

- `CheckedConn` transaction methods are thin wrappers, `execute(sql, &[])` then
  `.map(|_| ())`. Raw `BEGIN`/`COMMIT`/`ROLLBACK` text is used on all three
  backends (uniform, and valid on all three servers). Signature differs only by
  the driver's receiver convention: sqlite `pub fn begin(&self) -> Result<()>`
  (sync), postgres `pub async fn begin(&self)`, mysql
  `pub async fn begin(&mut self)`.
- **Amendment (round 3, real-DB evidence):** "valid on all three servers"
  holds for the SQL text, not for every wire protocol: MySQL 8 rejects `BEGIN`
  sent through the prepared-statement protocol (`ERROR 1295`), and the mysql
  wrapper routed it through `exec_drop` — so mysql transactions were unusable
  on a real server (round 2 had only compile/unit coverage). Ruling: mysql's
  `begin` / `commit` / `rollback` all issue their statements over the text
  protocol (`query_drop`, uniform three-piece); the thin-wrapper shape and the
  absence of transaction state tracking stay unchanged. Tester's real-DB run
  confirmed text-protocol `BEGIN` → `INSERT` → `ROLLBACK` rolls back on
  `mysql:8.0`; pg (`BEGIN`/`ROLLBACK` via the extended protocol) and sqlite
  (`execute_batch`) are unaffected — both re-verified real-DB in the same run.

### 6.2 sqlite

```rust
impl rusqlite::ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        use rusqlite::types::{ToSqlOutput, ValueRef};
        Ok(ToSqlOutput::Borrowed(match self {
            Value::Null => ValueRef::Null,
            Value::Bool(b) => ValueRef::Integer(i64::from(*b)),   // stored as 0 / 1
            Value::Int(i) => ValueRef::Integer(*i),
            Value::Float(f) => ValueRef::Real(*f),
            Value::Text(s) => ValueRef::Text(s.as_bytes()),
            Value::Bytes(b) => ValueRef::Blob(b),
        }))
    }
}
```

`CheckedConn::query` / `execute` keep `params_from_iter(params)` — with
`params: &[Value]` the iterator item is `&Value`, which is `ToSql` through
rusqlite's blanket reference impl (verified in rusqlite 0.32.1). `false`/
`true` read back as integers `0`/`1`; `Value::Bytes` writes a BLOB that reads
back as a JSON array of byte numbers (existing read-side behaviour).

### 6.3 postgres

`bind` becomes:

```rust
fn bind(params: &[Value]) -> Vec<&(dyn ToSql + Sync)> {
    params.iter().map(|p| p as &(dyn ToSql + Sync)).collect()
}
```

The parameter's PostgreSQL type is chosen by the *statement*, so `Value::Int`
must be narrowed to the target column type. Implement `ToSql` exactly like this
(`tokio_postgres::types::private::BytesMut` is the same `BytesMut` the trait
declares):

```rust
impl ToSql for Value {
    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut tokio_postgres::types::private::BytesMut,
    ) -> std::result::Result<
        tokio_postgres::types::IsNull,
        Box<dyn std::error::Error + Sync + Send>,
    > {
        use tokio_postgres::types::IsNull;
        match (self, ty.name()) {
            (Value::Null, _) => Ok(IsNull::Yes),
            (Value::Bool(v), "bool") => {
                out.extend_from_slice(&[u8::from(*v)]);
                Ok(IsNull::No)
            }
            (Value::Int(v), "int2") => {
                out.extend_from_slice(
                    &i16::try_from(*v).map_err(|_| range_error(v, "int2"))?.to_be_bytes(),
                );
                Ok(IsNull::No)
            }
            (Value::Int(v), "int4") => {
                out.extend_from_slice(
                    &i32::try_from(*v).map_err(|_| range_error(v, "int4"))?.to_be_bytes(),
                );
                Ok(IsNull::No)
            }
            (Value::Int(v), "int8") => {
                out.extend_from_slice(&v.to_be_bytes());
                Ok(IsNull::No)
            }
            (Value::Float(v), "float4") => {
                out.extend_from_slice(&narrow_f32(*v)?.to_be_bytes());
                Ok(IsNull::No)
            }
            (Value::Float(v), "float8") => {
                out.extend_from_slice(&v.to_be_bytes());
                Ok(IsNull::No)
            }
            (Value::Text(v), "text" | "varchar" | "bpchar" | "name" | "unknown") => {
                out.extend_from_slice(v.as_bytes());
                Ok(IsNull::No)
            }
            (Value::Bytes(v), "bytea") => {
                out.extend_from_slice(v);
                Ok(IsNull::No)
            }
            (value, target) => Err(mismatch_error(value, target)),
        }
    }

    fn accepts(_ty: &tokio_postgres::types::Type) -> bool { true }

    /// Equivalent to what `postgres_types::to_sql_checked!()` generates, with
    /// the `accepts` check folded out: this impl accepts every type and reports
    /// a mismatch from `to_sql` instead (messages name the offending column type).
    fn to_sql_checked(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut tokio_postgres::types::private::BytesMut,
    ) -> std::result::Result<
        tokio_postgres::types::IsNull,
        Box<dyn std::error::Error + Sync + Send>,
    > {
        self.to_sql(ty, out)
    }
}

/// Narrow `f64` to `f32`, rejecting conversions that lose the value entirely:
/// overflow to infinity, underflow to zero. In-range rounding is unavoidable
/// and allowed.
fn narrow_f32(value: f64) -> std::result::Result<f32, Box<dyn std::error::Error + Sync + Send>> {
    let narrowed = value as f32;
    let lost = (narrowed.is_infinite() && value.is_finite()) || (narrowed == 0.0 && value != 0.0);
    if lost { Err(range_error(value, "float4")) } else { Ok(narrowed) }
}

fn range_error(value: impl std::fmt::Display, target: &str) -> Box<dyn std::error::Error + Sync + Send> {
    format!("value `{value}` does not fit PostgreSQL {target}").into()
}

/// Names the variant only — never the data — so error messages cannot leak values.
fn mismatch_error(value: &Value, target: &str) -> Box<dyn std::error::Error + Sync + Send> {
    let kind = match value {
        Value::Null => "Null",
        Value::Bool(_) => "Bool",
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::Text(_) => "Text",
        Value::Bytes(_) => "Bytes",
    };
    format!("cannot bind Value::{kind} as PostgreSQL {target}").into()
}
```

Notes the coder must respect:
- Write `std::result::Result` (as above) in this impl:
  `crate::Result<T>` is the single-parameter alias and must not be in scope here.
- Never truncate: `i16::try_from` / `i32::try_from` overflow is an error. This is
  the one behaviour that must have an offline test (section 9).
- Match on `ty.name()` (a `&'static str`) exactly as the existing read-side
  `value()` helper does; do not match on `Type` constants.
- Delegating `to_sql_checked` to `to_sql` (instead of invoking the
  `to_sql_checked!` macro) is deliberate: `accepts` is per-type and cannot see
  the value, so the precise error can only come from `to_sql`.
- `Value::Text` bound to a `json`/`jsonb` column is an error (not in scope);
  `oid`, `numeric`, date/time, arrays are errors too.

### 6.4 mysql

```rust
fn bind(params: &[Value]) -> Vec<mysql_async::Value> {
    params.iter().map(to_mysql).collect()
}

/// `Bool` → `Int(0|1)`, `Float` → `Double` (lossless), `Text` → `Bytes`.
fn to_mysql(value: &Value) -> mysql_async::Value {
    use mysql_async::Value as My;
    match value {
        Value::Null => My::NULL,
        Value::Bool(v) => My::Int(i64::from(*v)),
        Value::Int(v) => My::Int(*v),
        Value::Float(v) => My::Double(*v),
        Value::Text(v) => My::Bytes(v.as_bytes().to_vec()),
        Value::Bytes(v) => My::Bytes(v.clone()),
    }
}
```

`Vec<mysql_async::Value>` already converts into the driver's `Params` (the
existing code relies on this). The read-side `value()` helper keeps its
behaviour and only takes the `Json` spelling from the import note in 6.1.

## 7. `queryset.rs`

Move the whole `QuerySet` out of `lib.rs` (unchanged public API except
`params()` → `&[Value]` and `filter_eq/gt/lt` taking `impl Into<Value>`), and
add execution. Imports:
`use std::marker::PhantomData; use crate::{Db, Model, OrmError, Value};`
(`serde_json::Value::as_i64` is spelled fully qualified, no import needed).

```rust
pub struct QuerySet<T: Model> {
    table: String,
    filters: Vec<String>,
    params: Vec<Value>,
    order_clauses: Vec<String>,
    limit_val: Option<usize>,
    offset_val: Option<usize>,
    _marker: PhantomData<T>,
}

/// Build the SQL string for this query (debugging and tests only).
pub fn to_sql(&self) -> String { self.select_sql(None) }

fn select_sql(&self, limit_override: Option<usize>) -> String {
    // "SELECT * FROM {table}{where}{order}{limit}{offset}";
    // limit = limit_override.or(self.limit_val), offset = self.offset_val
}

fn where_sql(&self) -> String {
    // "" when there are no filters, else " WHERE a AND b"
}

/// Run the query and decode every row into `T`.
pub async fn all<D: Db + ?Sized>(&self, db: &D) -> Result<Vec<T>> {
    let rows = db.query(&self.select_sql(None), &self.params).await?;
    rows.iter().map(T::from_row).collect()
}

/// `LIMIT 1`; decodes the first row, if any. Overrides any earlier `limit()`.
pub async fn one<D: Db + ?Sized>(&self, db: &D) -> Result<Option<T>> {
    let rows = db.query(&self.select_sql(Some(1)), &self.params).await?;
    rows.first().map(T::from_row).transpose()
}

/// `SELECT COUNT(*) AS count` with the WHERE clause only — ORDER BY / LIMIT /
/// OFFSET are ignored.
pub async fn count<D: Db + ?Sized>(&self, db: &D) -> Result<i64> {
    let sql = format!("SELECT COUNT(*) AS count FROM {}{}", self.table, self.where_sql());
    let rows = db.query(&sql, &self.params).await?;
    rows.first()
        .and_then(|row| row.get("count"))
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| OrmError::QueryError("count: unexpected result shape".into()))
}

/// Whether at least one row matches (`SELECT 1 … LIMIT 1`; no row is decoded).
pub async fn exists<D: Db + ?Sized>(&self, db: &D) -> Result<bool> {
    let sql = format!("SELECT 1 FROM {}{} LIMIT 1", self.table, self.where_sql());
    Ok(!db.query(&sql, &self.params).await?.is_empty())
}

/// `UPDATE` the matching rows. SET parameters are bound before WHERE parameters.
pub async fn update<D: Db + ?Sized>(&self, db: &D, sets: &[(&str, Value)]) -> Result<u64> {
    if sets.is_empty() {
        return Err(OrmError::QueryError("update: no columns to set".into()));
    }
    let mut assignments = Vec::with_capacity(sets.len());
    let mut params: Vec<Value> = Vec::with_capacity(sets.len() + self.params.len());
    for (field, value) in sets {
        validate_field(field)?;
        assignments.push(format!("{field} = ?"));
        params.push(value.clone());
    }
    params.extend_from_slice(&self.params);
    let sql = format!("UPDATE {} SET {}{}", self.table, assignments.join(", "), self.where_sql());
    db.execute(&sql, &params).await
}

/// `DELETE` the matching rows. With no filter this deletes every row.
pub async fn delete<D: Db + ?Sized>(&self, db: &D) -> Result<u64> {
    let sql = format!("DELETE FROM {}{}", self.table, self.where_sql());
    db.execute(&sql, &self.params).await
}
```

`validate_field` moves here with `queryset.rs` and needs no change (still private,
`Result<(), OrmError>`, `InvalidField` on rejection). `to_sql()` output is byte
for byte what it is today — every existing `to_sql` assertion in
`tests/orm_tests.rs` must still pass unmodified.

Decisions: `count()` drops ORDER BY/LIMIT/OFFSET (they cannot change a count);
`exists()` always forces `LIMIT 1`; both are documented on the methods. A
filter-less `delete()` deletes everything — plain SQL semantics, documented, no
guard (a guard would block a legitimate full-table delete).

## 8. `bee_orm_macro` v2

### 8.1 Attribute grammar

Struct level (one string literal, any number of `#[bee(...)]` attributes):

| key | meaning | default |
|---|---|---|
| `table = "..."` | table name | struct name lowercased + `"s"` (`User` → `users`, unchanged) |

Field level, combinable in one attribute (`#[bee(pk, auto)]`):

| key | meaning |
|---|---|
| `column = "..."` | column name; must match `[A-Za-z_][A-Za-z0-9_]*` |
| `pk` | this field is the primary key |
| `auto` | database-assigned; skipped by `insert_values` |
| `ignore` | not a table column; excluded from insert/update/from_row |

Resolution rules: field name is the column name unless `column` overrides it;
the primary key is the `#[bee(pk)]` field, or else the field literally named
`id` when no field is marked. Unknown keys are errors (`meta.error(...)`).

### 8.2 Compile errors (all via `syn::Error::to_compile_error()`, spanned at the offending item)

| condition | message |
|---|---|
| not a struct | `bee_orm: Model can only be derived for a struct` |
| unnamed / unit fields | `bee_orm: Model requires a struct with named fields` |
| lifetime or type params | `bee_orm: Model does not support generic structs` |
| unknown struct key | `bee_orm: unknown bee attribute for a struct (expected table)` |
| unknown field key | `bee_orm: unknown bee attribute for a field (expected column, pk, auto, ignore)` |
| two `pk` fields | `` bee_orm: multiple #[bee(pk)] fields (`a`, `b`) `` |
| `pk` + `ignore` on one field | `bee_orm: #[bee(pk)] and #[bee(ignore)] on the same field` |
| `auto` without `pk` | `bee_orm: #[bee(auto)] is only valid on the primary key field` |
| no pk resolvable | `bee_orm: no primary key: mark a field #[bee(pk)] or name it \`id\`` |
| duplicate column name | `` bee_orm: duplicate column name `x` `` |
| bad `column` literal | `` bee_orm: invalid column name `x` (expected [A-Za-z_][A-Za-z0-9_]*) `` |

`table` values are trusted literals from the user's own source (same trust level
as `QuerySet::filter`); they are spliced verbatim, only columns are validated.
`#[bee(ignore)]` fields are built with `Default::default()`, so their type must
be `Default` — a missing impl is a normal rustc error at the generated call.

### 8.3 Generated code contract

`Table`/`User` example — `#[derive(Model)] #[bee(table = "user_accounts")]` on

```rust
struct User {
    #[bee(pk, auto)] id: i64,
    #[bee(column = "user_name")] name: String,
    age: Option<i32>,
    #[bee(ignore)] avatar_cache: Vec<u8>,
}
```

must expand to exactly this shape:

```rust
impl bee_orm::Model for User {
    fn table_name() -> &'static str { "user_accounts" }

    fn pk_column() -> &'static str { "id" }

    fn from_row(row: &bee_orm::Row) -> bee_orm::Result<Self> {
        Ok(Self {
            id: bee_orm::decode(row, "id")?,
            name: bee_orm::decode(row, "user_name")?,
            age: bee_orm::decode(row, "age")?,
            avatar_cache: ::core::default::Default::default(),
        })
    }

    fn insert_values(&self) -> ::std::vec::Vec<(&'static str, bee_orm::Value)> {
        ::std::vec![
            ("user_name", bee_orm::Value::from(self.name.clone())),
            ("age", bee_orm::Value::from(self.age.clone())),
        ]
    }

    fn pk_value(&self) -> bee_orm::Value {
        bee_orm::Value::from(self.id.clone())
    }

    fn update_values(&self) -> ::std::vec::Vec<(&'static str, bee_orm::Value)> {
        ::std::vec![
            ("user_name", bee_orm::Value::from(self.name.clone())),
            ("age", bee_orm::Value::from(self.age.clone())),
        ]
    }
}

impl User {
    /// The table this model is mapped to.
    pub fn table_name() -> &'static str { "user_accounts" }

    /// Build a query set for this model's table.
    pub fn query() -> bee_orm::QuerySet<Self> { bee_orm::QuerySet::new("user_accounts") }
}
```

Rules encoded above:
- field declaration order everywhere; `insert_values` skips `auto` and `ignore`;
  `update_values` skips `pk` and `ignore`.
- `self.field.clone()` relies on the field type being `Clone` with
  `Value: From<FieldType>`; a type offering neither simply cannot be a mapped
  column — that is a deliberate constraint, not a bug to route around.
- `bee_orm::` prefix on every path (no `use` required at the call site), plus
  `::std::` / `::core::` absolute paths inside expressions so user imports
  cannot shadow them.
- The inherent `table_name()` and `query()` are kept. The inherent associated fn
  wins name resolution for `User::table_name()` (verified with rustc), so the
  existing test keeps passing; the trait method is reachable as
  `<User as Model>::table_name()`.
- The `#[async_trait]` trait defaults provide `insert`/`update`/`delete`; the
  derive never generates async code.
- Only `syn = "2" (full)`, `quote`, `proc-macro2` are needed — no macro-crate
  `Cargo.toml` change.

## 9. Tests

### 9.1 In `src/` (writer: `coder-orm`)

Backend unit tests must live next to the code: `tokio_postgres` and
`mysql_async` types are not re-exported to the integration tests (a dev-dep on
them would build those stacks even without the feature). Both are offline.

- `src/value.rs` — `Value::from(1u8) == Value::Int(1)`,
  `Value::from(None::<i32>) == Value::Null`, `Value::from(Some(3i32)) == Value::Int(3)`;
  build a `Row` inline with `active` = JSON `1` and assert
  `decode::<bool>` → `Ok(true)`, `decode::<Option<bool>>` → `Ok(Some(true))`;
  the same row with `active` = JSON `5` is an `Err`; a missing column decodes
  as `Ok(None)` for `Option<i32>` and is an `Err` for `i32`.
- `src/pool/postgres.rs` — `<Value as ToSql>::to_sql(...)` with
  `&Type::INT4` on `Value::Int(5)` writes `5i32.to_be_bytes()`, **and**
  `Value::Int(i64::from(i32::MAX) + 1)` on `&Type::INT4` is `Err` (the
  no-truncation rule), same for `int2`; `f64::MAX` on `&Type::float4` is `Err`;
  `Value::Null` is `Ok(IsNull::Yes)` with an untouched buffer; `Value::Text` on
  `&Type::INT4` is `Err`. Buffer:
  `tokio_postgres::types::private::BytesMut::new()`.
- `src/pool/mysql.rs` — `to_mysql(&Value::Bool(true)) == mysql_async::Value::Int(1)`
  and the `Text`/`Bytes`/`Null` mappings.

### 9.2 In `tests/` (writer: `tester`)

`tests/orm_tests.rs` (no feature gate; update + extend):
- All `to_sql()` assertions stay exactly as they are.
- `params()` assertions become `Value` comparisons, e.g.
  `assert_eq!(qs.params(), &[Value::from("o'neil")])` (this already compiles as
  written today, because `&str: Into<Value>`), and the comparison filters pass
  numbers instead of strings: `filter_gt("age", 18)` →
  `assert_eq!(qs.params(), &[Value::Int(18), Value::Int(65)])`.
- `User::table_name()` (inherent) unchanged; add a `Model` trait-level check.
- Model 2 exercising `#[bee(table = "user_accounts", column = "...", pk, auto, ignore)]`
  through `table_name` / `pk_column` / `insert_values` / `update_values` /
  `pk_value` / `from_row` (call `from_row` on a hand-built `Row`), including a
  `None` → null case and an `ignore` field defaulting.
- Compile-failure paths are **not** automated (would need a new `trybuild`
  dev-dependency); they are covered by `reviewer` reading the macro.

`tests/pool_sqlite.rs` (update): params become `Value` (`alice` stays text,
`30` becomes `Value::Int(30)`); the transaction test uses `begin()` /
`rollback()` / `commit()` instead of raw `"BEGIN"` strings, and asserts that a
committed row is visible and a rolled-back one is not.

`tests/orm_sqlite.rs` (new, starts with `#![cfg(feature = "sqlite")]`) — the
end-to-end acceptance test, table
`CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, age INTEGER, active INTEGER, data BLOB)`
and a model with `#[bee(pk, auto)] id: i64`, `name: String`, `age: Option<i32>`,
`active: bool`, `data: Vec<u8>`:

1. `model.insert(&pool)` → `1` affected; the `auto` pk was not sent (the row
   still has a DB-assigned id).
2. `User::query().filter_eq("name", "alice")?.one(&pool)` → `Some`, decoded
   fields equal the input, `id != 0`; `active` reads back as `true` from the
   stored `1`.
3. Same filter with a non-matching name → `None`.
4. `filter_gt("age", 18)` + `count` / `exists` on a matching and a
   non-matching filter.
5. `all()` with `order_by("id")` returns the inserted rows in order.
6. `found.update(&pool)` (model method) then re-read → fields changed; a second
   model value carrying a different pk must not be touched (pk-scoped WHERE).
7. `User::query().filter_eq("id", id)?.update(&pool, &[("name", "bob".into())])`
   → 1 affected and a non-matching row's name unchanged (proves SET-before-WHERE
   ordering).
8. `found.delete(&pool)` → 1; `count` drops by one; `QuerySet::delete(&pool)`
   with a filter, plus one unfiltered delete on an emptied table.
9. `let db: &dyn Db = &pool; User::query().all(db)` — object-safety smoke test.
10. Binding fidelity: insert `Value::from(1.5f64)`, `Value::Text`,
    `Value::Null` (via a second table or columns) and assert the JSON shapes
    that come back (`1.5`, text, `null`); a `Bytes` parameter round-trips to a
    JSON array.

No test may require a running PostgreSQL or MySQL server. CI runs
`--all-features`, so everything must at least compile offline.

## 10. Acceptance commands

Run from `/home/wwwroot/bee-rust`; all must pass before `reviewer` is pinged.

```bash
cargo fmt --all                                        # CI runs --check
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings  # exactly what CI runs
cargo test --workspace                                 # CI gate (default features)
cargo test --workspace --all-features                  # CI gate (sqlite+pg+mysql compile & run offline tests)
cargo test -p bee_orm --all-features                   # focused loop
cargo test -p bee_orm_macro                            # macro crate still builds
```

`cargo tree -p bee_orm --all-features | grep -c bytes` must be unchanged by
this round (no new direct dependency).

## 11. Decision log (one line each)

- Explicit `D: Db + ?Sized` over `db: &impl Db`: keeps `&dyn Db` call sites
  working; verified against async-trait 0.1.91 with rustc.
- `FromValue` + blanket `DeserializeOwned` impl + `decode` helper: one
  trait, no per-type code, and one place to attach the column name to an error.
- `decode` retries once with 0/1↔bool normalisation: SQLite stores booleans as
  integers and JSON refuses the coercion, so `bool` fields would otherwise be
  undecodable there; the retry only fires after a real decode failure.
- `Value` gets `PartialEq`/`Clone`/`Debug` but no serde derives: nothing needs
  to serialize a `Value`.
- No `From<u64>`: `as i64` would wrap silently; callers decide.
- PostgreSQL encodes by `ty.name()` (matches the existing read-side style) and
  errors instead of truncating; float4 uses `as` + a loss check because
  `f32::try_from(f64)` does not exist.
- `to_sql_checked` delegates instead of using `to_sql_checked!()`: `accepts` is
  value-blind, so the good error message can only come from `to_sql`.
- `BytesMut` from `tokio_postgres::types::private`: no new dependency and it is
  guaranteed to be the trait's own type.
- Pool keeps its inherent `query`/`execute`; `impl Db` delegates: zero breakage
  for existing callers, and feature-agnostic code gets the trait.
- Transactions are raw `BEGIN`/`COMMIT`/`ROLLBACK` on the checked-out
  connection, no auto-rollback on drop: uniform across drivers.
  (Superseded in round 2 by §15: sqlite and postgres now best-effort roll
  back on drop; mysql relies on the driver's check-out reset.)
- `count()` ignores ORDER BY/LIMIT/OFFSET; `exists()` forces `LIMIT 1`;
  unfiltered `delete()` is allowed — each documented on the method.
- `insert` on a model with zero writable columns is an `OrmError::QueryError`
  instead of a backend-specific empty-`VALUES` syntax error.
- `Model::from_row` carries `where Self: Sized` (imposed by `Result<Self>`, not a
  choice); the trait was already not object-safe, so nothing is lost.
- `table` literals are trusted (user source), `column` literals are validated:
  injection can only come from a value, and values are always bound.
- QuerySet moves to `queryset.rs` so every file stays well under 500 lines.

---

# Round 2 (2026-10-05): pool hardening

## 12. Scope, ownership, item → section map

Round 2 hardens the three connection pools. Nothing in §1–§11 changes unless a
section below says so explicitly (one decision-log line in §11 is marked
superseded).

| Item | Subject | Section | Writer |
| --- | --- | --- | --- |
| ① | pool-exhaustion wait timeouts | §13 | `coder-orm` |
| ② | PostgreSQL prepared-statement cache | §14 | `coder-orm` |
| ③ | transaction drop auto-rollback | §15 | `coder-orm` |
| ④ | PostgreSQL TLS (`postgres-tls` feature) | §16 | `coder-orm` |
| ⑤ | `Pool::status()` passthrough | §17 | `coder-orm` |
| (a)(b)(c) | derive-macro rulings from review | §18 | `coder-macro` |

Write ownership is unchanged: `coder-orm` → `crates/bee_orm/**` plus
`.github/workflows/ci.yml`; `coder-macro` → `crates/bee_orm_macro/**`;
`tester` → `tests/**`. `#[cfg(test)]` blocks inside `src/` belong to the file's
writer (`coder-orm`), as in round 1.

Not in scope for round 2 (deliberate omissions, not forgotten):

- configurable pool timeouts (a `PoolConfig` / builder knob);
- TLS for sqlite/mysql, custom CAs, client certificates, `sslmode` overrides;
- native OS certificate stores (bundled webpki roots only);
- exposing statement-cache clear/size APIs on `bee_orm::Pool`;
- mysql pool statistics (`mysql_async` has no stats API);
- retry/reconnect logic on top of connection errors.

### 12.1 Files touched

| File | Change |
| --- | --- |
| `crates/bee_orm/Cargo.toml` | feature wiring (`dep:tokio`, `postgres-tls`), three new optional deps |
| `crates/bee_orm/src/pool/mod.rs` | Transactions paragraph (§15.4) |
| `crates/bee_orm/src/pool/postgres.rs` | §13.1, §14, §15.1, §16, §17 |
| `crates/bee_orm/src/pool/sqlite.rs` | §15.2, §17 |
| `crates/bee_orm/src/pool/mysql.rs` | §13.3, §15.4 (docs only) |
| `.github/workflows/ci.yml` | one new build step (§16) |
| `crates/bee_orm_macro/src/lib.rs` | §18 unraw rule + duplicate `table` error |
| `crates/bee_orm_macro/Cargo.toml`, `tests/ui/**` | §18.3 trybuild UI cases + dev-deps |
| `Cargo.lock` | generated: `trybuild` (macro), TLS stack (bee_orm) |

## 13. Pool-exhaustion wait timeouts (item ①)

Motivation, verified against the pinned sources: deadpool's `Timeouts::new()`
leaves every field `None` (`deadpool-0.13.1/src/managed/config.rs:78-83`) and
deadpool-postgres 0.14.2 passes an all-empty `Timeouts`, so a full pool makes
`get()` wait forever. r2d2 already defaults `connection_timeout` to 30 s.
`mysql_async` 0.34.2 has no acquire timeout at all.

### 13.1 PostgreSQL (`pool/postgres.rs`)

`connect()` and `connect_tls()` (§16) both set:

```rust
use std::time::Duration;
use deadpool_postgres::{Manager, ManagerConfig, Pool as DeadpoolPool, Runtime, Timeouts};

let inner = DeadpoolPool::builder(manager)
    .max_size(max_size.max(1) as usize)
    // ponytail: fixed timeouts — turn into a PoolConfig parameter when a
    // caller needs other values.
    .timeouts(Timeouts {
        wait: Some(Duration::from_secs(30)),
        create: Some(Duration::from_secs(10)),
        recycle: None,
    })
    // Required, not decorative: deadpool 0.13 refuses to build a pool whose
    // Timeouts are non-empty while no runtime is set — `build()` returns
    // `BuildError::NoRuntimeSpecified` (deadpool-0.13.1/src/managed/builder.rs:90-96).
    // The tag is only consulted when a `get()` wait actually times out, so
    // `build()` still succeeds outside a tokio context — the §17 offline
    // status test stays a plain `#[test]`.
    .runtime(Runtime::Tokio1)
    .build()
    .map_err(conn_err)?;
```

`Runtime` is reachable through deadpool-postgres (`pub use
deadpool::managed::reexports::*;` at `deadpool-postgres-0.14.2/src/lib.rs:61`;
the re-export list at `deadpool-0.13.1/src/managed/reexports.rs:19-22` includes
`Runtime`, next to `Timeouts`/`TimeoutType`/`Status`), and `rt_tokio_1` is
deadpool-postgres's default feature — no direct `deadpool` dependency needed.

`get()` maps the wait timeout to a distinct message:

```rust
pub async fn get(&self) -> Result<CheckedConn> {
    let conn = self.inner.get().await.map_err(pool_err)?;
    Ok(CheckedConn { conn: Some(conn), in_transaction: AtomicBool::new(false) })
}

/// Wait-timeout errors get a distinct message; every other pool error keeps
/// the driver's text.
fn pool_err(e: deadpool_postgres::PoolError) -> OrmError {
    match e {
        deadpool_postgres::PoolError::Timeout(deadpool_postgres::TimeoutType::Wait) => {
            OrmError::ConnectionError(
                "pool exhausted: timed out waiting for a connection (postgres, 30s)".into(),
            )
        }
        e => conn_err(e),
    }
}
```

`PoolError`, `TimeoutType`, `Timeouts` and `Status` are all reachable through
`deadpool_postgres::` (it re-exports `deadpool::managed::reexports::*` —
`deadpool-postgres-0.14.2/src/lib.rs:61`, names listed in
`deadpool-0.13.1/src/managed/reexports.rs:19`). The driver's own `Display` for
this case is `Timeout occurred while waiting for a slot to become available`;
the variant match above is preferred over string matching.

Offline test (in `src/`): `deadpool_postgres::PoolError::Timeout(TimeoutType::Wait)`
constructs without a server — assert the mapped `OrmError::ConnectionError`
contains `pool exhausted`.

### 13.2 SQLite (`pool/sqlite.rs`) — confirmed, no change

r2d2 0.8.10 sets `connection_timeout: Duration::from_secs(30)` in the default
config (`src/config.rs:61`) and `Pool::get()` is
`get_timeout(self.connection_timeout)` (`src/lib.rs:410-411`). On expiry the
error's `Display` is exactly `timed out waiting for connection` (bare, no
inner detail — `get_timeout` returns `Error(internals.last_error.take())`,
`src/lib.rs:440`), which the existing `conn_err` already surfaces as
`OrmError::ConnectionError`. Quirk worth knowing: every r2d2 error shares that
description, so a connection-level failure reads `<phrase>: <detail>`; the
bare phrase is the wait timeout. No code change.

### 13.3 MySQL (`pool/mysql.rs`)

`mysql_async`'s `Pool` exposes `get_conn()` with no wait-timeout knob (only
`inactive_connection_ttl`, unrelated — checked 0.34.2 `src/conn/pool/mod.rs`).
Wrap it:

```rust
use std::time::Duration;

/// ponytail: fixed 30s — mysql_async 0.34.2 has no acquire timeout; make it
/// configurable when a caller needs to tune it.
const GET_CONN_TIMEOUT: Duration = Duration::from_secs(30);

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
```

Dropping the timed-out future removes the waiter from the queue
(`impl Drop for GetConn`, `mysql_async-0.34.2/src/conn/pool/futures/get_conn.rs:175`),
so the timeout leaks no wait slot. `Cargo.toml`: `mysql = ["dep:mysql_async",
"dep:tokio"]` — the workspace `tokio` has `features = ["full"]`, so `time` is
available. (`sqlite` already lists `dep:tokio`.)

## 14. PostgreSQL prepared-statement cache (item ②)

Today `CheckedConn::query`/`execute` pass SQL text straight to tokio-postgres,
which re-Parses and re-Describes every call. deadpool-postgres gives every
pooled client a `StatementCache` (`Arc<StatementCache>` per client, with a
registry on the `Manager`).

```rust
pub async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
    let statement = self.client().prepare_cached(&pg_sql(sql)).await.map_err(query_err)?;
    let rows = self.client().query(&statement, &bind(params)).await.map_err(query_err)?;
    Ok(rows.iter().map(row_to_json).collect())
}

pub async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64> {
    let statement = self.client().prepare_cached(&pg_sql(sql)).await.map_err(query_err)?;
    self.client().execute(&statement, &bind(params)).await.map_err(query_err)
}
```

- `prepare_cached(&self, query: &str) -> Result<Statement, Error>`
  (`deadpool-postgres-0.14.2/src/lib.rs:259`), reached through
  `Object → ClientWrapper` deref; `Statement: ToStatement`, so
  `query(&statement, …)` / `execute(&statement, …)` resolve on the inner
  `tokio_postgres::Client`.
- Module docs must add: the cache is per connection, keyed by (SQL text,
  parameter types), and **unbounded** — it grows with the number of distinct
  statements one connection ever sees, and entries are never evicted
  automatically. If DDL invalidates cached plans, clear it with
  `pool.manager().statement_caches.clear()` (`manager()`:
  `deadpool-0.13.1/src/managed/pool.rs:372`; `StatementCaches::clear`:
  `deadpool-postgres-0.14.2/src/statement_cache.rs`).
- `BEGIN` / `COMMIT` / `ROLLBACK` also flow through `execute` and get cached;
  they are constant strings, so this adds exactly three entries per connection.
- Not offline-testable beyond compiling: a `Statement` cannot be constructed
  without a live connection.

## 15. Transaction drop auto-rollback (item ③)

Round 1 shipped "nothing rolls back automatically". Round 2 adds a best-effort
safety net so a forgotten transaction cannot leak into the next check-out of
the same connection. Explicit `commit()` / `rollback()` remains the required
practice — the hook bounds the damage of forgetting, it does not replace
ending transactions.

### 15.1 PostgreSQL (`pool/postgres.rs`)

`CheckedConn` reshapes: the connection moves into an `Option` (so `Drop` can
move it into a spawned task) plus an `AtomicBool` flag. `AtomicBool` rather
than `Cell` to keep `CheckedConn: Send + Sync`; every method keeps `&self`:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

pub struct CheckedConn {
    conn: Option<deadpool_postgres::Object>,
    in_transaction: AtomicBool,
}

impl CheckedConn {
    fn client(&self) -> &deadpool_postgres::Object {
        self.conn.as_ref().expect("connection is taken only in Drop")
    }
    // query/execute (§14) call self.client() and no longer touch self.conn
    // directly. begin/commit/rollback keep their signatures and add the flag:

    pub async fn begin(&self) -> Result<()> {
        self.execute("BEGIN", &[]).await?;
        self.in_transaction.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub async fn commit(&self) -> Result<()> {
        self.execute("COMMIT", &[]).await?;
        self.in_transaction.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub async fn rollback(&self) -> Result<()> {
        self.execute("ROLLBACK", &[]).await?;
        self.in_transaction.store(false, Ordering::Relaxed);
        Ok(())
    }
}

impl Drop for CheckedConn {
    fn drop(&mut self) {
        let Some(conn) = self.conn.take() else { return };
        if !self.in_transaction.load(Ordering::Relaxed) {
            return; // dropped here: straight back to the pool
        }
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let _ = conn.execute("ROLLBACK", &[]).await;
                // `conn` dropped here: returned to the pool after the rollback
            });
        }
        // No runtime: best effort — the connection returns with the
        // transaction open. Documented, not an error path.
    }
}
```

- Flag semantics: set only after `BEGIN` succeeded; cleared only after
  `COMMIT` / `ROLLBACK` succeeded. A failed `COMMIT` leaves the flag set, so
  `Drop` still attempts `ROLLBACK` (rolling back nothing is a no-op warning
  on PostgreSQL, not an error).
- Only `begin()` is tracked: a raw `execute("BEGIN", &[])` is invisible to
  `Drop` — document that in the module docs.
- `client()`'s `expect` is a declared, unreachable invariant — the connection
  is `None` only inside `Drop`, and no `&self` method can run then. It is not
  a panic on a fallible path: converting it to `Result` would wire a
  never-triggerable error through every method. Consistent with §16 — the
  library never panics on anything a caller can provoke; §16's
  `with_safe_default_protocol_versions()` is fallible and is mapped, this one
  cannot fail at all.
- `Cargo.toml`: `postgres = ["dep:tokio-postgres", "dep:deadpool-postgres",
  "dep:tokio"]` (needed for `Handle::try_current` + `spawn`).
- Not offline-testable: an `Object` cannot be constructed without a pool that
  connected, so no unit test for this path — docs + review only.

### 15.2 SQLite (`pool/sqlite.rs`)

Same shape, synchronous `Drop`:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

pub struct CheckedConn {
    conn: r2d2::PooledConnection<SqliteConnectionManager>,
    in_transaction: AtomicBool,
}

impl CheckedConn {
    pub fn begin(&self) -> Result<()> {
        self.conn.execute_batch("BEGIN").map_err(query_err)?;
        self.in_transaction.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn commit(&self) -> Result<()> {
        self.conn.execute_batch("COMMIT").map_err(query_err)?;
        self.in_transaction.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn rollback(&self) -> Result<()> {
        self.conn.execute_batch("ROLLBACK").map_err(query_err)?;
        self.in_transaction.store(false, Ordering::Relaxed);
        Ok(())
    }
}

impl Drop for CheckedConn {
    fn drop(&mut self) {
        if self.in_transaction.load(Ordering::Relaxed) {
            // Best effort: the connection returns to the pool either way.
            let _ = self.conn.execute_batch("ROLLBACK");
        }
    }
}
```

`query` / `execute` are unchanged. `Pool::get()` constructs
`CheckedConn { conn: …, in_transaction: AtomicBool::new(false) }`.

Offline test (`:memory:` clamps the pool to one connection, so the same
database is reused): `get()` → `begin()` → insert → drop without commit →
a fresh `Pool::query` must not see the row. This test fails on round-1 code.

### 15.3 MySQL (`pool/mysql.rs`) — no code

Dropping a `Conn` returns it to the pool (`impl Drop for Conn`,
`mysql_async-0.34.2/src/conn/pool/mod.rs:363`), and the pool resets a
connection on check-out: `Pool::get_conn()` reads
`PoolOpts::reset_connection()` (`pool/mod.rs:241`), which defaults to `true`
(`opts/mod.rs:500`); `COM_RESET_CONNECTION` rolls back any open transaction.
`Pool::connect` builds `PoolOpts::default().with_constraints(…)` itself, so
DSN pool options are discarded and a `?reset_connection=false` cannot turn it
off. Net: a transaction left open by a dropped `CheckedConn` is rolled back
the next time that connection is checked out. No flag, no `Drop` impl.

### 15.4 Module docs (all three + `pool/mod.rs`)

Keep the "Transactions must run on a connection held from `Pool::get()`"
lead-in; replace the "Nothing rolls back automatically …" tail with the new
behavior, per backend:

- postgres: a `CheckedConn` dropped mid-transaction spawns a `ROLLBACK` on
  the current tokio runtime before the connection returns to the pool;
  without a runtime (or during shutdown) it cannot, and the connection
  returns with the transaction open. Only transactions started with `begin()`
  are tracked. End transactions explicitly.
- sqlite: a `CheckedConn` dropped mid-transaction runs a blocking `ROLLBACK`
  before the connection returns. Only `begin()` is tracked. End transactions
  explicitly.
- mysql: dropping returns the connection to the pool; the pool resets it on
  the next check-out (`reset_connection`, on by default), which rolls back a
  leftover transaction. End transactions explicitly anyway — until then the
  transaction holds locks.
- `pool/mod.rs`: one sentence pointing at the per-backend behavior, keeping
  "always end a transaction with `commit()` or `rollback()`" as the rule.

## 16. PostgreSQL TLS (item ④)

New optional feature `postgres-tls`. Hard constraint kept: `Pool::connect(dsn,
max_size)` (NoTls) is unchanged in signature, body and type, and **no
genericization is needed** — in deadpool-postgres 0.14.2 `Manager` is not
generic over TLS (the connector is boxed; `src/lib.rs:81-87`), so both
constructors build the same `Pool` type.

```toml
[features]
postgres = ["dep:tokio-postgres", "dep:deadpool-postgres", "dep:tokio"]
postgres-tls = ["postgres", "dep:tokio-postgres-rustls", "dep:rustls", "dep:webpki-roots"]

[dependencies.tokio-postgres-rustls]
version = "0.14"
optional = true
default-features = false

[dependencies.rustls]
version = "0.23"
optional = true
default-features = false
features = ["ring", "std", "tls12"]

[dependencies.webpki-roots]
version = "1"
optional = true
```

- Versions are the ones the workspace already resolves: rustls 0.23.45,
  webpki-roots 1.0.9, tokio-postgres 0.7.18. tokio-postgres-rustls 0.14.0
  (latest stable) requires `rustls ^0.23`, `tokio-postgres ^0.7`,
  `tokio-rustls ^0.26` — all satisfied. `default-features = false` on
  tokio-postgres-rustls selects no provider feature; its production code has
  no provider gates (checked 0.14.0: every `cfg(feature = "aws-lc-rs" |
  "ring")` sits inside `mod tests`).
- rustls keeps `default-features = false` so a bee_orm-only build does not
  drag `aws-lc-rs` in (C toolchain build). `ring` is the provider, `tls12`
  keeps TLS 1.2 servers reachable.

```rust
// pool/postgres.rs, in `impl Pool`:
#[cfg(feature = "postgres-tls")]
pub fn connect_tls(dsn: &str, max_size: u32) -> Result<Self> {
    let pg: tokio_postgres::Config = dsn.parse().map_err(conn_err)?;
    let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_client_config()?);
    let manager = Manager::from_config(pg, tls, ManagerConfig::default());
    let inner = DeadpoolPool::builder(manager)
        .max_size(max_size.max(1) as usize)
        .timeouts(/* timeouts + runtime as in §13.1 */)
        .build()
        .map_err(conn_err)?;
    Ok(Self { inner })
}

/// Bundled Mozilla roots (webpki-roots). The crypto provider is named
/// explicitly on purpose: workspace builds enable both rustls providers
/// (aws-lc-rs through other crates, ring here) and `ClientConfig::builder()`
/// panics when the process-level default is ambiguous.
/// `MakeRustlsConnect::with_webpki_roots()` has the same hazard — it
/// documents that it uses the process default.
#[cfg(feature = "postgres-tls")]
fn rustls_client_config() -> Result<rustls::ClientConfig> {
    let provider = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(conn_err)?
        .with_root_certificates(rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
        .with_no_client_auth();
    Ok(config)
}
```

- `with_safe_default_protocol_versions()` is the one fallible step — mapped,
  not `expect`ed, so the library never panics.
- TLS follows the DSN's `sslmode` (tokio-postgres default: `prefer`);
  `connect_tls` with `sslmode=disable` performs no TLS — the DSN is not
  overridden. Document that.
- Module docs: one line — TLS needs the `postgres-tls` feature and uses
  bundled roots only (no custom CAs / client certs; out of scope per §12).
- Offline test (in `src/`, `#[cfg(feature = "postgres-tls")]`): build the
  config and assert `webpki_roots::TLS_SERVER_ROOTS` is non-empty.
- CI: new step in the `check` job after `Build`:
  `cargo build -p bee_orm --features postgres-tls`.

## 17. `Pool::status()` (item ⑤)

Thin passthroughs, native return types, no invented abstraction:

```rust
// postgres.rs
pub fn status(&self) -> deadpool_postgres::Status { self.inner.status() }

// sqlite.rs
pub fn status(&self) -> r2d2::State { self.inner.state() }
```

- `deadpool::managed::Status { max_size, size, available, waiting }`
  (`deadpool-0.13.1/src/lib.rs:44-56`; `Pool::status` at
  `managed/pool.rs:354`); `r2d2::State { connections, idle_connections }`
  (`r2d2-0.8.10/src/lib.rs:574`).
- Named `status()` on both even though r2d2's native accessor is `state()`:
  callers should not have to remember two names; return types stay native.
- mysql: nothing — `mysql_async::Pool` has no stats API (checked 0.34.2
  `conn/pool/mod.rs`). Module docs get one line so the absence is deliberate.
- Offline tests: a fresh pg pool reports `size == 0` / `max_size == n`
  (connections open lazily); a sqlite `:memory:` pool reports
  `connections >= 1` after a `get()`.

## 18. Derive-macro rulings (reviewer follow-ups)

### 18.1 Raw identifiers — rule

Whenever the macro derives a **SQL identifier** from an `Ident`, it must unraw
it first (`syn::ext::IdentExt::unraw`) and only then stringify. The rule
applies to every derivation route (`lib.rs` line numbers as of round 2):

- default column name (`lib.rs:164`) — `struct Foo { r#type: String, … }` maps
  to column `type`;
- implicit-primary-key comparison against `id` (`lib.rs:195`) — a field
  written `r#id` counts as the name `id`, hence the implicit pk;
- default table name (`lib.rs:256`) — `struct r#User` maps to table `users`,
  not `r#users`; `unraw` before lowercasing.

Explicit `#[bee(column = "…")]` literals are unaffected, and generated code for
non-raw fields stays byte-identical (unraw is identity for non-raw idents) —
rule and table extension both verified so. Not covered by the rule: `lib.rs:186`
stringifies the raw field names only to build the "multiple `#[bee(pk)]`
fields" **message**; keeping `r#` there is correct (it quotes the source as
written) and never reaches SQL. Implemented by `coder-macro`.

### 18.2 `#[bee(auto)]` with no resolvable pk — precedence

Ruling: **"no primary key" wins.** pk resolution runs first and `auto`
validation is conditional on a resolvable pk, so `#[bee(auto)]` on a non-pk
field when no pk exists reports only
`bee_orm: no primary key: mark a field #[bee(pk)] or name it \`id\``. Once a
pk exists, `#[bee(auto)]` on any other field reports
`bee_orm: #[bee(auto)] is only valid on the primary key field`.
Rationale: the pk must be designated before "which field may be auto" is even
decidable. This is the shipped round-1 behavior, so it is a documented ruling,
not a code change.

### 18.3 Duplicate `#[bee(table = …)]` — ruling

Ruling: **compile error**, not last-wins. A second `table` key (same attribute
list or repeated attributes) reports
`bee_orm: duplicate #[bee(table)] attribute`, spanned at the second key,
combined with other accumulated errors. Rationale: a duplicate is never
intentional, silent last-wins hides a copy-paste bug, and it matches the
frozen "conflicts → compile_error!" policy. Implemented by `coder-macro`.

Addition to the §8.2 error table:

| case | message |
| --- | --- |
| two `table` keys | `bee_orm: duplicate #[bee(table)] attribute` |

Implementation notes (`coder-macro`): the second key is pushed into the
accumulated errors so it combines with everything else (no short-circuit);
`meta.error` puts the span on the second key. Verified by two macro unit tests
(`#[bee(table = "a", table = "b")]` and the two-attribute form) plus two
trybuild UI cases in `crates/bee_orm_macro/tests/ui/`:
`duplicate_table.rs` (single error, span on the second key) and
`duplicate_table_combined.rs` (duplicate + unknown keys, proving the combine).
This adds `trybuild` as a dev-dependency of `bee_orm_macro` (plus a
`bee_orm` path dev-dep, needed because the UI cases must link the real `Model`
trait) and new entries in the shared `Cargo.lock` — generated file, no owner.

## 19. Round-2 acceptance (additions to §10)

Run from `/home/wwwroot/bee-rust`, same gate as §10:

```bash
cargo build -p bee_orm --features postgres-tls     # new feature builds
cargo build -p bee_orm --features postgres         # NoTls path still builds
cargo test -p bee_orm --features sqlite            # drop-rollback + status tests
cargo test --workspace --all-features              # now also compiles the TLS stack
```

The three new direct dependencies must appear only under `postgres-tls`:
`postgres` alone pulls none of rustls / webpki-roots / tokio-postgres-rustls.

## 20. Round-2 decision log

- pg wait/create timeouts hardcoded (30 s / 10 s): the pinned default is
  all-`None` (wait forever); a `PoolConfig` parameter is the upgrade path,
  noted in-code with `ponytail:`.
- r2d2 needs no timeout code: `connection_timeout` defaults to 30 s, `get()`
  honors it, and its bare timeout message is already distinguishable.
- mysql gets a `tokio::time::timeout` wrapper because the driver has no
  acquire timeout; the timed-out future's `Drop` unqueues the waiter.
- Timeouts are mapped by variant (`PoolError::Timeout(TimeoutType::Wait)`),
  not by string matching; r2d2's error has no matchable variant, so its
  already-distinct text stands.
- pg statements run through `prepare_cached` + `Statement`: the cache is per
  connection and unbounded — documented rather than capped (a cap would be a
  new policy knob nobody asked for).
- Drop auto-rollback exists on sqlite and postgres only, because only they
  return a dirty connection to a non-resetting pool; mysql's pool resets on
  check-out (`reset_connection` default true, DSN cannot disable it here).
- pg `Drop` rolls back via `Handle::try_current()` + `spawn` because `Drop`
  is sync while `ROLLBACK` is async; without a runtime the connection returns
  dirty — documented best-effort, never silent.
- Transaction tracking uses `AtomicBool` + `&self` methods (not `Cell`, not
  `&mut self`): keeps `CheckedConn: Send + Sync` and every round-1 call site
  compiling.
- No genericization for TLS: deadpool-postgres 0.14.2's `Manager` is not
  TLS-generic, so `connect_tls` returns the same `Pool` — source
  compatibility holds structurally.
- rustls provider is explicit (`builder_with_provider(ring)`): both providers
  are enabled in workspace builds and `ClientConfig::builder()` would panic at
  runtime; `MakeRustlsConnect::with_webpki_roots()` carries the same hazard.
- rustls declared `default-features = false` + `ring` so a bee_orm-only build
  never compiles `aws-lc-rs`.
- `status()` on both pools with native types: one name for callers, no
  invented abstraction; mysql omits it with a doc note.
- Duplicate `#[bee(table = …)]` is a compile error (duplicates are always
  authoring bugs); `#[bee(auto)]`-without-pk precedence documented as shipped
  (pk errors first).
- Both pg constructors set `.runtime(Runtime::Tokio1)` because deadpool 0.13
  rejects non-empty `Timeouts` without a runtime value — a `build()`-time
  error, invisible to `cargo check` (caught by `coder-orm` during
  implementation, §13.1).

---

# Round 3 (2026-10-05): ORM conveniences + CI real-database tests

## 21. Scope, ownership, item → section map

| Item | Subject | Section | Writer |
| --- | --- | --- | --- |
| 1 | timestamps `#[bee(auto_now_add)]` / `#[bee(auto_now)]` | §23 | coder-orm (trait) + coder-macro (attrs) |
| 2 | soft delete `#[bee(soft_delete)]` + QuerySet filter + escape hatches | §24 | coder-orm (trait/QuerySet) + coder-macro (attr) |
| 3 | lifecycle hooks | §25 | coder-orm (trait) + coder-macro (grammar/codegen) |
| 4 | `Model::insert_many` | §26 | coder-orm |
| 5 | aggregates `sum` / `avg` / `min` / `max` | §27 | coder-orm |
| 6 | macro test-module split, before new macro code | §28.1 | coder-macro |
| 7 | interaction matrix | §29 | this document |
| — | CI real-database integration tests | §30 | tester (parallel stream, starts immediately) |

Write ownership: unchanged from §12, with one round-3 addition — **tester also
owns `.github/workflows/ci.yml` this round** (the round-2 `postgres-tls` build
step is already in place; coder-orm has no further ci.yml changes planned).
Item 6 is coder-macro's *first* task, before any new macro logic. The §30
stream is independent of the feature work and must not wait for coder-orm or
coder-macro.

Not in scope (deliberate, not forgotten): `group_by` / `having`; a
`deleted_at`-style timestamp soft delete; timestamp/soft-delete injection into
`QuerySet::update`; hooks on QuerySet or `insert_many`; transactions inside
`insert_many`; a TLS-handshake integration test (reason in §30); migrations and
relations (round 4, on the stabilized round-3 metadata surface).

## 22. Model trait additions (§5 revision — additive only)

The six required methods and the three async methods keep their signatures;
for models that use none of the round-3 attributes, every generated SQL
statement stays byte-for-byte what it is today. All additions have defaults:

```rust
#[async_trait]
pub trait Model: Send + Sync + 'static {
    // ... existing six required methods, unchanged ...

    /// Columns written with "now" (unix seconds) on `insert` only.
    fn auto_now_add_columns() -> &'static [&'static str] { &[] }

    /// Columns written with "now" (unix seconds) on `insert` and `update`.
    fn auto_now_columns() -> &'static [&'static str] { &[] }

    /// The soft-delete flag column, when the model has one.
    fn soft_delete_column() -> Option<&'static str> { None }

    async fn before_insert(&self) -> Result<()> { Ok(()) }
    async fn after_insert(&self) -> Result<()> { Ok(()) }
    async fn before_update(&self) -> Result<()> { Ok(()) }
    async fn after_update(&self) -> Result<()> { Ok(()) }
    async fn before_delete(&self) -> Result<()> { Ok(()) }
    async fn after_delete(&self) -> Result<()> { Ok(()) }

    async fn insert<D: Db + ?Sized>(&self, db: &D) -> Result<u64> {
        self.before_insert().await?;
        let values = insert_row_values(self, batch_now::<Self>()?);
        if values.is_empty() {
            return Err(no_columns("insert", Self::table_name()));
        }
        // ... the existing SQL build and execute, unchanged ...
        let affected = db.execute(&sql, &params).await?;
        self.after_insert().await?;
        Ok(affected)
    }
    // update(): before_update -> update_row_values(update_values + auto_now)
    //           -> existing SQL -> after_update.
    // delete(): before_delete -> soft UPDATE or DELETE (§24/§25) -> after_delete.
    // hard_delete(): before_delete -> plain DELETE -> after_delete.
    // insert_many(): §26.
}
```

Module-private helpers in `model.rs`:

```rust
/// Unix seconds. The only error path is a clock before 1970.
fn unix_now() -> Result<i64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|_| OrmError::QueryError("system clock is set before the unix epoch".into()))
}

/// One "now" per statement when the model needs it, 0 otherwise.
fn batch_now<M: Model + ?Sized>() -> Result<i64> {
    if M::auto_now_add_columns().is_empty() && M::auto_now_columns().is_empty() {
        Ok(0)
    } else {
        unix_now()
    }
}

fn insert_row_values<M: Model + ?Sized>(model: &M, now: i64) -> Vec<(&'static str, Value)> {
    let mut values = model.insert_values();
    values.extend(M::auto_now_add_columns().iter().map(|c| (*c, Value::Int(now))));
    values.extend(M::auto_now_columns().iter().map(|c| (*c, Value::Int(now))));
    values
}

fn update_row_values<M: Model + ?Sized>(model: &M, now: i64) -> Vec<(&'static str, Value)> {
    let mut values = model.update_values();
    values.extend(M::auto_now_columns().iter().map(|c| (*c, Value::Int(now))));
    values
}
```

Why declarative column lists instead of folding "now" into `insert_values`:
`insert_values` is sync and returns a bare `Vec` (frozen in §5/§8.3), while
`unix_now` is fallible; `insert`/`update` already return `Result`, so the
fallible part lives where a `?` exists. The lists are also the whole opt-in
mechanism for hand-written impls — no derive needed to get timestamps.

`delete` and the new `hard_delete`:

```rust
async fn delete<D: Db + ?Sized>(&self, db: &D) -> Result<u64> {
    self.before_delete().await?;
    let affected = match Self::soft_delete_column() {
        Some(column) => {
            let sql = format!(
                "UPDATE {} SET {column} = ? WHERE {} = ? AND {column} = ?",
                Self::table_name(),
                Self::pk_column()
            );
            db.execute(&sql, &[Value::Bool(true), self.pk_value(), Value::Bool(false)]).await?
        }
        None => {
            let sql =
                format!("DELETE FROM {} WHERE {} = ?", Self::table_name(), Self::pk_column());
            db.execute(&sql, &[self.pk_value()]).await?
        }
    };
    self.after_delete().await?;
    Ok(affected)
}

/// Always a real `DELETE`, soft flag ignored; delete hooks run.
async fn hard_delete<D: Db + ?Sized>(&self, db: &D) -> Result<u64> { /* before/after_delete around DELETE */ }
```

Deleting an already-deleted row returns 0 (the `AND {column} = ?` FALSE term)
— idempotent on purpose.

## 23. Timestamps (item 1)

- Attribute names are Beego's: `#[bee(auto_now_add)]` (set on insert only) and
  `#[bee(auto_now)]` (set on insert and refreshed on update); combinable with
  `column = "..."`.
- Storage: unix **seconds as `i64`** via `Value::Int` (fits `INTEGER`/`BIGINT`
  on all three backends; the existing Int decode covers read-back). The field
  type must decode from an integer JSON number — `i64` or `Option<i64>`.
  Derives cannot inspect field types, so a wrong type fails at `from_row`
  decode time, not at compile time (documented, not compile-checked).
- The macro excludes timestamp fields from generated
  `insert_values`/`update_values`; the trait injects the value. The struct's
  own value for those fields is never written — after an insert the in-memory
  struct is stale until re-read (Beego semantics; module docs say so).
- One `now` per statement; `insert_many` computes one `now` for the whole
  batch (§26).
- A model whose only writable column is a timestamp works: the `no_columns`
  check runs after injection.
- Clock before the epoch → `OrmError::QueryError`; no new enum variant.

## 24. Soft delete (item 2)

- `#[bee(soft_delete)]` marks exactly one field (a plain flag column; type
  must decode as `bool` — stored as 0/1 per backend). It stays an ordinary
  writable column in `insert_values`/`update_values` (users seed and restore
  by writing it).
- `T::soft_delete_column()` drives both the trait default and the QuerySet
  filter; hand-written impls can override it.
- Model level: `delete()` becomes the soft UPDATE shown in §22;
  `hard_delete()` is the real DELETE. Instance `update()` is pk-addressed and
  does **not** consult the flag (updating a soft-deleted row works — that is
  also the restore path).
- QuerySet level: two new private fields —
  `soft_filter: Option<(&'static str, Value)>` (from
  `T::soft_delete_column()`, value `Value::Bool(false)`, set in `new()`) and
  `include_deleted: bool`. All execution paths (`to_sql`, `all`, `one`,
  `count`, `exists`, `update`, `delete`, aggregates) build from:

```rust
/// "" or " WHERE flag = ? AND a AND b" — the flag term first while active.
fn active_where_sql(&self) -> String;
/// The flag parameter first (it is the first `?`), then the user params.
fn active_params(&self) -> Vec<Value>;
```

- Escape hatches: `QuerySet::with_deleted()` (sets `include_deleted`) and
  `QuerySet::hard_delete()` (plain `DELETE` with the active WHERE — so
  `with_deleted().hard_delete()` purges everything matching the filters).
  `QuerySet::delete()` on a soft model is
  `UPDATE t SET flag = ? WHERE flag = ? AND …` (set-params before where-params,
  mirroring `update`). Neither injects timestamps (the explicit-sets rule
  below).
- No `restore()` helper: restore = an explicit update, e.g.
  `User::query().with_deleted().filter_eq("id", id)?.update(db, &[("deleted", Value::Bool(false))])`.
- **NULL caveat, documented on the attribute and in module docs:** the
  active filter is `flag = ?` with `false`, and SQL `NULL = false` is NULL, so
  pre-existing rows with NULL in the flag column are invisible to default
  queries. Schema migrations must add the column `DEFAULT 0 NOT NULL` (or
  backfill).
- `Value::Bool` maps per backend: sqlite `ValueRef::Integer(0|1)`
  (`sqlite.rs:167`), mysql `Int(0|1)` (`mysql.rs:138`), pg `[u8]` bool
  (`postgres.rs:285-287`) — verified round-3.

## 25. Hooks (item 3)

One decision, stated: **hooks are `Model` trait methods with empty defaults;
the derive overrides exactly the hooks listed in a struct-level
`#[bee(hooks(...))]` attribute, forwarding each to a same-named inherent async
method the user writes.** Derive users never write `impl Model`; hand-written
impls override the trait methods directly (no attribute needed).

```rust
// user writes:
#[bee(hooks(before_insert, after_delete))]
struct Order { /* … */ }

impl Order {
    async fn before_insert(&self) -> bee_orm::Result<()> { /* … */ Ok(()) }
    async fn after_delete(&self) -> bee_orm::Result<()> { /* … */ Ok(()) }
}
```

Generated (only for listed hooks):

```rust
#[bee_orm::__private::async_trait]   // an impl overriding async trait methods must carry it
impl bee_orm::Model for Order {
    // … the usual methods …
    async fn before_insert(&self) -> bee_orm::Result<()> {
        Self::before_insert(self).await // inherent method wins resolution — NOT recursion
    }
    async fn after_delete(&self) -> bee_orm::Result<()> {
        Self::after_delete(self).await
    }
}
```

- `Self::hook(self)` resolves to the inherent method (the same verified rule as
  the inherent `table_name()` in §8.3). Round-3 design scratch-verified with
  rustc + async-trait 0.1.x (fresh 0.1.92; the mechanism is version-independent
  within 0.1): the forwarder reaches the inherent method (the trait-default
  recursion hazard does not fire), a hook-less impl stays unannotated and
  compiles, and the `where Self: Sized` async associated fn is callable.
- Required signature: `async fn NAME(&self) -> bee_orm::Result<()>`; any
  visibility (the generated impl sits in the defining module). A missing or
  mismatched method is a normal rustc error at the generated call — same
  policy as the `ignore`-field `Default` requirement in §8.2.
- Order and error semantics: `before_*` first — its `Err` aborts before any
  SQL runs. `after_*` runs after the SQL succeeded — its `Err` is returned
  even though the write is committed (this layer has no transaction context;
  documented on the trait, not silent).
- Hooks do not run for QuerySet operations or `insert_many` (table-level/batch
  APIs carry no instance lifecycle). `hard_delete` fires the delete hooks.
- `__private` re-export (§28.3) exists so the derive can name
  `async_trait` without bee_orm making it a new public API.

## 26. `insert_many` (item 4)

```rust
/// Bulk `INSERT`; one statement per chunk of rows.
async fn insert_many<D: Db + ?Sized>(db: &D, models: &[Self]) -> Result<u64>
where Self: Sized
```

Associated-fn form (`User::insert_many(&pool, &rows)`): a `&self` method would
need a receiver row, and the batch API has none.

```rust
{
    if models.is_empty() { return Ok(0); }
    let now = batch_now::<Self>()?;              // one "now" for the whole batch
    let mut rows = Vec::with_capacity(models.len());
    for model in models { rows.push(insert_row_values(model, now)); }
    let columns: Vec<&str> = rows[0].iter().map(|(c, _)| *c).collect();
    if columns.is_empty() { return Err(no_columns("insert_many", Self::table_name())); }
    if columns.len() > 999 {
        return Err(OrmError::QueryError(format!(
            "insert_many: {} columns exceed the 999-parameter batch limit",
            columns.len()
        )));
    }
    // ponytail: 999 is sqlite's classic bound; pg/mysql allow 65535 —
    // conservative floor for every backend, one constant, no per-backend SQL.
    let chunk = 999 / columns.len();
    let mut affected = 0u64;
    for group in rows.chunks(chunk) {
        let tuple = format!("({})", vec!["?"; columns.len()].join(", "));
        let values_clause = vec![tuple.as_str(); group.len()].join(", ");
        let sql = format!(
            "INSERT INTO {} ({}) VALUES {values_clause}",
            Self::table_name(),
            columns.join(", ")
        );
        let mut params = Vec::with_capacity(columns.len() * group.len());
        for row in group { params.extend(row.iter().map(|(_, value)| value.clone())); }
        affected += db.execute(&sql, &params).await?;
    }
    Ok(affected)
}
```

- Chunked multi-row `VALUES` over a per-row loop: the per-row loop was the
  other honest option, rejected because batching is the point of the API —
  a loop is just N calls to `insert`. Uniform column sets are guaranteed by
  construction (every row is the same `Self`).
- Partial failure semantics: **there is no transaction** — the `Db` trait has
  no transaction API (§4/§7), and wrapping `Pool::get()` would make this
  backend-specific. The first failing chunk aborts with the backend's error;
  rows in earlier chunks are already committed. Documented on the method.
- Timestamps (§23) are injected per row with the one shared `now`; hooks do
  not run (§25); the soft-delete flag is an ordinary column.

## 27. Aggregates (item 5)

```rust
/// `SELECT SUM(field) AS value …`; `None` when no row matches (SQL NULL).
pub async fn sum<D: Db + ?Sized>(&self, db: &D, field: impl Into<String>) -> Result<Option<f64>>
pub async fn avg<D: Db + ?Sized>(&self, db: &D, field: impl Into<String>) -> Result<Option<f64>>
/// `MIN`/`MAX` are type-agnostic — the raw JSON value of the column.
pub async fn min<D: Db + ?Sized>(&self, db: &D, field: impl Into<String>) -> Result<Option<serde_json::Value>>
pub async fn max<D: Db + ?Sized>(&self, db: &D, field: impl Into<String>) -> Result<Option<serde_json::Value>>
```

- `field` passes `validate_field` (invalid → `OrmError::InvalidField`) and is
  then spliced as `SUM({field})`.
- All four honour the active WHERE (§24), so soft-deleted rows are excluded
  unless `with_deleted()`.
- Private helpers in `queryset.rs`:

```rust
fn aggregate_sql(&self, function: &str, field: &str) -> String
// "SELECT {function}({field}) AS value FROM {table}{active_where}"
fn json_to_f64(value: &serde_json::Value) -> Option<f64>
// Number -> as_f64; String -> parse (pg returns NUMERIC as a string)
```

  `json_to_f64` returning `None` on a non-numeric value is an
  `OrmError::QueryError` at the call site ("sum: non-numeric result"); a
  missing row or `value` column is `"…: unexpected result shape"` (the §7
  `count` precedent). A JSON null (`SQL NULL`) is `Ok(None)`, not an error.
- `min`/`max` return `serde_json::Value` because text/date columns are
  legitimate; `serde_json::Value` is already the public `Row` vocabulary
  (§7's `count` reads through it), so this is not new surface.
- `sum`/`avg` as `f64`: AVG is inherently fractional, and SQL SUM exceeds
  f64 precision only past 2^53 (documented ceiling).
- `group_by`/`having`: **not built** (YAGNI — no consumer on the roadmap;
  filters + `order_by` cover the current needs). Revisit with a real use case.

## 28. Macro changes (round-3 macro side)

### 28.1 Split first (item 6)

`crates/bee_orm_macro/src/lib.rs` is 492 lines; the whole
`#[cfg(test)] mod tests { … }` block (lines 333-492 today) moves to
`crates/bee_orm_macro/src/tests.rs`, declared in `lib.rs` as
`#[cfg(test)] mod tests;`. This lands **before** any round-3 macro logic.
Projected sizes: `lib.rs` ≈ 340 + ≈120 new ≈ 460; `tests.rs` ≈ 160 + new cases
— both under 500.

### 28.2 Grammar additions

Struct level: `hooks(before_insert, after_insert, before_update, after_update,
before_delete, after_delete)` — nested list (syn `meta.parse_nested_meta`),
any subset, duplicates deduplicated silently (idempotent config, no error
row). Field level: `auto_now_add`, `auto_now`, `soft_delete` — flags,
combinable with `column`.

Revised §8.2 rows (messages change):

| condition | message |
| --- | --- |
| unknown struct key | `bee_orm: unknown bee attribute for a struct (expected table, hooks)` |
| unknown field key | `bee_orm: unknown bee attribute for a field (expected column, pk, auto, ignore, auto_now_add, auto_now, soft_delete)` |
| unknown hook name | `` bee_orm: unknown hook `x` (expected before_insert, after_insert, before_update, after_update, before_delete, after_delete) `` |

New-compile-error additions to §8.2:

| condition | message |
| --- | --- |
| `auto_now_add` + `auto_now` | `bee_orm: #[bee(auto_now_add)] and #[bee(auto_now)] on the same field` |
| timestamp + `ignore` | `bee_orm: #[bee(auto_now_add)] is not valid on an ignored field` (same with `auto_now`) |
| timestamp + `auto` | `bee_orm: #[bee(auto_now_add)] is not valid on a database-assigned (auto) field` (same with `auto_now`) |
| two `soft_delete` fields | `bee_orm: multiple #[bee(soft_delete)] fields` |
| `soft_delete` + `ignore` | `bee_orm: #[bee(soft_delete)] is not valid on an ignored field` |
| `soft_delete` on the pk field | `bee_orm: #[bee(soft_delete)] is not valid on the primary key field` |

No rule against timestamps on the pk (a timestamp primary key is legitimate);
`auto` + timestamp is already caught by the auto row, since `auto` is pk-only.

### 28.3 Generated code additions — only when the attributes are used

```rust
fn auto_now_add_columns() -> &'static [&'static str] { &["created_at"] } // omit when none
fn auto_now_columns() -> &'static [&'static str] { &["updated_at"] }     // omit when none
fn soft_delete_column() -> Option<&'static str> { Some("deleted") }      // omit when unmarked
async fn before_insert(&self) -> bee_orm::Result<()> { Self::before_insert(self).await } // per listed hook
```

- Timestamp fields are excluded from generated `insert_values` **and**
  `update_values` (the trait injects them); `auto_now_add` is also excluded
  from `update_values` (update never touches it). `soft_delete` stays in both.
  `from_row` is unchanged (timestamps decode as ints, the flag as bool).
- An impl carrying hook forwarders is emitted as
  `#[bee_orm::__private::async_trait] impl bee_orm::Model for X { … }`;
  hook-less structs keep the unannotated impl byte-for-byte.
- New hidden re-export in `crates/bee_orm/src/lib.rs`:
  `#[doc(hidden)] pub mod __private { pub use async_trait::async_trait; }`.

### 28.4 Regression gate

The five existing expansion tests in `src/tests.rs` must pass **unmodified**:
non-round-3 structs expand byte-identically (the "omit when unused" rule is
the whole point). Round-3 additions to those tests: one expansion per new
attribute, the new error rows, and (recommended) a trybuild pass-case with
`hooks(...)` — it pins the forwarder resolution in a real crate — plus a UI
fail-case (e.g. `soft_delete` on the pk).

## 29. Interaction matrix (item 7)

| # | Interaction | Behavior |
| --- | --- | --- |
| 1 | soft-delete × insert | flag is an ordinary writable column; inserting `deleted = true` is allowed (seeding) |
| 2 | soft-delete × Model::update | pk-addressed, no flag condition — updating a deleted row succeeds (also the restore path) |
| 3 | soft-delete × Model::delete | soft UPDATE, `AND flag = false` — second delete returns 0 |
| 4 | soft-delete × Model::hard_delete | real DELETE; delete hooks run |
| 5 | soft-delete × QuerySet all/one/count/exists | implicit `flag = ?(false)` filter first; `with_deleted()` disables |
| 6 | soft-delete × QuerySet::update | active filter applies; **no** timestamp injection (explicit sets) |
| 7 | soft-delete × QuerySet::delete / hard_delete | soft UPDATE / plain DELETE, both under the active filter |
| 8 | soft-delete × aggregates | active filter applies; `with_deleted()` aggregates over deleted too |
| 9 | soft-delete × insert_many | flag per row, ordinary column |
| 10 | soft-delete × NULL flag | `NULL = false` is NULL → NULL-flagged rows invisible to default queries (§24 caveat) |
| 11 | timestamps × insert | `auto_now_add` + `auto_now` columns ← one `now` |
| 12 | timestamps × Model::update | `auto_now` refreshed; `auto_now_add` untouched |
| 13 | timestamps × Model::delete (soft) | flag only — no timestamp refresh (deleting ≠ updating) |
| 14 | timestamps × QuerySet::update / delete / hard_delete | no injection |
| 15 | timestamps × insert_many | one `now` per batch, injected per row |
| 16 | timestamps × reads | ordinary int column decode; in-memory struct stale after insert until re-read |
| 17 | hooks × insert / update | before → SQL → after; before-error aborts pre-SQL; after-error propagates post-write |
| 18 | hooks × delete / hard_delete | same, wrapping soft UPDATE or DELETE |
| 19 | hooks × soft delete order | before_delete → flag UPDATE → after_delete |
| 20 | hooks × timestamps order | before hook runs first; `now` is computed after it (hooks take `&self`, so they cannot see injected values) |
| 21 | hooks × QuerySet / insert_many | do not run (table-level/batch APIs) |
| 22 | hooks × hand-written impls | override the six trait methods directly; attribute path is derive-only |
| 23 | multiple timestamps | several `auto_now_add`/`auto_now` fields allowed, all get the same `now` |
| 24 | timestamp field type | must decode as int (`i64` / `Option<i64>`); wrong type fails at `from_row`, not at compile time |

## 30. CI real-database integration tests (tester, parallel stream)

Starts immediately; does not wait for coder-orm or coder-macro. Tester owns
`.github/workflows/ci.yml` this round. Recognition due: this stream turns
round-2's review-only claims into measurements.

Job shape — a **new, independent job** `integration` in `ci.yml`, same triggers
as the workflow (push / pull_request), not gating `check`; actions pinned by
the same SHA convention as the existing jobs:

```yaml
  integration:
    name: Integration (postgres + mysql)
    runs-on: ubuntu-latest
    timeout-minutes: 20
    services:
      postgres:
        image: postgres:16-alpine
        env: { POSTGRES_PASSWORD: postgres }
        ports: ["5432:5432"]
        options: >-
          --health-cmd "pg_isready -U postgres"
          --health-interval 5s --health-timeout 5s --health-retries 10
      mysql:
        image: mysql:8
        env: { MYSQL_ROOT_PASSWORD: root, MYSQL_DATABASE: bee_test }
        ports: ["3306:3306"]
        options: >-
          --health-cmd "mysqladmin ping -h 127.0.0.1 -proot"
          --health-interval 5s --health-timeout 5s --health-retries 10
    env:
      BEE_ORM_PG_DSN: postgres://postgres:postgres@127.0.0.1:5432/postgres
      BEE_ORM_MYSQL_DSN: mysql://root:root@127.0.0.1:3306/bee_test
    steps: # checkout (same SHA pin) + toolchain setup mirroring `check`
      - run: cargo test -p bee_orm --all-features
```

Gating: env-var skip, local `cargo test` stays green. `tests/common/mod.rs`
gets `pub fn dsn(var: &str) -> Option<String>` — unset/empty → the test prints
`"… not set; skipping"` and returns Ok. Files: `tests/integration_pg.rs`
(`#![cfg(feature = "postgres")]`) and `tests/integration_mysql.rs`
(`#![cfg(feature = "mysql")]`), each with `mod common;`.

PostgreSQL:
1. Round-trip: DDL, Model insert (auto pk), QuerySet read/filter/order/limit,
   Model update/delete, value fidelity (int, float, text, null, bool, bytes
   `BYTEA`).
2. Drop auto-rollback (§15.1): pool `max_size = 1`; `get`, `begin`, insert,
   drop without commit; sleep ≈100 ms (the `Drop` spawns the ROLLBACK task —
   documented race note); re-`get` and count → row absent.
3. Pool exhaustion (§13.1): `max_size = 1`; hold one connection; the second
   `get()` → `OrmError::ConnectionError` containing `pool exhausted`.
   **Takes ~30 s by design** — no wall-clock assertion; the job timeout
   absorbs it.
4. Statement cache second use (§14): `max_size = 1`; the same SQL twice
   through `pool.query` → both succeed with equal results. Behavioral only:
   a per-client `StatementCache::size()` exists in the driver, but reaching a
   pooled client's cache is not part of our public API — do not assert on it.
5. TLS negative (inside `#[cfg(feature = "postgres-tls")]`):
   `connect_tls(…?sslmode=require…)` against the non-TLS container → the first
   `get()` errors — proves the rustls connector is engaged on the real wire.

MySQL:
1. Round-trip (same shape as pg).
2. Check-out reset (§15.3, measured for the first time): `max_size = 1`;
   `get`, `begin`, insert, drop mid-transaction; re-`get` → row absent.
3. Pool exhaustion (§13.3): `max_size = 1` → `ConnectionError` containing
   `pool exhausted` (~30 s).

sqlite's drop-rollback already has an offline test (§15.2) — nothing to add.

**TLS handshake (batch-5A amendment, §41 A2): built.** The original note here
said a real handshake was impossible because `connect_tls` bundles webpki
roots and has no injection point. A2 adds
`connect_tls_with(dsn, max_size, rustls::ClientConfig)`, closing that gap
without touching the bundled-roots path:

- New env-gated target `tests/integration_pg_tls.rs` (own var
  `BEE_ORM_PG_TLS_DSN`, `sslmode=require`). It builds a **test-only no-verify**
  `ServerCertVerifier` (`rustls::client::danger`; signature checks via the
  ring provider's `signature_verification_algorithms`) — tests-only code,
  never in the crate. If the pinned rustls gates `ConfigBuilder::dangerous()`
  behind `dangerous_configuration`, `[dev-dependencies] rustls` toggles it
  (already a dependency — not a new package).
- **Positive**: `connect_tls_with(…, no_verify_config)` → Ok, and
  `SELECT ssl FROM pg_stat_ssl WHERE pid = pg_backend_pid()` is true —
  measured on the real wire.
- **Negative A** (existing item 5, unchanged, runs before TLS is enabled
  below): `connect_tls` with `sslmode=require` against the TLS-less container
  → first `get()` errors.
- **Negative B** (new): after TLS is on, `connect_tls` (bundled webpki roots)
  → error — the self-signed cert is not trusted. Verification is enforced by
  the wrapper; injection is how a caller opts out.

CI addition — after the main `cargo test` step, one step turns TLS on for the
postgres service container (services live on the runner's docker daemon; no
new images, no new cert files in the repo):

```yaml
      - name: Enable TLS on postgres
        run: |
          openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
            -keyout /tmp/pg.key -out /tmp/pg.crt \
            -subj "/CN=127.0.0.1" -addext "subjectAltName=IP:127.0.0.1"
          cid=$(docker ps -q -f ancestor=postgres:16-alpine)
          docker exec "$cid" mkdir -p /var/lib/postgresql/tls
          docker cp /tmp/pg.crt "$cid:/var/lib/postgresql/tls/server.crt"
          docker cp /tmp/pg.key "$cid:/var/lib/postgresql/tls/server.key"
          docker exec "$cid" chown -R postgres:postgres /var/lib/postgresql/tls
          docker exec "$cid" chmod 600 /var/lib/postgresql/tls/server.key
          docker exec "$cid" psql -U postgres -c "ALTER SYSTEM SET ssl = on"
          docker exec "$cid" psql -U postgres -c "ALTER SYSTEM SET ssl_cert_file = '/var/lib/postgresql/tls/server.crt'"
          docker exec "$cid" psql -U postgres -c "ALTER SYSTEM SET ssl_key_file = '/var/lib/postgresql/tls/server.key'"
          docker exec "$cid" psql -U postgres -c "SELECT pg_reload_conf()"
      - env:
          BEE_ORM_PG_TLS_DSN: "postgres://postgres:postgres@127.0.0.1:5432/postgres?sslmode=require"
        run: cargo test -p bee_orm --all-features --test integration_pg_tls
```

`ssl*` are SIGHUP-context settings — `pg_reload_conf()` suffices (no
restart), and non-TLS clients keep working on the same port. Local
reproduction: run the same step against a `docker run`ed container (`-f
ancestor=postgres:16-alpine` finds it the same way).

**YAML defect fixed at implementation (pg16-verified).** `ALTER SYSTEM SET`
takes string literals — the two file-path settings must be quoted
(`ssl_cert_file = '/var/…'`); unquoted is `ERROR: syntax error at or near
"=/"` and would red the whole job under `bash -e`. `ssl = on` stays bare (a
keyword, not a path) — do not quote it. Tester verified on pg16 (`SHOW ssl`
= on, all three values effective, `openssl s_client -starttls postgres`
handshake shows the self-signed CN); `ci.yml` matches this block.

Local reproduction (same commands in the spec for reviewers):

```bash
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:16-alpine
docker run -d -p 3306:3306 -e MYSQL_ROOT_PASSWORD=root -e MYSQL_DATABASE=bee_test mysql:8
BEE_ORM_PG_DSN='postgres://postgres:postgres@127.0.0.1:5432/postgres' \
BEE_ORM_MYSQL_DSN='mysql://root:root@127.0.0.1:3306/bee_test' \
cargo test -p bee_orm --all-features
```

## 31. Round-3 acceptance (additions to §10/§19)

```bash
cargo fmt --all
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-features
cargo test -p bee_orm --all-features          # focused loop
cargo test -p bee_orm_macro
cargo build -p bee_orm --features postgres-tls   # round-2 step, still green
# CI stream locally (DSN-gated, §30):
BEE_ORM_PG_DSN=… BEE_ORM_MYSQL_DSN=… cargo test -p bee_orm --all-features
```

New functional tests (`tests/**`, tester — after coder signals): timestamps
(insert fills both, update refreshes `auto_now` only); soft delete (delete →
absent from all/one/count/exists, `with_deleted()` sees it, instance update
touches it, `hard_delete` removes); hooks (before/after fire in order, a
before-error aborts before any row change); `insert_many` (round-trip and a
chunking case — e.g. a 1-column model × 1000 rows crosses the 999 floor; the
`>999 columns` guard itself is review-only, it needs a 1000-column struct);
aggregates (fixture values, empty-set `None`, `InvalidField` rejection).

## 32. Round-3 decision log

- Timestamps ride declarative trait column lists (`auto_now_add_columns` /
  `auto_now_columns`) instead of folded into `insert_values`: the frozen
  `insert_values` is sync `Vec` (no `?`), and the lists double as the
  hand-written-impl opt-in.
- Unix seconds as `i64` (§1 constraint: zero new deps; `SystemTime` only),
  one `now` per statement/batch; wrong field types fail at decode, documented
  rather than compile-checked (derives cannot see types).
- Soft delete is a bool flag, not `deleted_at`: one semantic, `Value::Bool`
  maps cleanly on all three backends, and the NULL caveat is documented with
  the `DEFAULT 0 NOT NULL` migration note.
- QuerySet soft filter lives in an `Option` field + `include_deleted` toggle
  (not smuggled into `filters`) so `with_deleted()` is a flag flip, not a
  positional removal — and `to_sql()` stays byte-identical for non-soft
  models.
- `QuerySet::delete()` goes soft on soft models (consistent with the model),
  `hard_delete()` is the explicit escape; neither injects timestamps —
  QuerySet is the explicit-sets layer.
- Instance `delete()` on an already-deleted row returns 0 (idempotent), via
  the `AND flag = false` term.
- Hooks are Model trait methods + a `#[bee(hooks(...))]` forwarder to
  same-named inherent async methods: no supertrait (hand-written impls keep
  compiling), no method-name detection (impossible in a derive), one
  grammar row. Resolution to the inherent method is scratch-verified, and a
  trybuild pass-case pins it in-repo.
- `after_*` hook errors propagate with the write already committed — no
  transaction exists at this layer; documented instead of silently swallowed.
- `insert_many` is an associated fn (a batch has no receiver row), chunked at
  a conservative 999 parameters (sqlite's classic bound; pg/mysql allow more —
  one constant, uniform SQL), and is **not transactional**: partial failure is
  aborted-with-error, earlier chunks committed, documented.
- Aggregates: `f64` for sum/avg (AVG is fractional; 2^53 ceiling documented),
  raw JSON for min/max (text columns are legitimate); `group_by`/`having`
  deliberately not built (YAGNI).
- `__private::async_trait` re-export: the derive must annotate hook-bearing
  impls, but async-trait should not become new public API.
- Macro test module moves to `src/tests.rs` before new logic (lib.rs was at
  492/500; the file-limit rule outranks test locality).
- CI: separate `integration` job, env-gated skips keep local `cargo test`
  green, pool-exhaustion tests pay the real ~30 s, TLS is covered by the
  `sslmode=require` negative test — a self-signed handshake is impossible
  under the bundled-roots-only API.
- `QuerySet::params()` keeps its frozen `&[Value]` signature and returns the
  user parameters only; the soft-delete flag parameter is injected by every
  execution path via `active_params()` (flag first), while `to_sql()` does
  contain the flag placeholder. The asymmetry is deliberate: `to_sql()`
  reflects the SQL that actually runs, and a frozen borrow has nowhere to hold
  a synthesized parameter. A manual `to_sql()` + `params()` pairing on a soft
  model must prepend `Value::Bool(false)` — after `with_deleted()` the pair is
  self-consistent — documented at the `params()` doc, in `src/tests.rs`
  assertions, and in `pool/mod.rs`. No new public API for manual pairing
  (YAGNI: the execution methods are the path).
- pg read side gains a `bytea → Vec<u8>` decode (write side already maps
  `Value::Bytes → bytea`): without the branch a BYTEA column fell through the
  catch-all `String` decode and was silently nulled. Found by tester while
  round-3 was in flight, and locally verified against a real `postgres:16`
  container (`integration_pg` 5/5, including the `Vec<u8>` roundtrip at
  `integration_pg.rs:68`); the other undecoded types (numeric, date/time,
  array) keep the read helper's documented stance. `model.rs` line-260
  (insert_many param flattening) and a `src/tests.rs` deref were the round's
  two E0308 fixes; clippy needed zero corrections.
- Real-DB verification (tester's Part A against `mysql:8.0` / `postgres:16`,
  service-container parity with CI) surfaced a round-2 gap: mysql
  `CheckedConn::begin` sent `BEGIN` through the prepared protocol, where
  MySQL 8 answers `ERROR 1295` — mysql transactions were broken on a real
  server, having been compile/unit-verified only in round 2. Fixed by issuing
  the three transaction statements over the text protocol (`pool/mysql.rs`
  `query_drop` ×3, the 1295 reason in the method docs; §6.1 carries the
  amended ruling; `integration_mysql` 3/3 green against `mysql:8.0`,
  including the previously-red checkout-reset case). Same class as §13.1's
  `.runtime(Tokio1)`: a write-it-this-way-or-the-real-DB-breaks trap no
  compiler check surfaces — and the episode is the argument for the
  env-gated CI integration job: container-backed runs catch protocol-level
  breakage that unit tests structurally cannot.

## 33. Round-4 scope, ownership, file map

Round 4 = **migrations** (Beego RunSyncdb style) + **relations** (FK metadata,
belongs_to, has_many). Explicitly NOT a versioned migration system: no history
table, no up/down files, no DROP, no type changes — only
`CREATE TABLE IF NOT EXISTS` and `ALTER TABLE ADD COLUMN`. Builds on the
frozen round-3 surface. Docs stay untouched this round (the final docs sweep,
incl. "planned" wording and count refresh, is scheduled separately by the
lead).

Ownership (unchanged): coder-orm → `crates/bee_orm/**`; coder-macro →
`crates/bee_orm_macro/**`; tester → `crates/bee_orm/tests/**` (+ ci.yml if
needed — not expected this round).

Files touched:
- coder-orm: `src/model.rs` (metadata types + `columns()` default),
  `src/migrate.rs` (new), `src/rel.rs` (new), `src/db.rs` (`dialect()`),
  `src/queryset.rs` (`filter_in`), `src/lib.rs` (modules, re-exports, hoisted
  `MAX_BIND_PARAMS`).
- coder-macro: `src/lib.rs` — **split first** (file is at 493/500; §28.1
  precedent) — then grammar/codegen additions; `src/tests.rs`; `tests/ui/`.
- tester: `crates/bee_orm/tests/` (new sqlite e2e file; DSN-gated DDL cases in
  `integration_pg.rs` / `integration_mysql.rs`).

Hard constraints: **zero new dependencies** (migrate/rel are string building
over the existing `Db`/`QuerySet`); files <500 lines; the §18.1 unraw rule
applies to every new identifier emission; never-destructive DDL is a property
to verify, not just a promise.

## 34. Column metadata and type mapping (item: migrations)

**Design: spelling-based canonical mapping + per-field raw override** (the
hybrid of the brief's two options). The macro maps the field's *syntactic*
type spelling; an unmappable spelling is a `compile_error` pointing at the
field with the `#[bee(sql_type = "…")]` hint. Type aliases and custom types
therefore require the override — derives cannot resolve paths, documented.

Metadata types live in `src/model.rs` (next to `Model`, since `columns()` is a
`Model` method):

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dialect { Sqlite, Postgres, Mysql }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SqlType { Int, BigInt, Real, Double, Bool, Text, Blob, Raw(&'static str) }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefaultValue { Bool(bool), Int(i64), Raw(&'static str) }

#[derive(Debug, Clone, Copy)]
pub struct Reference {
    pub table: fn() -> &'static str,      // target Model's table_name
    pub pk_column: fn() -> &'static str,  // target Model's pk_column
}
// PartialEq is hand-written (compares the resolved names): comparing fn
// pointers trips rustc's unpredictable-function-pointer-comparisons lint, and
// name equality is the meaningful comparison anyway.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnDef {
    pub name: &'static str,
    pub sql: SqlType,
    pub nullable: bool,
    pub primary_key: bool,
    pub auto_increment: bool,          // #[bee(auto)]
    pub default: Option<DefaultValue>,
    pub references: Option<Reference>, // #[bee(fk = T)]
}

// Model addition, defaulted so hand-written impls keep compiling:
fn columns() -> &'static [ColumnDef] { &[] }
```

`SqlType` → dialect rendering:

| SqlType | sqlite | postgres | mysql |
|---|---|---|---|
| `Int` | INTEGER | INTEGER | INT |
| `BigInt` | INTEGER | BIGINT | BIGINT |
| `Real` | REAL | REAL | FLOAT |
| `Double` | REAL | DOUBLE PRECISION | DOUBLE |
| `Bool` | INTEGER | BOOLEAN | BOOLEAN |
| `Text` | TEXT | TEXT | TEXT |
| `Blob` | BLOB | BYTEA | BLOB |
| `Raw(s)` | `s` | `s` | `s` |

Spelling table (match on the last path segment; strip `Option<…>` first — its
presence sets `nullable`):

| Rust spelling | SqlType |
|---|---|
| `bool` | Bool |
| `i8` `i16` `i32` `u8` `u16` `u32` | Int |
| `i64` | BigInt |
| `f32` | Real |
| `f64` | Double |
| `String` | Text |
| `Vec<u8>` | Blob |
| anything else | compile_error + `sql_type` hint |

Deliberately unmapped: `u64`, `usize`, `isize`, `i128`/`u128`, aliases and
custom types — `u64`/`usize` are excluded from `Value` on purpose (round 1,
"not lossless"), and the rest have no round-trip path; the override is the
documented door. `#[bee(sql_type = "…")]` → `SqlType::Raw(s)`, verbatim on
every dialect; skips the spelling check entirely (any field type).
Option-ness still comes from the outer spelling. Documented limitations: one
string for all dialects (no per-dialect split), and JSON-typed model fields
are unsupported — `Value` has no serde_json conversions at all (round-5
candidate, not built now).

Derive-baked adjustments (self-contained metadata; see decision log — migrate
never consults the `auto_now*` / `soft_delete_column` lists):

- `auto_now_add` / `auto_now` fields: `sql = BigInt` **forced** (the brief's
  "timestamp columns are BIGINT" holds even if the field is spelled `i32`; a
  `sql_type` override wins for the type string only — the auto-write paths
  always bind `Value::Int`, so an override away from an integer type is
  unsupported and will fail in the driver), `nullable = false`,
  `default = Some(Int(0))`.
- `soft_delete` field: `nullable = false`, `default = Some(Bool(false))` —
  the §23 `DEFAULT 0 NOT NULL` mandate becomes structural.
- the pk field (`#[bee(pk)]` / `#[bee(auto)]` — the macro's existing round-1
  resolution, unchanged here): `primary_key = true`; with `auto`,
  `auto_increment = true` and the spelling must be an integer kind
  (`Int`/`BigInt`) unless a `sql_type` override is present — otherwise
  compile_error.

## 35. Migrations (item: migrations)

Module `bee_orm::migrate` — no feature gate (pure string building plus the
existing `Db`):

```rust
pub async fn create_table<M: Model, D: Db + ?Sized>(db: &D) -> Result<()>;
pub async fn add_missing_columns<M: Model, D: Db + ?Sized>(db: &D) -> Result<u64>;
pub async fn sync<M: Model, D: Db + ?Sized>(db: &D) -> Result<u64>; // create then add
```

**Dialect discovery** — `Db` gains a defaulted method (additive; mocks and
external impls keep compiling):

```rust
fn dialect(&self) -> Option<Dialect> { None }
```

the three pools return `Some(...)`; migrate functions error on `None`
(`OrmError::QueryError`, "this Db does not report a dialect"). Free functions
win over the brief's `Model::create_table(db)` example — migrations are an
operation over a `Db`, not instance lifecycle (decision log).

**DDL shape.** `CREATE TABLE IF NOT EXISTS <table> (<coldef>, …)`; columns in
`columns()` order; no identifier quoting (established rounds-1–3 convention —
names come from macro idents (unraw'd per §18.1) or your own literals;
documented, not changed here). Column clause: `<name> <type> [NOT NULL]
[DEFAULT <v>]`, plus pk handling:

- non-auto pk: `NOT NULL PRIMARY KEY` inline.
- auto pk — the one structural dialect special case:
  - sqlite: exactly `INTEGER PRIMARY KEY AUTOINCREMENT` (AUTOINCREMENT is
    only legal inline on an INTEGER pk; BigInt renders INTEGER on sqlite
    anyway).
  - postgres: `<INTEGER|BIGINT> GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY`
    — BY DEFAULT, not ALWAYS: explicit-id inserts keep working, mirroring
    sqlite/mysql semantics.
  - mysql: `<INT|BIGINT> NOT NULL AUTO_INCREMENT PRIMARY KEY`.
- defaults render per dialect: `Bool(b)` → `0` / `1` on sqlite and mysql,
  `false` / `true` on postgres; `Int(i)` → decimal; `Raw(s)` verbatim.
- `references` renders inline: `REFERENCES <table> (<pk_column>)` (the
  `Reference` fn pair supplies both names). Amendment (real-DB evidence,
  tester: `mysql:8.0.46` real server + sqlite through the pool): the inline
  clause is portable *intent*, and enforcement differs per backend —
  postgres enforces (a missing parent target fails CREATE loudly); mysql
  parses and **ignores** inline `REFERENCES` entirely (`SHOW CREATE TABLE`
  shows no FK; dangling parents create fine); sqlite **enforces** (a dangling
  CREATE succeeds, but the first dangling INSERT fails with `FOREIGN KEY
  constraint failed`). The pool issuing no PRAGMA is NOT the test: the
  bundled build compiles with `-DSQLITE_DEFAULT_FOREIGN_KEYS=1` (libsqlite3-sys
  `build.rs:123`), so enforcement is on by default. Enforcement parity is
  therefore a mysql-only gap — table-level `FOREIGN KEY` + `ADD CONSTRAINT`
  on the sync path is a round-5+ candidate in §40 (constraint naming; would
  fail `sync` at startup on databases with pre-existing orphan rows). A
  non-bundled sqlite build would need per-connection `PRAGMA foreign_keys =
  ON` (r2d2_sqlite `with_init`); bee_orm pins `bundled`, so not needed
  in-tree.

**`add_missing_columns`** — read existing columns, diff by name, then one
`ALTER TABLE <t> ADD COLUMN <clause>` per missing column (uniform: sqlite
allows only one column per ALTER anyway). Existing-column query per dialect,
row key uniformly `name` (alias explicitly):

- sqlite: `PRAGMA table_info(<table>)` (its first column is literally `name`).
- postgres: `SELECT column_name AS name FROM information_schema.columns
  WHERE table_schema = current_schema() AND table_name = ?`.
- mysql: `SELECT column_name AS name FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = ?`.

Missing table while running `add_missing_columns` alone: the column list
reads empty, every column is "missing", and the first ALTER fails with the
driver's no-such-table error — loud, acceptable (`sync` creates first).
Renames are not detected: a renamed field appears as a missing column → ADD
COLUMN; the old column is left alone (never destructive). NOT NULL columns on
a populated table: only ts/soft columns carry defaults; a *new* non-`Option`
column without a sql_type default fails loudly on sqlite/postgres (mysql
backfills) — documented guidance: make it `Option<T>` or give a `sql_type`
with `DEFAULT`. For fk (`references`) columns sqlite's rules are sharper
(probed through this exact path,
`orm_migrate_rel.rs::sqlite_fk_add_column_restrictions_are_data_checks`): on
a populated table the only addable shape is `Option<T>` (NULL default) — a
`REFERENCES` column carrying a non-NULL default is rejected (`Cannot add a
REFERENCES column with non-NULL default value`), and a non-nullable one
without a default fails the NOT NULL rule; both are *data* checks, not schema
checks, so empty tables accept either shape. No locking: boot-time
single-writer assumption, documented.

Empty `columns()` → `QueryError` naming the model (hand-written impls must
implement `columns()` to use migrations — documented on the trait method).

**`bee_cli migrate`: not wired** (decision log): bee_cli is clap-only and
cannot enumerate an application's models; auto-registration would need a
registry crate (new dependency — forbidden). Deferred, recorded here.
**Superseded in batch 5**: the generative design (§49) needs no registry and
no new dependency — the CLI scaffolds a user-side binary that calls
`migrate::sync` itself, so the CLI process never has to see the models.

## 36. Relations (item: relations)

Minimal complete set: FK metadata + `belongs_to` + `has_many` with IN-batched
loading. Read-only. **No joins** (the batched API covers list loading; a JOIN
builder is a round-5+ topic), **no m2m** (needs junction DDL + an attach/detach
write API — explicitly deferred to round 5), no has_one (expressible as
children + first), no eager-attach struct fields (no hidden non-column
navigation fields, no `Default`-bound attachment machinery — pure query API;
decision log).

**FK metadata (macro).** `#[bee(fk = Target)]` on a column field
(`user_id: i64`); value is a type path required to be a `Model` (the derive
emits a bound assertion; a non-Model target surfaces the standard trait-bound
error at the attribute span). Semantics: this column references `Target`'s
primary key. `Option<i64>` → nullable column, no check on NULL. Multiple FKs
to one target are allowed (discovery is first-declared; the `_via` variants
exist for the rest). Errors: fk on an `ignore` field, fk on the pk field.
CRUD is unaffected — fk is a plain writable column (`insert_values` /
`update_values` include it). Migrations render the `REFERENCES` clause inline.

**Runtime — `bee_orm::rel`** (all reads via `QuerySet`, so the child's and
parent's own soft-delete filter applies by default; `with_deleted()` stays
reachable through the query-builder variants):

```rust
pub fn fk_column_to<C: Model, P: Model>() -> Option<&'static str>; // first column referencing P
pub fn children_query<C: Model, P: Model>(parent: &P) -> Result<QuerySet<C>>;
pub fn children_query_via<C: Model, P: Model>(parent: &P, fk_column: &str) -> Result<QuerySet<C>>;
pub async fn children<C: Model, P: Model, D: Db + ?Sized>(db: &D, parent: &P) -> Result<Vec<C>>;
pub async fn children_via<C: Model, P: Model, D: Db + ?Sized>(db: &D, parent: &P, fk_column: &str) -> Result<Vec<C>>;
pub async fn children_for<C: Model, P: Model, D: Db + ?Sized>(db: &D, parents: &[P]) -> Result<Vec<Vec<C>>>;
pub async fn children_for_via<C: Model, P: Model, D: Db + ?Sized>(db: &D, parents: &[P], fk_column: &str) -> Result<Vec<Vec<C>>>;
pub async fn belongs_to<P: Model, D: Db + ?Sized>(db: &D, fk: impl Into<Value>) -> Result<Option<P>>;
```

- `children_query` discovers the fk via `fk_column_to`; no metadata →
  `QueryError` naming both models and the `#[bee(fk = …)]` fix; the builder is
  `C::query().filter_eq(fk_column, parent.pk_value())?`.
- `children_for` is the anti-N+1 workhorse: dedupe parent pk values, chunk at
  `MAX_BIND_PARAMS` (999 — hoisted in `lib.rs` so `insert_many` and this share
  one constant), one `filter_in` query per chunk, group rows by the fk
  column's JSON value (`serde_json::Value::to_string()` — canonical string
  key; `Value` itself has no `Hash`/`Eq` because of `Float`), and return
  **index-aligned with the input slice** (`Vec<Vec<C>>`; the same parent twice
  → its group twice). Empty `parents` → `Ok(vec![])`, no db round-trip.
  Amendment (found during implementation, real-DB semantics): `Json::Bool` is
  normalized to `1`/`0` before stringifying — the parent side holds
  `Value::Bool` (`"true"`/`"false"`), while sqlite/mysql read a Bool column
  back as 1/0, so a literal `to_string()` could not group a Bool pk on real
  backends; Text `"1"` stays distinct from integer 1 (five-case test pins it).
- `belongs_to`: `P::query().filter_eq(P::pk_column(), fk)?.one(db)` — the
  parent's soft filter applies (soft-deleted → `None`); a NULL fk never
  matches (`= NULL` SQL semantics) → `None`.
- `QuerySet::filter_in(field, values: &[Value])` (new, additive): one
  `field IN (?, ?, …)` term, N params in positional order; **empty slice is an
  error** (`QueryError`, "filter_in: empty value list") — loud beats a silent
  nothing-match; internal callers skip empty input.

**Cross-behavior (explicit, per the brief):**

- soft delete × relations: relation reads inherit the soft filter of the
  queried model (deleted children excluded; soft-deleted parent →
  `belongs_to` returns `None`, `children` still returns the children rows —
  filtering is per-model, the fk link is not consulted). Escaping to deleted
  rows means the query-builder variants + `with_deleted()`. No cascade
  anything: soft-deleting or hard-deleting a parent never touches children
  rows.
- timestamps/hooks × relations: the relation API is **read-only** — there is
  no relation write path this round. Writing a child is an ordinary
  `insert`/`update` (hooks and timestamps behave exactly as §23/§25), with the
  fk column set by hand.
- migrations × relations: FK renders inline (`REFERENCES`, §35). Create
  parents before children — no registry means no auto-ordering; a missing
  parent fails loudly on pg (at CREATE) and on sqlite (at the first child
  INSERT), while mysql accepts both (the clause is ignored — §35 amendment).
  No `ON DELETE` action is emitted, so the SQL default applies: on the
  enforcing backends a parent hard-delete with existing children **errors**
  (nothing is removed silently); mysql allows it. Soft delete is an UPDATE
  and never trips the FK. Treat the clause as portable metadata with pg +
  sqlite (bundled) enforcement and a documented mysql gap.
- fk values in grouping: Int/Text/Bool pks work; a float pk is not a
  supported model design (documented).

## 37. Macro changes (round 4)

**§37.0 split first.** `crates/bee_orm_macro/src/lib.rs` is at 493/500; the
file-limit rule outranks locality (§28.1 precedent). Move attribute parsing
and/or codegen into module file(s); the split itself is a pure move — the five
existing expansion tests must be byte-identical across it (§28.4 regression
gate). Only then add round-4 logic.

**§37.1 grammar.** Field options gain `sql_type = "…"` (string literal) and
`fk = <Path>`. The unknown-field-attribute error message gains both names so
the existing diagnostic stays accurate.

**§37.2 checks** (all compile_error at the offending span, message names the
fix):

- unmapped type spelling without `sql_type` → error + override hint (§34
  table; covers aliases, `u64`, custom types).
- `fk` + `ignore` → error (an ignored column cannot carry FK metadata).
- `fk` on the pk field → error.
- `#[bee(auto)]` pk with a non-integer spelling and no `sql_type` → error.
- `Option<T>` outer → `nullable = true` (both for mapped and Raw types).

**§37.3 codegen.** The derive emits
`fn columns() -> &'static [ColumnDef] { … }` in declaration order, ignored
fields excluded, names post-`column`-rename and unraw'd (§18.1); ts/soft/pk
adjustments baked per §34. FK fields emit
`references: Some(Reference { table: <Target as bee_orm::Model>::table_name,
pk_column: <Target as bee_orm::Model>::pk_column })` plus a const assertion
that `Target: bee_orm::Model` (readable error at the attribute for non-Model
targets). The literal shape (static-promoted slice vs explicit
`const COLUMNS: [ColumnDef; N]`) is coder-macro's call, but the
const/fn-pointer promotion must be **scratch-verified before the shape is
committed** (§25 precedent — round-3's hook-forwarder resolution was proven
on scratch first).

**§37.4 test baselines.** The five existing expansion tests necessarily
change — by the added `columns()` block and nothing else; fold the new
baseline in and the review diff-checks exactly that. New unit tests: spelling
table hits (incl. `Option<…>`), one `sql_type` override, error rows (unmapped,
fk+ignore, fk-on-pk, auto non-integer). trybuild: fail cases for unmapped
type and one fk conflict; a pass case exercising `sql_type` + `fk` together.

## 38. Interaction matrix additions

- `create_table` × auto pk → the three dialect forms of §35 (real-DB tests
  pin pg/mysql; sqlite pinned in the e2e).
- `create_table`/`sync` × ts/soft fields → `BIGINT NOT NULL DEFAULT 0` /
  `… DEFAULT 0 NOT NULL` regardless of spelling (the §23 mandate).
- `sync` on an existing table → adds exactly the missing columns; a second
  run returns 0 (idempotent, never destructive).
- `add_missing_columns` on a populated table with a new non-`Option` column
  and no default → loud driver failure on sqlite/pg (documented; use
  `Option<T>` or `sql_type` + `DEFAULT`). For fk columns on populated sqlite
  tables `Option<T>` is the only addable shape — `sql_type` + `DEFAULT`
  cannot carry `REFERENCES` there (§35, probed; both rules are data checks).
- `children`/`children_for` × soft-deleted child → excluded; `with_deleted()`
  via the query-builder variants is the escape.
- `belongs_to` × soft-deleted parent → `None`; × NULL fk → `None`.
- `children_for` × duplicate parent in the slice / empty slice / index
  alignment → covered in the e2e; the 999-chunk loop is exercised at small
  sizes only (a boundary test at 999 rows is impractical; the chunk count is
  arithmetic, reviewed).
- `filter_in` × empty slice → `QueryError` (tested); × N values → positional
  params in order (tested).
- child writes × fk column → plain column semantics (existing insert/update
  coverage; no new machinery).

## 39. Acceptance (round 4)

- **sqlite in-memory e2e** (new test file, `#[cfg(feature = "sqlite")]`):
  `Pool::connect(":memory:", 1)` — note the documented single-connection
  clamp, which is what makes in-memory coherent. Flow: `create_table::<M>` →
  insert → `find` → add-column path via a hand-made table missing one column
  → `sync` adds it → reads work → `sync` again returns 0. Relations roundtrip
  with two parents + children: `children` / `children_for` (alignment +
  duplicate parent), `belongs_to`, soft-delete filtering both directions,
  `filter_in` empty-slice error.
- **Real-DB, DSN-gated** (`BEE_ORM_PG_DSN` / `BEE_ORM_MYSQL_DSN`, existing
  §30 pattern, no CI change expected): `create_table` (incl. auto pk and an FK
  `REFERENCES` parent→child pair, parents created first) + `sync`
  idempotency, per backend. The §36 enforcement matrix is pinned where cheap:
  pg asserts the loud missing-parent failure; mysql asserts a dangling create
  still succeeds (the documented ignore semantics); sqlite asserts the
  dangling INSERT fails (`FOREIGN KEY constraint failed` — the bundled
  default is enforcement-on).
- **Macro**: `cargo test -p bee_orm_macro` — unit + trybuild green; the five
  expansion-test diffs contain only the `columns()` block.
- **Gates** (round-3 five-stage pattern, rc=0 each): fmt; clippy
  `--all-features -D warnings`; build all-features; build bare; test
  `--all-features`.
- **Never-destructive audit**: `migrate.rs` contains no `DROP`, no type
  change, no rename machinery — verified at review by reading, plus the
  idempotent-second-run test above.
- Docs untouched this round (lead schedules the sweep after round 4).

## 40. Round-4 decision log

- Type mapping is the hybrid: spelling table + `SqlType::Raw` override.
  Spelling-only cannot serve aliases (a derive sees spellings, not resolved
  types); override-only would tax every plain `i64` field. One verbatim
  override string for all dialects is an accepted limitation (no per-dialect
  split until a real need exists).
- `ColumnDef.references` is a `(table, pk_column)` fn pair, not one table
  fn: the DDL clause needs the target's pk column name, and `create_table<M>`
  has no `Target` type parameter to ask — the pair is resolvable at macro
  time without instantiating anything.
- Auto pk: postgres `GENERATED BY DEFAULT AS IDENTITY` over `SERIAL` (modern
  spelling; BY DEFAULT preserves explicit-id inserts, matching sqlite/mysql);
  sqlite `INTEGER PRIMARY KEY AUTOINCREMENT` inline (the only legal form);
  mysql `NOT NULL AUTO_INCREMENT PRIMARY KEY`.
- Column metadata is self-contained: the derive bakes nullability/defaults
  (ts `NOT NULL DEFAULT 0`, soft `DEFAULT 0 NOT NULL`) into `ColumnDef`, so
  `migrate` never consults the `auto_now*`/`soft_delete_column` lists — one
  source of truth, no run-time list/String matching.
- Migrations are free functions in `bee_orm::migrate`, not `Model` methods:
  they are operations over a `Db` (dialect-dependent), and `Model` stays
  about row CRUD; also keeps the frozen `Model` surface additive-free beyond
  `columns()`.
- Add-column reads existing columns first, then issues one explicit ALTER per
  missing column: `ADD COLUMN IF NOT EXISTS` is not portable (mysql lacks
  it), and read-then-diff needs no error-string matching. Never destructive
  by construction.
- No versioned migrations (brief mandate): no history table, no up/down
  files; renames appear as adds, old columns are never dropped, type changes
  are unsupported and documented.
- `bee_cli migrate` not wired: no model registry exists and building one
  (inventory-style auto-registration) needs a new dependency — forbidden.
  Recorded for a future round that can spend that dependency. **Superseded
  in batch 5** by the generative path (§49) — no registry, no dependency.
- Relations are a read-only query API: no join builder (the IN-batched path
  covers list loading at this scale), no m2m (junction DDL + attach/detach
  write API — round 5), no has_one (children + first), no eager-attach
  struct fields (no hidden non-column fields, no `Default` machinery).
- `children_for` returns index-aligned `Vec<Vec<C>>` instead of a
  `HashMap<Value, Vec<C>>`: `Value` has no `Hash`/`Eq` (f64), and
  string-keying by JSON text would need key replication on the caller side —
  alignment with the input slice is the useful shape anyway.
- `filter_in` with an empty slice is an error, not an empty match: a silent
  nothing-matches filter is the classic way `IN ()` bugs reach production;
  internal callers (children_for) skip empties explicitly.
- 999 is hoisted to `MAX_BIND_PARAMS` and shared by `insert_many` and
  `children_for` chunking — one constant, conservative on all backends.
- `Db::dialect()` is a defaulted `Option` method: additive (hand-written and
  mock impls keep compiling), and migrate functions fail loudly on `None`
  rather than guessing a dialect.
- sqlite FK enforcement is the **bundled default**, not a switch we flip:
  `libsqlite3-sys` compiles with `-DSQLITE_DEFAULT_FOREIGN_KEYS=1`
  (`build.rs:123`), so declarative `REFERENCES` is enforced with no
  pool-level `PRAGMA` (verified live — dangling INSERT errors; a pool that
  issues no PRAGMA is not evidence of anything). No dial is added; the
  connection-level `PRAGMA foreign_keys` remains the portability hook if the
  bundled feature is ever dropped.
- fk targets are type paths, not strings: a path gives a compile-time
  `Model` assertion and refactor safety; a string cannot be resolved by a
  derive anyway.
- `u64`/`usize`/`isize` left out of the spelling table: `Value` deliberately
  excludes them (not lossless) — no round-trip exists, so no silent DDL
  mapping; the override is the explicit door.
- FK enforcement parity deferred (mysql-only gap): inline `REFERENCES` is
  enforced by postgres and by sqlite — the bundled build compiles with
  `-DSQLITE_DEFAULT_FOREIGN_KEYS=1`, so the pool issuing no pragma is not
  evidence either way (tester's real-pool probe: `PRAGMA foreign_keys` = 1,
  dangling INSERT rejected) — while mysql parses and ignores inline clauses.
  Closing the gap — mysql table-level `FOREIGN KEY` in CREATE plus
  `ADD CONSTRAINT` in `add_missing_columns` — is a deliberate round-5+
  candidate, not a round-4 wiring tweak: it needs deterministic constraint
  naming and would make `sync` fail at startup on databases with pre-existing
  orphan rows. The DDL keeps the inline clause as the portable statement of
  intent. (A non-bundled sqlite build would need per-connection `PRAGMA
  foreign_keys = ON` via r2d2_sqlite `with_init`; in-tree is bundled.) Docs
  sweep wording: enforced on postgres and sqlite (bundled), ignored by mysql
  — neither "three-backend enforcement" nor "postgres only".

## 41. Batch-5A: small fix pack (bee_rust backend features + pg TLS injection)

Two items, implemented before round-5 work. New write scope this batch:
`crates/bee_rust/**` (coder-orm). No new dependencies.

**A1 — backend forwarding in `bee_rust`.** `crates/bee_rust/Cargo.toml`
gains four features mirroring bee_orm's engine names:

```toml
orm-sqlite = ["orm", "bee_orm/sqlite"]
orm-postgres = ["orm", "bee_orm/postgres"]
orm-postgres-tls = ["orm", "bee_orm/postgres-tls"]
orm-mysql = ["orm", "bee_orm/mysql"]
```

`default`/`full` deliberately unchanged — the umbrella does not force a
bundled-C sqlite build onto every default user, and backend choice stays
explicit (decision log §48). Today `bee_rust/orm` enables no backend at all;
after A1 each engine is one feature away. Acceptance:
`cargo check -p bee_rust --features orm-sqlite` (and `orm-postgres` /
`orm-postgres-tls` / `orm-mysql`, one at a time), plus bare
`cargo check -p bee_rust` and `--features orm` staying green.

**A2 — caller-supplied TLS config for postgres.** `pool/postgres.rs`
(feature `postgres-tls`) gains:

```rust
pub fn connect_tls_with(dsn: &str, max_size: u32, config: rustls::ClientConfig) -> Result<Self>;
```

`connect_tls` becomes a thin wrapper — `connect_tls_with(dsn, max_size,
rustls_client_config()?)` — the bundled-roots config (ring provider + webpki
roots) and all pool behavior stay byte-identical. The crate root gains a
feature-gated `pub use rustls;`: the signature names rustls types, so callers
must be able to name the same rustls the crate links against (version-locked
re-export; `ClientConfig` is `Clone` in rustls 0.23, so one config can be
cloned across pools).

CI (§30's design gains this): with an injection point, the integration job
now does a **positive** TLS handshake test — a test-only no-verify
`ServerCertVerifier` (lives in `tests/`, never in the crate), built from the
re-exported rustls; `connect_tls_with` against the real pg container with a
`sslmode=require` DSN; assert `SELECT ssl FROM pg_stat_ssl WHERE pid =
pg_backend_pid()` is true. Env-gated like the other real-DB tests; the
existing negative (bundled-roots vs self-signed) test stays. This closes the
round-2 note that a real handshake was impossible under the bundled-roots-only
API.

Acceptance: the A1 checks; `cargo check -p bee_orm --features postgres-tls`;
the new TLS test (gated) plus fmt/clippy on touched crates.

## 42. Round-5 scope, ownership, file map

Round 5 = **m2m** + **mysql table-level FK opt-in** + **JSON columns**. Same
hard constraints: zero new dependencies (`serde_json` is already a bee_orm
dependency), files <500 lines, additive by default, no behavior change
without an explicit opt-in.

Ownership: coder-orm → `crates/bee_orm/**` (plus A1's `crates/bee_rust/**`);
coder-macro → `crates/bee_orm_macro/**`; tester → `crates/bee_orm/tests/**`.

Files: coder-macro (`parse.rs` struct-attr grammar + `expand.rs` m2m/Json
emission + `types.rs` spelling table + tests/ui); coder-orm (`model.rs`
M2mDef/`Model::m2m()`/`SqlType::Json`, `value.rs` `Value::Json` + FromValue,
`pool/{postgres,sqlite,mysql}.rs` bind/read arms, `migrate.rs`
MigrateOptions + join tables + constraint machinery, new `src/m2m.rs`,
`lib.rs` re-exports).

## 43. m2m (item B1)

**Grammar.** Struct-level, repeatable, value is a type path (Model-bound
assertion emitted; non-Model target → E0277 at the attribute):

```rust
#[bee(m2m(Tag))]
#[bee(m2m(Tag, table = "user_tag", local = "user_id", foreign = "tag_id"))]
```

Targets are matched by path: the same target twice on one model →
compile_error (discovery is by type and must be unique — so no `_via`
variants exist for m2m). The declaring model is `local`, the target `foreign`.

**Pinned messages (implementation-time rulings).** Landed verbatim in
`parse.rs`/`expand.rs`: duplicate target `bee_orm: duplicate m2m target: each
target can appear only once`; resolved-column collision `bee_orm: m2m columns
collide: add explicit local and foreign overrides`; unknown m2m option
`bee_orm: unknown m2m option (expected table, local, foreign)`; bare-`Value`
Json hint per §45. Two coder-macro additions accepted to keep failures loud
(§18.3 "a duplicate is a typo, never last-wins"): `bee_orm: m2m requires a
target type` (keyed args only — `m2m(table = "x")`) and `bee_orm: duplicate
#[bee(m2m)] option` (repeated key inside one `m2m(...)`). The §8.2 revised-row
struct-key message is extended to `bee_orm: unknown bee attribute for a
struct (expected table, hooks, m2m)`.

**Conventions (macro-time, not runtime).** Defaults derive from the *struct
idents lowercased*: join table `{local}_{target}` (e.g. `user_tag`), columns
`{local}_id` / `{target}_id`. Ident-based (not `table_name`-based) naming
keeps everything `&'static str`/const — no runtime `format!` — and survives
`#[bee(table = "…")]` renames untouched. Self-referential m2m (target path =
`Self`, or any target whose last ident lowercases to the local ident) makes
both column defaults collide → compile_error demanding explicit `local` /
`foreign` overrides. The exact syntactic rule: resolve local/foreign column
names (override or `{ident}_id` default) and error iff the two resolve
**equal** — the `{local}_{target}` table default is never ambiguous and may
stay.

**Metadata** (`model.rs`; `Model::m2m() -> &'static [M2mDef] { &[] }`
defaulted; the derive emits the fn only when at least one m2m attr exists —
omit-when-unused, §28.3):

```rust
pub struct M2mDef {
    pub table: &'static str,          // join table (convention or override)
    pub local_column: &'static str,
    pub foreign_column: &'static str,
    pub target_table: fn() -> &'static str,             // discovery key + REFERENCES
    pub target_columns: fn() -> &'static [ColumnDef],   // target pk name + SqlType for DDL
}
```

**Runtime — `bee_orm::m2m` (new module).** Two-step IN, no JOIN SQL (the
round-4 "no joins" stance holds; the join table is not a `Model`, so step 1
is raw SQL through `Db`, step 2 reuses `QuerySet` + `filter_in` — which means
`R`'s soft filter applies automatically):

```rust
pub fn m2m_def<L: Model, R: Model>() -> Option<&'static M2mDef>; // discovery by target_table
pub async fn related_ids<L: Model, R: Model, D: Db + ?Sized>(db: &D, local: &L) -> Result<Vec<Value>>;
pub async fn related<L: Model, R: Model, D: Db + ?Sized>(db: &D, local: &L) -> Result<Vec<R>>;
pub async fn related_for<L: Model, R: Model, D: Db + ?Sized>(db: &D, locals: &[L]) -> Result<Vec<Vec<R>>>;
pub async fn attach<L: Model, R: Model, D: Db + ?Sized>(db: &D, local: &L, foreign: &R) -> Result<u64>;
pub async fn detach<L: Model, R: Model, D: Db + ?Sized>(db: &D, local: &L, foreign: &R) -> Result<u64>;
```

- `related_ids`: `SELECT <foreign_column> FROM <table> WHERE <local_column>
  = ?` — the escape hatch for callers wanting their own ordering/limits
  (`R::query().filter_in(R::pk_column(), &ids)?.order_by(…)`).
- `related`: step 1 above, then `R::query().filter_in(pk, ids)?.all(db)`.
  No ordering guarantee (join-row order is db-defined) — documented.
- `related_for`: dedupe local pk values, chunk at `MAX_BIND_PARAMS`, one
  chunked `IN` per chunk, group the `(local, foreign)` pairs, one chunked
  `filter_in` for all distinct foreign ids, reassemble **index-aligned with
  the input slice** (same shape as `children_for`; share the grouping helper
  `pub(crate)`). Empty `locals` → `Ok(vec![])`. Foreign ids matching no `R`
  row (deleted or, on mysql, dangling) are dropped — their local gets a
  short/empty group.
- `attach` / `detach`: the minimal write path (a read-only m2m would be
  unusable and the join table has no `Model`). Plain `INSERT` / `DELETE`
  with both pk values bound; **no upsert** (no portable spelling across the
  three dialects — a duplicate `attach` is a loud driver error on our DDL's
  composite PK), `detach` is idempotent (second call → 0 rows). No `_many`
  variants (loop in caller code; YAGNI).
- No m2m def → `QueryError` naming both models and the `#[bee(m2m(…))]` fix.

**Migrations.** `create_table::<M, _>` / `sync::<M, _>` also create each `M2mDef`
join table (`CREATE TABLE IF NOT EXISTS`); `add_missing_columns` does **not**
touch join tables (structural, not column-synced — documented; a renamed
convention leaves the old table alone, never destructive). DDL shape:

```sql
CREATE TABLE IF NOT EXISTS user_tag (
    user_id INTEGER NOT NULL REFERENCES users (id),
    tag_id  INTEGER NOT NULL REFERENCES tags (id),
    PRIMARY KEY (user_id, tag_id),
    UNIQUE (tag_id, user_id)
)
```

Column types are each side's pk `SqlType` (local from `Self::columns()`,
target from `target_columns()`; either side hand-written without `columns()`
→ the existing empty-columns `QueryError`). `PRIMARY KEY` kills duplicate
pairs; `UNIQUE (foreign, local)` is the reverse-direction index — portable
inline on all three dialects in the one statement, and (redundantly) unique
already, so no `CREATE INDEX` idempotency problem (mysql has no
`CREATE INDEX IF NOT EXISTS`). Under the mysql FK opt-in (§44) the join
table's FKs render table-level instead. Ordering rule (sharper than fk
relations): the join table is created by the **declaring** model's
`create_table`, and it references *both* tables — so an m2m **target** model
must be synced before the declaring model (`sync::<Tag, _>` before
`sync::<User, _>` — the two-parameter signature makes the bare
`sync::<Tag>` spelling an E0107). A dangling join-table CREATE fails loudly on pg at DDL time;
sqlite accepts the DDL and fails at INSERT — §35 matrix.

**Cross-behavior.** Join rows are never cleaned by anything: soft- or
hard-deleting `L`/`R` leaves them; `R`'s soft filter hides deleted `R` from
`related`/`related_for`; `L` visibility is the caller's concern (they hold
the instance). `attach`/`detach` touch only the join table — `L`/`R` hooks
and timestamps do not run.

## 44. mysql table-level FK opt-in (item B2)

**Explicit opt-in, default off — the existing ignore behavior is
unchanged.** `migrate.rs` gains:

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct MigrateOptions { pub table_level_fk: bool }

pub async fn create_table_with<M: Model, D: Db + ?Sized>(db: &D, options: MigrateOptions) -> Result<()>;
pub async fn add_missing_columns_with<M: Model, D: Db + ?Sized>(db: &D, options: MigrateOptions) -> Result<u64>;
pub async fn sync_with<M: Model, D: Db + ?Sized>(db: &D, options: MigrateOptions) -> Result<u64>;
```

The existing three functions become `MigrateOptions::default()` wrappers.
**Effect: mysql only** — on pg/sqlite the option is a documented no-op (inline
already enforced); nothing about those dialects' DDL changes.

- CREATE path (mysql + opt-in): each fk column loses its (ignored) inline
  `REFERENCES` and the table body gains
  `FOREIGN KEY ({col}) REFERENCES {t} ({pk})` (join tables included).
- ADD path: `ADD COLUMN` (no inline reference on mysql) followed by
  `ALTER TABLE {t} ADD CONSTRAINT {name} FOREIGN KEY ({col}) REFERENCES {t2}
  ({pk})`, where `{name}` = `{table}_{column}_fk` (documented; mysql's
  64-char identifier limit not specially handled — driver error if exceeded).
  Idempotency: constraint existence checked by name
  (`information_schema.table_constraints`, `constraint_type = 'FOREIGN KEY'`,
  schema `DATABASE()`); present → skip (a guard against re-adding a
  constraint that already exists, not a retry mechanism). **Not convergent
  after a failure (tester-pinned, `integration_mysql_fk.rs`): an existing
  column short-circuits before the constraint-name check ever runs, so a
  failed `ADD CONSTRAINT` (orphans) is not retried by re-running — the
  remedy is a manual `ALTER TABLE … ADD CONSTRAINT {table}_{column}_fk`
  (verified enforced once created), consistent with the no-retrofit ruling.**
- **Orphan policy: error.** An `ADD CONSTRAINT` over pre-existing orphan rows
  fails with the driver's error propagated as `QueryError` (clean the data or
  stay opted out — a warn-and-continue would leave a half-applied schema).
- **No retrofit**: enabling the option on a database whose tables already
  exist adds nothing to existing columns (fresh `CREATE TABLE`s and
  columns added while opted in are the covered paths; retrofitting is manual
  `ALTER`s — documented, out of scope).

## 45. JSON columns (item B3)

**`Value::Json(serde_json::Value)`** — a variant, not a conversion layer:
round-tripping pg `jsonb` through `Text` is not possible (a text-typed bind
parameter is rejected by the server against a jsonb target), so the value
type must carry it. `serde_json` is already a dependency (zero new deps);
`Value` keeps `Debug, Clone, PartialEq` (no `Hash`/`Eq` — same `f64`
reason as `Float`).

- **Conversions**: `impl From<serde_json::Value> for Value`; `FromValue for
  serde_json::Value` — parse-on-text/bytes for sqlite (TEXT holding the
  serialized form; invalid JSON → decode error, honest), pass-through for pg
  (`json`/`jsonb` read branch added to the pool helper, `opt::<serde_json::Value>`
  — the `with-serde_json-1` feature is already on), and mysql json (arrives
  as bytes; decoded via parse — the roundtrip is pinned by the gated test,
  mechanism coder's choice). Scalar JSON (`42`, `true`, `null`) maps
  directly where the backend already yields scalars.
- **Writes**: pg — `serde_json::Value`'s existing `ToSql` (accepts json and
  jsonb; the server infers the parameter type at prepare). sqlite —
  `to_string()` as TEXT. mysql — serialized as bytes (json columns accept a
  valid JSON string).
- **Macro spelling**: the path `serde_json::Value` (segments
  `serde_json` + `Value`) maps to new `SqlType::Json` — rendered sqlite
  `TEXT` / pg `JSONB` / mysql `JSON`; `#[bee(sql_type = …)]` overrides
  verbatim (that is the pg `JSON` vs `JSONB` dial). A bare single-segment
  `Value` stays **unmapped** (ambiguous with `bee_orm::Value` — the existing
  compile_error + `sql_type` hint covers it). `SqlType` gains
  `#[non_exhaustive]` while a variant is being added (protects future
  additions; nothing is released yet, but the fence is free).
- `Option<serde_json::Value>` → nullable, per the usual rule.

**Null semantics pinned at implementation (§47 "null both ways").** A SQL
NULL must decode to `None` for `Option<serde_json::Value>` while a *stored*
JSON null document decodes to `Some(Json::Null)` — so the decision is made on
the raw cell, before parsing: SQL NULL is the only `Json::Null` raw cell, and
on pg the json/jsonb read branch re-serializes a JSON null document to the
text `"null"` (objects, arrays, and other scalars keep passing through
untouched; no existing test pinned the old shape). Non-Option targets get
`Json::Null` for both. Mechanism (internal): since the `DeserializeOwned`
blanket `FromValue` impl cannot be specialized (E0119), `decode` — now
`decode<T: FromValue + 'static>`, no API change for other types — recognizes
the `serde_json::Value` / `Option<serde_json::Value>` targets by `TypeId` and
parses text/bytes cells; the Option path returns through a `TypeId`-guarded
`Box<dyn Any>` downcast (no `unsafe`; one small allocation per such cell).
The special case is deliberately scoped to those two top-level targets —
`Vec<serde_json::Value>`, maps, and structs ride the normal blanket path
(the intermediate value is real JSON for array/object cells, so element
values pass through correctly).

## 46. Interaction matrix additions

Round-5 rows on top of §38:

| | m2m | opt-in FK | Json |
|---|---|---|---|
| soft delete | `R` deleted → hidden by `R::query()` filter; join rows left behind | join tables' FKs render table-level under opt-in | n/a |
| migrate | join tables in `create_table`/`sync` only (never `add_missing_columns`); parents before children | retrofit of existing columns: **no** — fresh creates and newly added columns only | `SqlType::Json` DDL per §45 |
| dialect gaps | attach duplicate → driver error (PK); dangling foreign ids dropped from `related*` (mysql, ignored FK) | pg/sqlite: option is a documented no-op | sqlite strict (`TEXT`, invalid JSON → decode error); pg JSONB; mysql JSON |
| hooks/timestamps | `attach`/`detach` write only the join table — `L`/`R` hooks and auto fields do not run | n/a | n/a |
| nullable | m2m columns are NOT NULL by construction | — | `Option<serde_json::Value>` per usual rule |

## 47. Round-5 acceptance

**coder-macro**: grammar units — `m2m(Target)` with and without keyed
overrides; duplicate target → compile_error; self-m2m without overrides →
compile_error; existing expansions' sha-check: non-m2m models byte-identical
(omit-when-unused), new emissions limited to m2m models; spelling: `serde_json::Value`
→ `SqlType::Json`, bare `Value` stays unmapped; trybuild rows added for
`m2m` duplicate target, self-m2m default collision, and a pass row combining
`m2m` + a Json field. Old rows untouched.

**coder-orm**: `cargo check` per backend feature + all-features + A1's
`bee_rust` feature checks still green; lib/unit tests; sqlite e2e in
`tests/`: m2m via `create_table` (join DDL asserted), attach → related →
`detach` → related shrinks (`detach` again → 0 rows), duplicate attach
errors, `related_for` index-aligned with an empty group included, soft-deleted
`R` excluded, `related_ids` escape hatch honored; Json: a model with
`serde_json::Value` + `Option<serde_json::Value>` round-trips (object, string
scalar, null both ways), raw `Value::Json` through `QuerySet`, invalid TEXT
→ decode error. DSN-gated: pg jsonb roundtrip + m2m end-to-end; mysql json
roundtrip; mysql opt-in — fresh `create_table_with` → `information_schema`
shows the FK and a dangling INSERT now **fails**, default path still
succeeds (round-4 pin unchanged), `add_missing_columns_with` re-run is a
no-op (name-matched), and an ADD CONSTRAINT over orphans (populated table +
fk column carrying a non-null hanging default) propagates the driver error
as `QueryError`.

**tester**: independent re-run of the above on the frozen tree; own
falsification of the m2m assembly (index alignment, chunking boundary at
`MAX_BIND_PARAMS`) and of the opt-in idempotency; the A2 TLS handshake test
(§41) stays tester-owned. **Files <500 lines; zero new dependencies**
(`cargo tree` diff on touched crates must be empty).

## 48. Batch-5 decision log

- **A1**: default/full deliberately unchanged (bundled sqlite stays opt-in
  via `orm-sqlite`; `full` remains the light stack). **A2**: `connect_tls_with`
  takes `rustls::ClientConfig` by value (caller-owned, no generic parameter —
  the wrapper needs a concrete type) + `pub use rustls;` re-export so callers
  never version-match by hand. TLS CI test: real handshake against the CI
  postgres, test-only no-verify `ServerCertVerifier` (generating a self-signed
  cert would need a new dependency — forbidden; verification is the caller's
  dial, the transport is what we test).
- **B1**: macro-time ident-lowercase naming (const metadata, no runtime
  `format!`, `#[bee(table = …)]` renames don't shift join names); duplicate
  target + self-m2m defaults are compile errors → discovery is unique → no
  `_via` variants; two-step IN, no JOIN SQL (join table is not a `Model`;
  step 2's `QuerySet` gives ordering/limit and the soft filter for free);
  `attach`/`detach` are the minimal write path (no upsert — no portable
  spelling; duplicate attach is a loud PK error, detach idempotent);
  composite PK + `UNIQUE (foreign, local)` = both-direction index coverage in
  one portable statement (no `CREATE INDEX IF NOT EXISTS` — mysql lacks it);
  join tables ride `create_table`/`sync` only (RunSyncdb spirit; never
  column-synced); `m2m()` emitted omit-when-unused (§28.3); no join-row
  cleanup anywhere (never destructive).
- **B2**: `MigrateOptions` default `false`; mysql-only effect (pg/sqlite no-op,
  documented); constraint name `{table}_{column}_fk` + `information_schema`
  name check for idempotency; orphans → error (a warn-and-continue would
  leave a half-applied schema); **no retrofit** of existing columns.
- **B3**: `Value::Json` variant (a conversion layer cannot bind pg jsonb —
  text-typed parameters are rejected; the value type must carry it);
  `serde_json` is already a dependency; sqlite stores the serialized TEXT
  form, decode parses (invalid → error); single-segment `Value` stays
  unmapped (ambiguous with `bee_orm::Value`); `SqlType` gains
  `#[non_exhaustive]` while a variant is added.

## 49. Batch-5C: `bee-rust migrate` (generative, no registry)

Owner: **coder-orm** (crates/bee_cli/**; they already hold bee_orm + bee_rust
this batch). Written after B froze because the generated entrypoint's body
must name B's surface (`sync` including join tables, §43/§44's `_with`
options only if the user opts in). **Zero new dependencies** (bee_cli already
has clap; the rest is `std::fs` + `std::process`). **The CLI process never
links user models and holds no global mutable state** — the registry design
is rejected outright: a CLI-process registry cannot see models that live in
the user's crate without linking it or parsing its source (both are heavier
than generating four lines the user owns).

Two subcommands under the existing CLI:

```
bee-rust migrate init   # scaffold src/bin/bee_migrate.rs in the current crate
bee-rust migrate run    # cargo run --bin bee_migrate
```

**`init`** — boundary checks first: `./Cargo.toml` must exist (error exit
naming the path if not); `src/bin/bee_migrate.rs` must **not** exist (error
exit `"… already exists; remove it or edit it in place"` — never overwrite
user code). Writes `src/bin/bee_migrate.rs` (creating `src/bin/` if needed).
The file is not a template with placeholders — it is a runnable program the
user edits exactly once. Required content (exact wording is the coder's,
these elements are pinned):

- a header comment: "generated by `bee-rust migrate init`; edit freely";
  "add `bee_orm` (with your backend feature) and `tokio` to
  [dependencies] if not already present" (init does **not** edit
  `Cargo.toml` — that is the user's file);
- `#[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>>`
  reading the DSN from `DATABASE_URL` (`std::env::var`, error if unset);
- the pool construction for the chosen backend with a TODO marker;
- an ordered `TODO` block of `bee_orm::migrate::sync::<Model, _>(&db).await?;`
  lines, with a comment stating the rule: parents before children (FKs and
  cross-model join tables are created in that order), and that `sync` also
  creates each `#[bee(m2m(…))]` join table (§43). **Spelling corrected at
  implementation (e2e-caught): `sync<M, D: Db + ?Sized>` has two type
  parameters, so the bare `sync::<Model>(&db)` this section first drafted is
  an E0107 — the scaffold and its tests pin `sync::<Model, _>(&db)`.**

**`run`** — refuse with a friendly error (exit ≠ 0) if
`src/bin/bee_migrate.rs` is missing ("run `bee-rust migrate init` first"),
then `std::process::Command::new("cargo").args(["run", "--bin",
"bee_migrate"])` with inherited stdio; propagate cargo's exit code verbatim
(build errors land in the user's terminal unchanged — the CLI adds no build
logic of its own).

Acceptance (tester): unit tests for the scaffold logic in bee_cli's tests
(temp dir: `init` creates the file and its parent, second `init` refuses
without touching the file, missing `Cargo.toml` refuses, `run` without the
bin refuses) — file-ops only, no network, no cargo invocation; plus one real
e2e in a scratch crate (or `examples/hello` if it can host the bin cheaply):
`init` → user fills in a sqlite model → `run` → rc 0 and the table exists.
Exact e2e placement is the coder's call, but a real `cargo run` of a
scaffolded crate must happen at least once before the batch closes. Docs
wording (CLI row `planned` → `implemented`) rides the lead-triggered final
sweep. **Additional final-sweep item (found at C implementation):** the old
`migrate up|down` CLI is gone — `docs/api*.md` (13 languages) and the
2026-07-29 spec still advertise it; the sweep must replace it with
`migrate init|run`. Also note the sweep should quote call sites as
`sync::<Model, _>(&db)` (§49 above) — the one-parameter spelling does not
compile.

## 50. Batch-5 tail: bee_kv RedisStore reconnect (coder-orm)

Scope: `crates/bee_kv/**`. Current state (verified): `redis_store.rs` holds a
bare `redis::aio::MultiplexedConnection` — no reconnect, every command errors
after a break — while `Cargo.toml` already enables the `connection-manager`
feature (dead config). Fix: the field becomes
`redis::aio::ConnectionManager`, built with
`ConnectionManager::new_with_config(client, ConnectionManagerConfig::new()
.set_connection_timeout(5s).set_response_timeout(30s))` — the same 5 s/30 s
timeouts as today, eager connect (bad addr still fails in `new`, so the
`RedisStore::new(addr)` signature and error semantics are unchanged). Method
bodies do not change: `ConnectionManager` is `Clone` and implements
`ConnectionLike`, so the existing `query_async(&mut self.conn.clone())` idiom
is untouched.

Source-verified semantics to document (redis 0.27.6, `aio/connection_manager.rs`):
reconnection is **lazy and background** — `send_packed_command` checks the
connection future pre-send (`reconnect_if_io_error!`) and, on a dropped-IO
result, triggers `reconnect()` (which `spawn`s the new connection with
exponential backoff + jitter); the failing command returns its error and the
*next* command awaits the reconnected connection. So: automatic recovery
without operator action, but the command that hits the break may error —
crate docs say exactly this (no health-check thread exists).

Verification: build + unit tests (offline); reconnect e2e is env-gated
(`BEE_KV_REDIS_DSN`, e.g. `redis://127.0.0.1:6379`) — connect, SET, kill the
connection from a side client (`CLIENT KILL`), then bounded retry (≤10
attempts, ~100 ms apart) until GET succeeds; assert recovery, do **not**
assert a specific attempt count. If no redis env exists when the batch
closes, the e2e is tagged review-only. The test file is written by coder-orm
inside `bee_kv` (their scope, env-gated like the other real-server suites);
tester owns the CI side — a `redis:7-alpine` service in the integration job
makes it real — and verifies independently when an env exists.

Landed 2026-10-06 (coder-orm): as specced, plus one implementation note —
the kill is a **raw RESP `CLIENT KILL TYPE normal` sent over a throwaway
`TcpStream`** instead of a redis client call, deliberately avoiding a redis
dev-dependency so `cargo tree -p bee_kv` stays byte-identical. Live run
against a real container observed `attempt 1 failed: … broken pipe` /
`recovered on attempt 2` — exactly the lazy-reconnect semantics documented
above. Tester's independent re-verification is open.

## 51. Batch-6 scope (after C): memcached KV store + remote cache backends

Deliverables: `bee_kv` memcached `KvStore`; `bee_cache` Redis/Memcache
`Cache` backends (features/deps are placeholders today — no impls exist).
Owner: coder-orm; scheduled after batch-5 C. Zero new dependencies (redis
0.27 / memcache 0.17 already declared in both crates; one dep *chain* is
removed, below). Files: `bee_kv/src/memcache_store.rs`,
`bee_cache/src/redis_cache.rs`, `bee_cache/src/memcache_cache.rs` + lib.rs
re-exports; existing feature names kept (`redis`, `memcached` in bee_kv;
`redis`, `memcache` in bee_cache).

## 52. memcache 0.17 facts (locked source) and the bee_kv store

Verified against `memcache-0.17.2`:

- **Sync-only.** No async/tokio/futures anywhere; deps are byteorder,
  enum_dispatch, r2d2 (non-optional), rand, url, optional openssl. There is
  no async client to use — every backend op bridges through
  `tokio::task::spawn_blocking` over an `Arc<memcache::Client>`. (Consequence:
  both crates' tokio features gain `"rt"` — spawn_blocking lives there; no
  new crate.)
- **Pool is built in**: `Client::with_pool_size(target, u32)` (`connect` =
  size 1), `Client::with_pool`, `pub type Pool = r2d2::Pool<ConnectionManager>`
  — so no bb8 and no r2d2-memcache (a new crate) is needed. Default size 4,
  plus a `with_pool_size` constructor; docs note that under >4-way
  concurrency spawn_blocking threads wait on pool checkout (r2d2 default
  30 s checkout timeout).
- **Binary-safe**: `ToMemcacheValue for &[u8]`, `FromMemcacheValue for
  Vec<u8>`; `add` / `increment` / `decrement` / `touch` / `delete -> bool`
  exist; **no multi-get and no exists command**; `KeyNotFound` arrives as
  `MemcacheError::CommandError(CommandError::KeyNotFound)`; expiration is
  `u32` seconds where **0 = never expires** and **>30 days = absolute unix
  timestamp** (protocol).
- **Wiring**: both crates set the memcache dep to `default-features = false`
  — the default `tls` feature pulls `openssl-sys`; the crate is cfg-gated on
  it and compiles without (TLS-to-memcached unsupported, documented; re-enable
  by flipping the flag if ever needed). `dep:openssl` leaves the graph.

`MemcacheStore` (module `memcache_store.rs`, feature `memcached`):
`new(addr)` and `with_pool_size(addr, size)` (addr passes through
`Connectable` — `127.0.0.1:11211` or a `memcached://` URL). Mappings:

- `get`/`set`/`del` straight; `exists` = `get().is_some()` (no EXISTS in the
  protocol — a full fetch is the only way, documented).
- `expire(seconds)`: ≤ 0 → `delete` (mirrors Redis `EXPIRE k ≤0` deleting the
  key — and dodges the memcached 0 = never-expires trap); > 30 d →
  `now_unix + seconds` (absolute form); otherwise seconds; saturating `u32`.
  Missing key → `Ok(())` (mirrors RedisStore's current "EXPIRE reply 0 is
  fine" behavior).
- `incr(key, amount)`: `amount >= 0` → `increment(key, amount as u64)`;
  negative → `decrement(key, amount.unsigned_abs())`; result `u64 → i64`
  (`unsigned_abs`, so `i64::MIN` cannot overflow on negation).
  **Corrected at implementation (probe-verified against a real memcached
  1.6):** the binary protocol memcache 0.17 speaks does **not** return
  NOT_FOUND for a missing key — `increment` *creates the counter at 0 and
  returns Ok(0) without applying the delta*, so the KeyNotFound retry this
  section first drafted never fires, and a naive first `incr(k, 1)` would
  silently return 0. Landed shape: eagerly `create_counter` — `add(key,
  "0", 0)`, treating `Ok(())` and `KeyExists` (binary; the text-mode
  `NOT_STORED` maps to `Ok(())`) as success — before every
  increment/decrement; costs one extra round trip per op. The alternative
  "reply 0 means just created" heuristic was rejected as
  protocol-quirk-fragile. A missing-key create yields a persistent counter
  (exp 0), matching Redis `INCR`.
  **Caveat (documented, pinned by tester):** counters are unsigned — a
  decrement floors at 0 where Redis goes negative.
- `mget`/`mset`: per-key loops (no multi API in the crate) — order preserved,
  no atomicity, documented.

## 53. bee_cache remote backends (direct, not adapted)

**Ruling: implement directly against redis/memcache; do not adapt bee_kv's
stores.** An adapter must encode `Vec<u8>` into `KvStore`'s `String` (hex or
base64 — a hand-rolled codec or a new dependency) which breaks byte fidelity
and foreign-tool interop, splits `set(bytes, ttl)` into SET+EXPIRE (the
single atomic command exists), and collides with `Cache::incr` (a
hex-encoded counter is not a numeric string). bee_cache also has no bee_kv
dependency today — adapting would add a workspace dependency for no gain.
The two wrappers (~60 lines each) deliberately duplicate the connection
handling; a shared internal crate is overkill at this size (revisit only if a
third consumer appears).

- **`RedisCache`**: deps mirror bee_kv's redis features
  (`["tokio-comp", "aio", "connection-manager"]`); same `ConnectionManager`
  and reconnect semantics as §50 (same doc paragraph). `get` → GET (binary
  safe); `set` → `SET key val [EX ttl]` — one atomic command; **`ttl =
  Some(0)` → `DEL` + Ok** (redis rejects `EX 0`; MemoryCache's observable
  semantics for Some(0) is an already-expired entry, i.e. absent); `delete` →
  `DEL` count 0 → `CacheError::NotFound` (trait contract); `incr` → `INCR`
  (creates 0). Error mapping by kind: `ErrorKind::ResponseError` →
  `CacheError::SerializeError` (covers INCR on a non-numeric value and
  WRONGTYPE), everything else → `ConnectionError`.
- **`MemcacheCache`**: same store wiring as §52 (spawn_blocking, pool,
  `default-features = false`); `set` ttl mapping identical to §52 (Some(0) →
  delete; > 30 d absolute; saturating u32; None → memcached 0 = never);
  `delete() -> bool` → false = `NotFound`; `incr` = eager `create_counter` +
  `increment(key, 1)` (same corrected mechanism as §52 — see the amendment
  there); data refusals → `SerializeError`, transport errors →
  `ConnectionError`. **Corrected at implementation:** the binary protocol
  reports refusals as status codes, not `CLIENT_ERROR` text — a non-numeric
  counter arrives as `CommandError::Unknown(0x0006)` and is matched
  explicitly (`NON_NUMERIC_COUNTER`); text-mode ClientError → SerializeError
  is kept as well.

## 54. Batch-6 acceptance

Unit (offline): expiration mapping fn (0 / ≤0 / normal / >30 d absolute /
u32 saturation), incr dispatch (increment vs decrement), redis SET-with-EX
packing as in `redis_store.rs`'s existing packed-command tests. Env-gated
e2e (bee_kv: `BEE_KV_REDIS_DSN`, `BEE_KV_MEMCACHE_ADDR`; bee_cache reuses the
same two vars): per backend — set/get round trip, delete → `NotFound`, incr
creates-at-0 then increments, TTL expiry (short ttl, sleep, gone),
non-UTF-8 byte fidelity through `RedisCache` and `MemcacheCache`; memcache
only: large ttl (>30 d) set+get stays alive (absolute-timestamp mapping);
memcache only: **decrement below zero floors at 0** (unsigned counters —
divergence from Redis, pin it explicitly, e.g. decr from 2 by 5 → 0, and a
second decrement stays 0). Placement: this pin goes in the **memcache-gated
test**, never in the shared `exercise` body — Redis returns -3 for the same
operation, so a shared-body assertion would red one backend or the other.
`cargo tree` diff on both crates: no additions; `dep:openssl` removals
expected (from the memcache default-features flip). The gated e2e tests are
written by coder-orm inside `bee_kv`/`bee_cache` (their scope, same file
conventions as §50); tester may add `redis:7-alpine` + `memcached:1-alpine`
services to the integration job (ci.yml is tester's — recommended, this is
what makes both suites real in CI); if no env exists at close, the memcache
e2e is review-only.

Decision log (batch 6): direct backends (above); r2d2 via the memcache crate
itself (bb8 / r2d2-memcache rejected — new crates for an already-shipped
pool); spawn_blocking per op as the sync→async bridge (ceiling: one blocking
thread per op in flight — fine at this scale, revisit only on measured
pressure); built-in pool, default 4 + `with_pool_size`; `tls` off by
deliberate default (openssl chain); expire ≤0 → delete; incr-on-miss = eager
`create_counter` (the drafted NOT_FOUND dance was disproved by probe: the
binary protocol creates at 0 with Ok(0) and no delta — see §52); exists via
get; mget/mset loops; ttl 0 → delete in caches; >30 d absolute; error-kind
mapping.

Landed 2026-10-06 (coder-orm), all gates rc=0: `bee_kv/src/memcache_store.rs`
(new) + `bee_kv/tests/kv_store_e2e.rs` (shared `exercise(&dyn KvStore)` +
gated per-backend tests); `bee_cache/src/{redis_cache,memcache_cache}.rs`
(new) + `bee_cache/tests/remote_caches.rs` (same shape); lib.rs re-exports.
Manifests: feature-list / `default-features` flips only. `cargo tree` diff:
zero additions; the memcache `tls`-off flip *removed* openssl, openssl-sys,
openssl-macros, openssl-probe and foreign-types from `Cargo.lock`. Live e2e
green against real containers on both crates (redis + memcache, both env
vars inert when unset). Duplicated store wiring between the two crates is
deliberate per §53 (String vs `Vec<u8>` trait surfaces). Tester's quiet
window: §50 mutation + these suites + full-crate runs.

---

## 55. Batch-7A: m2m 错误提示按类型 ident 拼写（reviewer backlog ①）

reviewer 收口 LOW 观察项（m2m.rs:180）：`def_or_err` 的 `#[bee(m2m({}))]` 建议片段取 `R::table_name()` —— 目标模型经 `#[bee(table = "…")]` 改名后，按表名拼出的属性不可编译（属性位要类型 ident）。`src/tests/m2m.rs:149-160` 的 `Reader`/`Author` 恰好都是改名表（表 `readers`/`authors`），天然可钉。

**机制裁定（M2mDef 携带 ident + 反向 def 拼提示）**：失败查询 `def_or_err::<L, R>()` 在正向无 def 时，运行时没有任何途径拿到 `R` 的**类型 ident** —— `Model` trait（model.rs:163 起）不带 ident，为它加 trait 项会破坏手写 impl 面。唯一可读的 ident 是**反向声明** `m2m_def::<R, L>()` 那条 def 的 target ident：它拼出的正是提示里要展示的 `#[bee(m2m(Author))]` 片段。因此：宏在 emission 侧把 target ident 字符串写入 `M2mDef`；`def_or_err` 正向失败时读反向 def 的该字段拼提示；两向都无 def 时没有 ident 可拼——用字面占位符给形状，**绝不把表名放进 `m2m(…)` 片段位**（表名看着可复制、复制即编译错，正是被报的坑）。

**宏侧（coder-macro，先动）**：
- `M2mDef` 新增 `target_ident: &'static str`。取 plain str（ident 宏期即知，不需要 target_table/target_columns 那样经 fn 指针延后到目标类型解析）；其余字段形式不动。
- 取值语义：属性 target 路径**最后一节 ident、`unraw()`、保留原大小写**（`Tag`→`"Tag"`；`crate::models::Tag`→`"Tag"`；`r#type`→`"type"`）；`Self` → 声明侧结构体名（同 expand.rs:264-269 断言拼写的既有 Self 语义）。注意 parse.rs:165-174 已有的 `target_ident` 是**小写**默认值拼写（table/local/foreign 默认用）——§55 是新字段、**原样**拼写，同源不同物，不得复用混写。落点建议：parse.rs 的 `M2m` 存原样 ident（String），expand.rs 的 M2mDef 字面量（:239-245）发字面量。
- 宏单测/token 基线覆盖三拼写（裸 ident / 全路径末节 / Self）与 unraw。
- Landed（coder-macro，2026-10-06）：`M2m` 落点存 **LitStr**（自带 target span、与 table/local/foreign 同型；小写默认改由 verbatim 派生，§43 三默认逐字不变）；单测四拼写含 unraw（显式断言字面量**永不出现** `"r#type"`）与 §43 小写默认对照；trybuild pass 用例升级 `crate::Category` 全路径 + `target_ident` 运行期断言。宏侧门禁：单测 21/21、trybuild 7 fail + 3 pass、fmt/clippy rc=0。

**ORM 侧（coder-orm）**：
- `model.rs` `M2mDef`（:136-148）加同名字段；手写 `PartialEq`（:150-160）加 `self.target_ident == other.target_ident`（直接比 str，不是 fn 指针；派生模型互比不回归）。
- `def_or_err::<L, R>`（m2m.rs:176-185）：

```rust
m2m_def::<L, R>().ok_or_else(|| {
    let hint = match m2m_def::<R, L>() {
        Some(reverse) => format!(
            "`{}` declares the reverse `#[bee(m2m({}))]` — call from `{}`, or declare `#[bee(m2m(<Target>))]` on `{}`",
            R::table_name(), reverse.target_ident, R::table_name(), L::table_name()
        ),
        None => format!(
            "declare `#[bee(m2m(<Target>))]` on `{}` (or the reverse on `{}`)",
            L::table_name(), R::table_name()
        ),
    };
    OrmError::QueryError(format!(
        "m2m: no relation from `{}` to `{}`; {hint}",
        L::table_name(), R::table_name()
    ))
})
```

  措辞可微调顺序，断言以 contains 为准（下）。
- 测试（src/tests/m2m.rs:149-160 扩展）：既有断言全保留（`authors`/`readers`/`#[bee(m2m(`）；新增：`related::<Author, Reader>` 错误信息 `contains("m2m(Author)")` 且 `!contains("m2m(readers")` —— Reader 声明 `m2m(Author)`、两表皆改名，"提示按 ident 拼写"的钉（`Author` 的表名是 `authors`，泄表名即红）；再钉一处反向提示词（按最终措辞）；无声明方向的对（如 `related::<Reader, Tag>`）无表名内嵌片段。
- 验收取**单测**（消息是运行期字符串；lead 的 "trybuild line or unit test" 二选一）——`cargo test -p bee_orm` 全绿即可。
- Landed（coder-orm，2026-10-06）：model.rs :151 字段 + :164 PartialEq 直比（非 fn 指针）；m2m.rs :189 反向读 def 拼 `target_ident`；tests/m2m.rs 三组断言（正向 `contains("m2m(Author)")`、`!contains("m2m(readers")`、反向 `<Target>` 占位）。默认关门禁 lib 52 全绿。

**排程**：bee_orm 侧字段与 bee_orm_macro 侧 emission 必须同树落地才编译（struct 字面量缺字段即红）；两 crate 文件互不重叠，双侧同时动、后落地者跑全门禁。§55 并入下一批提交。

---

## 56. Batch-7B scope: chrono / rust_decimal 类型映射轮

补齐日期时间与 Decimal 的端到端映射（现状债务：pg TIMESTAMP/TIMESTAMPTZ/DATE、mysql DATETIME/DATE、DECIMAL/NUMERIC 读回 null、写不进）。feature-gated：`bee_orm` 新增 `chrono` / `rust_decimal` 两个 feature。

硬约束（lead 定，逐条可验）：
- **feature 关 = 零新依赖、零行为变化**：`Cargo.lock` 零 diff；`cargo tree -p bee_orm` 与轮前逐字节一致；新臂全 cfg，关闭时既有路径逐字节不动。
- **feature 开 = 只加 chrono / rust_decimal 本体**（常规 serde 组合），不加别的直接依赖；引入的包名已全在锁里（§57），故锁仍零 diff。
- feature 关时模型带日期字段 → **编译期**报错（拼写映射无条件，缺 `From<NaiveDate> for Value` 即报）——契约写文档（lead 终刷清单）。
- 范围外：NaiveTime / 时间列（pg postgres-types chrono_04 就没有 NaiveTime impl，三后端无一致载体）；`DateTime<Local>` / `DateTime<FixedOffset>`（只做 UTC；spelling 对非 Utc 泛型参不给映射）；纳秒/typmod 精度参数。

交付面：① Value 扩展（cfg 变体 + From）；② 三后端 bind/read、migrate DDL、SqlType 无条件变体、宏拼写表（§58）；③ bee_rust 转发 feature `orm-chrono` / `orm-rust_decimal`（§41 面一致性，照 `orm-sqlite` 模式）；④ §54 式语义钉子（§59）与真库门（tester）。

---

## 57. 锁定源码事实（2026-10-06 逐一核实，feature 代数成立的前提）

- `mysql_async-0.34.2`：feature `chrono = ["mysql_common/chrono"]`、`rust_decimal = ["mysql_common/rust_decimal"]`；**`default-rustls`（bee_orm 已在用）已含 rust_decimal** → `mysql_common/rust_decimal` 今天就在依赖树里。`mysql_common-0.32.4`：`FromValue for NaiveDate/NaiveTime/NaiveDateTime`、`impl From<NaiveDateTime> for Value`（convert/chrono.rs:186）与 `From<NaiveDate> for Value`（:204）、`impl From<Decimal> for Value`（convert/decimal.rs:62，值转即长度编码文本）——bind 直接用 `mysql_async::Value::from(x)`。binary protocol 的 `Value::Date(u16,u8,u8,u8,u8,u8,u32)` = 年/月/日/时/分/秒/微秒，时间列无 chrono 也已解析成字段。
- `tokio-postgres-0.7.18`：`with-chrono-0_4 = ["postgres-types/with-chrono-0_4"]`。`postgres-types-0.2.14` `chrono_04.rs`：`NaiveDate/NaiveDateTime/DateTime<Utc|Local|FixedOffset>` 有 FromSql/ToSql——**无 NaiveTime**；其 chrono 依赖 default-features=false + clock。
- `rust_decimal-1.43.0`：default = `["serde","std"]`（serde 默认**字符串**形，保精度）；`db-tokio-postgres = ["dep:bytes","dep:postgres-types","std"]`；`src/postgres/driver.rs` `impl FromSql for Decimal`（:20）/ `impl ToSql for Decimal`（:119）——pg NUMERIC 原生收发；无 mysql_async 原生集成（mysql 走文本，见上）。
- `chrono-0.4.45`：default = clock/std/oldtime/wasmbind——**serde 不在默认**：bee_orm 的 chrono dep 必须显式 `features = ["serde"]`。`Cargo.lock` 已含 chrono 0.4.45 / mysql_common 0.32.4 / postgres-types 0.2.14 / rust_decimal 1.43.0 → 本轮 **Cargo.lock 零新 package**（钉子 §59#6）——锁是 feature 无关的：新增 optional 依赖会在锁里落依赖边（实作期实测终态 **6 行、全为既有 package 依赖边追加**：bee_orm +2（chrono、rust_decimal）、bee_orm_macro +1（fail 例 dev-dep）、mysql_common +1（chrono，弱激活链落点——非 mysql_async）、postgres-types +1（chrono）、rust_decimal +1（postgres-types）；零 `[[package]]` 增删、零版本漂移——架构师 `git diff Cargo.lock` 逐行复核）。
- `rusqlite-0.32.1` 带可选 chrono feature——**不用**：sqlite 读写全走显式文本转换与文本解码，少一个 feature 面。
- **读取面关键结论**：chrono/rust_decimal 开 serde 后即 `DeserializeOwned` → 直接走 value.rs 既有 blanket `impl<T: DeserializeOwned> FromValue`（:104-108）：**无需新 FromValue impl、无 E0119、value.rs 零改动**。读取单元格由 `serde_json::to_value(typed)` 产出——与 decode 的 serde 解析天然对称（往返 by construction）。

---

## 58. 设计：Value / feature 代数学 / 三后端映射

### Value（value.rs）
- `#[cfg(feature = "chrono")]`：`Date(NaiveDate)` / `DateTime(NaiveDateTime)` / `DateTimeUtc(DateTime<Utc>)` + 三个 `From`。
- `#[cfg(feature = "rust_decimal")]`：`Decimal(Decimal)` + `From`。
- 命名贴既有短名（Int/Float/Text/Bytes/Json）；`DateTimeUtc` 显式标 UTC（mysql 无时区类型、pg timestamptz 归一）。
- 现有 match 点（pool/sqlite.rs ToSql :168-181、pool/mysql.rs to_mysql :145-156、pool/postgres.rs ToSql :322-392）加 cfg 臂；**不要**以 `_` 兜底吞新变体（类型不匹配须走既有 TypeMismatch 报错，不能静默走错路）。

### feature 代数学（bee_orm/Cargo.toml，:18-22）

```toml
chrono = ["dep:chrono", "tokio-postgres?/with-chrono-0_4", "mysql_async?/chrono"]
rust_decimal = ["dep:rust_decimal"]
postgres = ["dep:tokio-postgres", "dep:deadpool-postgres", "dep:tokio", "rust_decimal?/db-tokio-postgres"]

[dependencies.chrono]
version = "0.4"
features = ["serde"]
optional = true

[dependencies.rust_decimal]
version = "1"
optional = true
```

- `?/` 弱激活：sqlite-only + rust_decimal 不引入 postgres-types；chrono 关闭时 pg/mysql 编译面不动。**tokio-postgres 既有 dep 行不得改**（把 `with-chrono-0_4` 写进 dep features 会 postgres 一开就强拉 chrono，违"关=零变化"）。
- bee_rust（:26-41）：`orm-chrono = ["orm", "bee_orm/chrono"]`、`orm-rust_decimal = ["orm", "bee_orm/rust_decimal"]`。

### SqlType / 迁移（无条件）
- `SqlType`（model.rs:69-81，`#[non_exhaustive]`）加 `Date/DateTime/DateTimeTz/Decimal`——纯数据变体，DDL 渲染不需要 chrono/decimal 类型在场。
- `migrate::sql_type()`（:341）新臂：

| SqlType | sqlite | pg | mysql |
|---|---|---|---|
| Date | TEXT | date | date |
| DateTime | TEXT | timestamp | datetime(6) |
| DateTimeTz | TEXT | timestamptz | timestamp(6) |
| Decimal | TEXT | numeric | decimal(65,30) |

- sqlite 一律 TEXT（iso8601 文本存储；**Decimal 尤其**：声明 DECIMAL 得 NUMERIC 亲和，文本 "1.50" 会被转 REAL 1.5 丢精度——§59#4 钉）。
- mysql `decimal(65,30)`：对齐 pg 无约束 numeric 的"任意精度"意图（读回定标填充串，Decimal 数值相等成立；钱型请 `#[bee(sql_type = Raw("decimal(12,2)"))]` 逃生口）；`datetime(6)`/`timestamp(6)` 微秒精度；timestamp 有 2038 上限与会话时区口径（文档注）。
- 既有 `migrate sync` 幂等语义不受影响（ALTER ADD COLUMN 走同一 sql_type）。

### 宏拼写（bee_orm_macro/src/types.rs `spelling()` :24；expand.rs :133-155 应用）
- `"NaiveDate"→"Date"`；`"NaiveDateTime"→"DateTime"`；`"Decimal"→"Decimal"`。
- `"DateTime"`：仅当唯一泛型参末节为 `Utc` → `"DateTimeTz"`；否则 None（逼 `#[bee(sql_type = …)]` 显式覆盖）。`spelling()` 现按末节匹配，需扩到看 `Type::Path` 泛型参。
- 拼写表无条件、与 feature 无关；`Option<T>` 走既有 nullable 拆解路径（确认四拼写透传）。
- 既有拼写表单测（tests.rs 的 `spelling_table_maps_every_supported_type`）扩：三新拼写 + `DateTime<Utc>` → DateTimeTz + 裸 `DateTime` → None 两例。
- Landed（coder-macro，2026-10-06）：判定 `utc_datetime` = 唯一泛型参且其末节为 `Utc`；裸 `DateTime`/`DateTime<Local>`/`NaiveTime`（范围外）/`Option<DateTime<Local>>` 四例均走既有 "no SQL type mapping … add #[bee(sql_type = …)]" 提示。基线纪律：§34 巨型拼写基线**逐字未动**，§56 五列（Date/DateTime/Decimal/DateTimeTz×nullable 两态）以第二条 assert 追加——旧钉零风险。

### 三后端

**pg**（pool/postgres.rs）
- read `value()`（:278-291）在 `_ => opt::<String>`（:289）前插 cfg 臂：`"date" => serde_opt::<NaiveDate>`、`"timestamp" => serde_opt::<NaiveDateTime>`、`"timestamptz" => serde_opt::<DateTime<Utc>>`、`"numeric" => serde_opt::<Decimal>`。新 helper `serde_opt<T: serde::Serialize>`（既有 `opt()` 是 `Into<Json>` 面；chrono/decimal 是 serde 面）：DB 错/`to_value` 失败 → `Json::Null`（既有 opt 约定）。cfg 关时维持落 `_` 臂的 null 现状。
- ToSql for Value（:322-392）加臂：`(Value::Date(d), "date")`、`(Value::DateTime(dt), "timestamp")`、`(Value::DateTimeUtc(dt), "timestamptz")`、`(Value::Decimal(d), "numeric")` → 各自 `to_sql`；其余组合落既有不匹配报错。
- 单元格 = `to_value` 的 serde 形（date "2026-10-06"、naive "…T…%.f"、Utc RFC3339 Z）——decode 由用户 serde 对称解析。

**mysql**（pool/mysql.rs）
- read：`row_to_json`（:158）已在读 `columns_ref()`；`value(v)`（:169）改为收列信息（列型），cfg(chrono) 臂按**列型**分派（`V::Date` 变体不带列型，DATETIME 与 TIMESTAMP 必须靠 column type 区分）：
  - `MYSQL_TYPE_DATE` → `"YYYY-MM-DD"`；
  - `MYSQL_TYPE_DATETIME` → naive `"YYYY-MM-DDTHH:MM:SS[.ffffff]"`；
  - `MYSQL_TYPE_TIMESTAMP` → 同 naive 形 + `Z`（RFC3339 Z——mysql 无时区类型：TIMESTAMP 视为 UTC，DateTimeUtc 字段对口 TIMESTAMP 列）；
  - `V::Time(..)` 仍 `Json::Null`（范围外，无条件）。cfg 关 → 全体维持现状 null。
  - 口径：DATETIME 单元格解 `NaiveDateTime` ✓、解 `DateTime<Utc>` ✗（无偏移，serde 拒）；TIMESTAMP 反相。列型对口、错配 decode 显式报错，文档写清。
- bind（`to_mysql` :145-156）cfg 臂：`Value::Date(d) → Value::from(d)`；`Value::DateTime(dt) → Value::from(dt)`；`Value::DateTimeUtc(dt) → Value::from(dt.naive_utc())`（归 UTC 存 naive）；`Value::Decimal(d) → Value::from(d)`（mysql_common 的 From 即长度编码文本）。**会话时区口径**：TIMESTAMP 列按会话时区转换读写，严格 UTC 往返要求会话时区=UTC（容器默认即 UTC；文档注）。
- 拼写/DDL 见上表。

**sqlite**（pool/sqlite.rs）
- read 零改动（`value()` :183-194 Text→String 单元格即文本，serde 解码在 decode 侧）。
- bind（`impl rusqlite::ToSql for Value` :168-181）cfg 臂，显式文本转换（不用 rusqlite chrono feature）：
  - `Date(d) → d.format("%Y-%m-%d")`；
  - `DateTime(dt) → dt.format("%Y-%m-%dT%H:%M:%S%.f")`（`%.f` 无小数时不留点）；
  - `DateTimeUtc(dt) → format!("{}Z", dt.naive_utc().format("%Y-%m-%dT%H:%M:%S%.f"))`；
  - `Decimal(d) → d.to_string()`。
- 存储格式即上述字面量（§59#4 钉）。

### Landed（coder-orm，2026-10-06）

- 文件面：model.rs（SqlType 四变体、M2mDef.target_ident）、m2m.rs、value.rs（四 cfg 变体 + 四 From + 2 往返单测，decimal 定值钉 `row["v"]=="1.50"`）、rel.rs、migrate.rs、pool/{sqlite,postgres,mysql}.rs、tests/{m2m,mod}.rs、tests/datetime_types.rs（新，188 行）、bee_orm/Cargo.toml（feature 代数逐字如 §58；tokio-postgres dep 块未动）、bee_rust/Cargo.toml（orm-chrono / orm-rust_decimal）。
- 双跑：默认关 lib 52 + orm_tests 18 + doctests 1；`--all-features` lib 77（70 基线 + value 2 + datetime_types 3 + mysql 单元格 2）+ orm_tests 18 + doctests 5 + 集成 5/2/8/2/6/2/6/6/11（此列表为 coder-orm 时点；tester 真库增补 +4 后 pg 10 / mysql 7，见 §59#8）；clippy 双跑 / fmt rc=0；bee_orm_macro 21 unit + trybuild（8 fail + 3 pass）联测绿。**口径订正（tester，2026-10-06）**：简报原文 "doctests 18" 实为 `tests/orm_tests.rs` 目标 18（两跑法均 18）；真 doctests 为 bare 1 / all-features 5（`cargo test -p bee_orm --doc` 实测）——以 tester 实测为准。
- 树证：默认关 `cargo tree -p bee_orm` grep `chrono|decimal` 零命中；`--all-features` 两者在。
- 变异证伪（逐字节还原后 cmp sqlite.rs 同）：① migrate.rs sqlite Decimal `"TEXT"`→`"DECIMAL"` → DDL 钉红（datetime_types.rs:148）；② sqlite bind `to_string()`→`normalize().to_string()` → 两 decimal 定值钉红（"1.5" vs "1.50"）。
- **历史修订（设计段未列、feature 开才暴露的两处 E0004 必修）**：① pool/postgres.rs `mismatch_error` kind 串加 cfg 臂——诊断只报变体名、不带数据；② rel.rs `json_of` 加 cfg 臂，走 `serde_json::to_value`（与读取单元格同形 → "绑定值 vs 解码 cell" 分组键一致）。两处 cfg 关时逐字节不存在（"关=零变化"不变式不破）。
- 小项：mysql `date_cell` 八参→元组形参（过 clippy too_many_arguments，未用 allow）；datetime_types 夹具四类型列可空（各 feature 只插自己子集）；真库文件 coder-orm 未动——真列型分派钉与 §59#8 归 tester（已落，见 §59#8 Landed）。
- **tester 真库实测补记（2026-10-06）**：mysql `decimal(65,30)` 回填 30 位小数文本读回无损——rust_decimal 解析器按 estimated_max_precision 截断系数并四舍五入，尾零填充不触发 >28 scale 守卫（一度怀疑的缺陷经实测否定）；DECIMAL(35,0) 30 位大数 mysql 保持文本 cell、decode 才 err，与 pg Null cell 的不对称如 §59#3 所述。

---

## 59. 验收与钉子（§54 式语料）

1. **pg TIMESTAMPTZ 时区往返**：`DateTime<Utc>`（源可造自 `parse_from_rfc3339("…+05:30")`）写入→读回同瞬时（微秒）；raw SQL 写 `+05:30` 的 timestamptz → decode `DateTime<Utc>` 为同瞬时 UTC。原生路径（driver impls），无文本中转。
2. **DATE 无时区**：三后端 `NaiveDate` 往返精确（pg date / mysql date / sqlite TEXT），无时间成分。
3. **Decimal 精度损失路径**：pg 原生——超 96-bit 尾数（>29 有效位）的 numeric → FromSql 错 → 单元格 `Json::Null`（文档化天花板，钉 null）；界内 "1.50" 往返数值等（pg numeric 无 typmod 保留 scale）。mysql 文本——>29 位串到 **decode** 才由用户 serde 报错（口径不对称，文档写明）；界内往返数值等。sqlite `to_string()` 文本、TEXT 亲和不变形。
4. **sqlite 存储格式**：raw 读 `typeof(col)=="text"`；定值样本逐字相等（"2026-10-06" / naive "…%.f" / "…Z" / decimal 串）；`sqlite_master` DDL 中 Decimal 列声明为 TEXT（NUMERIC 亲和陷阱守门钉——若声明 DECIMAL，"1.50" 会被转 REAL 1.5，此钉必须能红）。Landed（coder-orm）：已证能红——`"TEXT"→"DECIMAL"` 变异使本钉红于 datetime_types.rs:148，逐字节还原复绿（全记录见 §58 Landed）。独立复验（tester 实做，2026-10-06）：`"TEXT"→"DECIMAL"`（migrate.rs:394 一臂）→ 红 @datetime_types.rs:148:5、红点单一无级联；逐字节还原（sha 回 `2c1a91f2…`）+ touch 后重编译复绿。另 reviewer 收口 M-A 同钉同语义先行证（红 @datetime_types.rs:148、红点单一【2 passed/1 failed，:149 反证钉未触发】→ cmp 还原复绿；写入瞬间 python 锚计数==1 证落笔前为原始态）；mysql 真列型分派独立证伪见 §59#8 Landed——本钉三方证伪闭环（coder-orm + tester + reviewer），原定 reviewer 候选①据此撤下。
5. **mysql 微秒 + TIMESTAMP 归一**：datetime(6)/timestamp(6) 含 `.123456` 往返精确；`DateTimeUtc` ↔ TIMESTAMP 往返（会话 UTC 前提，文档注）。
6. **feature 代数钉子**：`Cargo.lock` 零新 package（唯一 diff = optional 依赖边记录——锁 feature 无关，属预期；**终态 6 行、构成见 §57**；逐条核无 `[[package]]` 增删/版本漂移，架构师已 `git diff Cargo.lock` 逐行复核）；feature 关时 `cargo tree -p bee_orm` 与轮前逐字节一致（取证）；`cargo test -p bee_orm`（默认关）与 `--all-features`（chrono+rust_decimal+全后端）双绿；无 feature 带日期字段的编译失败例——**已裁定**：trybuild **fail 例**（bee_orm_macro 加 chrono **dev-dep** 使类型可解析，触发 `Value: From<NaiveDate>` 不满足；trybuild 生成 crate 无法按用例开关 feature，故 pass 面不做成 trybuild 例），pass 面由 coder-orm 的 `--features chrono` 单测覆盖。Landed（coder-macro，2026-10-06）：`tests/ui/date_field_without_chrono_feature.rs` + `.stderr` 钉住 E0277（`Value: From<NaiveDate>` 不满足；derive 的两处写路径折叠为**单错、无级联**）；bee_orm_macro 仅 [dev-dependencies] 加 chrono（serde 开）、[dependencies] 未动；锁 0 新 package（宏条目 +1 chrono 边）。**`.stderr` 稳定性口径**：help 段完整枚举 17 条 `From<T> for Value`（无截断）——§56 新增的 From 全在 cfg 下，feature 关的 scratch 编译里不存在，故该文件在 §56 落地后应**逐字节不变**；若届时 trybuild 变红且 diff 是枚举多出 chrono/decimal 行——那不是重生成信号，是 **cfg 泄漏缺陷**（feature 关面被改），路由架构师按缺陷处理。trybuild scratch 始终按宏 crate dev-dep 集编译（外层 `--all-features` 不传入），任何门禁跑法下此钉稳定。
7. **零回归**：lib 70 / m2m_json 6 / assembly 2 / 真库既有用例全绿；§55 测试扩展绿；fmt/clippy 全 rc=0。
8. **真库门（tester）**：扩展 `tests/integration_pg.rs` / `integration_mysql.rs`（DSN 门控，复用 bee-pg-x 5432 / bee-mysql-x 3407 模式）；sqlite 钉在 lib 单测。Landed（tester，2026-10-06）：pg +2 / mysql +2 用例（timestamptz 两向、date、微秒 + 真列型分派、文本 decimal 路径）；前提断言进测试（pg `SHOW timezone`=UTC；mysql SYSTEM→系统 UTC）；`--test integration_pg --test integration_mysql` rc=0（pg 10/10、mysql 7/7）。C 抽查：mysql `date_cell` DATETIME→Utc 泄漏变异 → 红 @integration_mysql.rs:313（left `…123456Z` vs right `…123456`）→ 逐字节还原（sha256 回 f3e6e98…）+ touch 后重编译复绿。A3 复核：`.stderr` sha256 `a6767dd1…` 与基线逐字节同（untracked 文件以 sha 为基线口径、`git diff` 为空属 vacuous——正确指认）；trybuild 绿本身即 cfg 泄漏检查通过。

**分工/排程**：coder-macro（§55 宏半，先；§56 拼写表随后）→ coder-orm（§55 ORM 半；§56 主体）→ tester（真库门 + 证伪）→ reviewer（收口）。§56 独立推进（不并 §55 提交）。文档终刷（api 注：feature 表、sqlite TEXT 格式、mysql 时区口径、pg 精度天花板、编译期契约）lead 触发——本文件仅记录。

**决策日志（batch 7）**：M2mDef.target_ident 取 plain `&'static str`（宏期已知，无需 fn 指针延后；PartialEq 直比 str）。提示修复走反向 def 的 target_ident（正向无 def 无 ident 可拼；占位符 `<Target>` 给形状不给错拼写；Model trait 不加 ident 项——保手写 impl 面）。chrono 显式开 serde（默认为关）；rust_decimal 默认即带。rusqlite chrono feature 不用（自文本转换）。mysql 按列型分派 DATE/DATETIME/TIMESTAMP（`V::Date` 无列型信息，必须读 `columns_ref()`）；TIMESTAMP ↔ DateTimeUtc 归 UTC（会话时区前提文档化）。sqlite 全 TEXT 声明（Decimal NUMERIC 亲和陷阱）。mysql decimal(65,30)（范围对齐 pg numeric 意图）。解码头走 serde blanket（value.rs 零改动、by construction 对称）。NaiveTime / 本地时区 / 纳秒精度范围外。

**收口（reviewer，2026-10-06；4/4 变异 + 1 LOW finding）**：F-7.1（LOW，随 §56 修）=mysql.rs:185 `column_type` 参数在 chrono 关配置 unused → `cargo clippy -p bee_orm --features mysql --all-targets -- -D warnings` rc=101（bee_rust `orm-mysql` 裸配置面；CI 不踩——CI clippy=零 feature 默认面/all-features 含 chrono）；修复 Landed（coder-orm，2026-10-06）：`#[cfg_attr(not(feature = "chrono"), allow(unused_variables))]` 落地（留名未改名、零行为差）；该配置 rc=101→0；单后端裸配置扫描（sqlite / postgres / mysql+rust_decimal 等组合）全 rc=0。覆盖观察：rel.rs `json_of` 四 cfg 臂无测试钉（变异候选③证死）→ 已补：src/tests/rel.rs 两钉（chrono：date/naive/utc 逐字；rust_decimal："1.50"）——补钉自证变异（两臂改 `Json::Null`）两钉 FAILED（left Null / right String(…)）→ cmp 还原 → 2 passed；all-features lib 77→79、默认关 52 不变。变异：M-A sqlite TEXT→DECIMAL 红 datetime_types.rs:148；M-B m2m 反向 ident→表名 红 m2m.rs:164；M-C mysql TIMESTAMP 去 Utc 红 integration_mysql.rs:313-317；M-D pg timestamptz→NaiveDateTime 红 integration_pg.rs:406；均逐字节还原（基线=冻结副本非 HEAD，树未提交；CODE_DELTA_IDENTICAL=crates/+Cargo.lock 对冻结 delta.patch 去 docs 段逐字节同）；`.stderr` sha 一致、无 cfg 泄漏；弱激活面复核（sqlite,rust_decimal 下 postgres-types=0）。**下轮候选**：sqlite Decimal 绑定臂钉 / macro DateTimeTz 泛型校验放宽 / pg numeric 读臂换 f64 变异 / m2m 占位符 UX（表名不得进 `m2m( )` 片段位——§55 约束仍在）。**环境教训（团队级）**：本机 shell `diff` 为 OpenHarmony 二进制、静默无输出——核验一律 `cmp`/`git diff`。

**F-7.1 终验（reviewer，2026-10-06，闭合）**：修复配置 clippy rc=101→0；`--all-features --lib` 79 passed（=77+2）、默认 `--lib` 52；漂移面恰两文件（mysql.rs 单处 cfg_attr 签名拆 3 行；tests/rel.rs 纯文末追加，vs HEAD 单 hunk `@@ -110,3 +110,25 @@` 零既有行改动），其余 24/24 冻结副本逐文件 MATCH；`sha256sum -c SHA256SUMS` 26/26；三未跟踪文件逐一 cmp 冻结副本 OK（datetime_types.rs 本轮零触碰）。补钉非空性独立复跑（reviewer，单臂版交叉先跑 + 授权两臂版）：两臂（`Value::Date`/`Value::Decimal` @rel.rs:165/:171）→ `Json::Null` → **恰 2 钉红、零级联**（0 passed / 2 failed / 77 filtered；left Null vs right String("1.50") / String("2026-10-06")）——分臂精确、两版互证（单臂版 decimal 钉作对照组保持绿）；还原三重口径（运行前快照 cmp + 冻结副本 cmp + `git diff HEAD -- rel.rs` 空、零 marker）、touch 后重编非陈旧；复绿 2/2 + lib 79/79。**提交完整性链（cbd4dc7）**：`git status` 全空；HEAD==freeze 24/24（未变文件）；mysql.rs 与 tests/rel.rs 的 HEAD 内容==修复后 live（cfg_attr + 两钉在提交内）；`git grep MUTATION cbd4dc7` 零命中、HEAD rel.rs==freeze → 提交未捕获变异（与独立复跑窗口重叠的 gate 点已验）；三原未跟踪文件入 HEAD 且==freeze。**排程口径记录**：§55+§56 实际合并于 cbd4dc7 单提交（原排程"§56 独立"未保持，以实际提交为准）；另 1078298 已落（CI migrate_e2e 移除离线约束 + .gitignore tmp 防线 = 1.2.1 红点修复，main 线）。批次 7 全链闭合，无遗留。

---

## 60. Batch-8: dogfood 示例应用 `examples/shortlink`（DX 挖矿）

**目标**：一个独立于工作区、只依赖 crates.io 已发布 `bee_rust = "1.2.3"`（禁 path 依赖）的短链服务示例应用。实现者 = **全新 agent（零内部上下文）**，参考面只许公开文档；每遇到"文档没讲清 / 要翻源码才懂 / API 别扭 / 报错难懂"即记一条 DX 摩擦日志。挖出的问题走架构师裁决流程，归口下轮。

### 60.1 前提、参考面与写权（硬约束）

- **前提（已核 2026-10-06）**：crates.io 上 `bee_rust 1.2.1` 可解析（`cargo search bee_rust` 返回 1.2.1）。开工时以 `cargo build` 实际解析为准；解析不到 1.2.1 → 停手报 lead。
- **独立于工作区**：短链应用自带 Cargo.toml / Cargo.lock，**不得出现任何 path 依赖**；根 Cargo.toml 不得改动。**修订（app-dev 实测 + 架构师独立探针复现，2026-10-06）**：根 `exclude = ["examples/*"]` 的 **glob 在 cargo 1.99 下不生效**（探针矩阵：glob→rc=101 "believes it's in a workspace"；字面路径 `examples/shortlink`→rc=0；入 members→rc=0——exclude 条目按字面路径匹配）——`examples/*` 条目自写下起即为 no-op（`examples/hello` 能用纯因显式 members）；故短链应用**自带空 `[workspace]` 表**豁免（app-dev 修法，正确）。根 exclude 的卫生修正待 lead 决策，见 §60.6。
- **参考面白名单**：`README.md`、`docs/crates-readme.md`、`docs/api.md`（或任一语言镜像）、docs.rs/bee_rust rustdoc、crates.io 页面。**黑名单**：`crates/**` 源码、本计划文件（§60.1–60.4 的任务简报段除外）、`examples/hello`。**卡死救急规则（架构师裁定 + lead 顺序约束）**：仅当**先尝试过文档路径**（该条日志内须记录尝试内容：查了哪份文档/哪段 API、为何不够）且仍无法推进时，才允许翻 `crates/**` 源码救急；每一次救急都必须记一条摩擦日志（类别按实际、解卡方式填"翻源码（救急）"，注明读了哪个文件、**文档为什么不够**）——顺序约束防"一上来就翻源码"；"被迫翻源码"正是 dogfood 要度量的事实，允许发生、禁止不留痕。
- **写权**：`examples/shortlink/**` 独占；不得修改本目录外的任何文件（根 Cargo.toml / CI / docs / crates 全禁）；不执行任何 git 操作（提交归 lead）；交付时目录外无残留（临时文件放系统临时目录）。
- **版本口径**：文档参考面是 main 分支（含 1.2.1 后的文档刷新）；**以 crates.io 1.2.1 的实际行为为准**——文档与行为不符同样记摩擦（版本口径差本身就是发现）。

### 60.2 应用需求（精确版）

数据模型（均 `#[derive(Model)]`）：

```rust
#[derive(Model)]
#[bee(table = "links")]
#[bee(m2m(Tag))]                       // join 表 link_tag（文档口径），随 sync 创建
struct Link {
    #[bee(pk, auto)] id: i64,
    code: String,
    url: String,
    #[bee(auto_now_add)] created_at: i64,   // 公开文档配对写法：unix 秒
    #[bee(soft_delete)] deleted: bool,
}

#[derive(Model)]
#[bee(table = "tags")]
struct Tag { #[bee(pk, auto)] id: i64, name: String }

#[derive(Model)]
#[bee(table = "clicks")]
struct Click {
    #[bee(pk, auto)] id: i64,
    #[bee(fk = Link)] link_id: i64,         // 安全阀：若公开口径要求 Option 形，可 Option 化并记摩擦
    #[bee(auto_now_add)] created_at: i64,
}
```

- 建表：启动时 `migrate::sync`，顺序 **Tag → Link → Click**（m2m 目标先建，文档要求）。
- 缓存：`MemoryCache`（bee_cache）做 code→url 热路径；**命中不查库**（缓存值须自带 link id 供点击记账）；TTL 60s 常量即可。
- HTTP：bee_router 路由 + axum 0.8 serve；JSON API；sqlite 文件库。

端点（精确行为，全部钉死）：

1. `POST /api/links` body `{url, code?}`：url 须为绝对 http(s)（前缀大小写不敏感），拒绝 `//host`、无 scheme、其他 scheme、长度 >2048 → 400；code 可选，给出须匹配 `^[a-z0-9]{4,32}$` 否则 400，未给出自动生成 8 位 `[a-z0-9]`（无新依赖实现，碰撞重试）；重复 code（**含软删行**，`with_deleted` 判重）→ 409；成功 → 201 JSON（至少 `{id, code, url}`）。
2. `GET /api/links` → 200 JSON 数组，仅存活行、id 升序；元素 `{id, code, url, created_at, clicks, tags}`；clicks = 该 link 的 Click 行数；tags = 标签名数组（tag id 升序）。N+1 查询可接受（示例应用，可在注释标 `ponytail:`）。
3. `GET /:code`（根路径）→ 302 + `Location`；命中缓存直接 302；未命中查库（仅存活行）并回填缓存；未找到/已软删 → 404；**每次 302 记一行 Click**（含缓存命中路径）。
4. `DELETE /api/links/:code` → 204 软删 + 删缓存键（`Cache::delete` 对 NotFound 需容忍）；未找到（含已删）→ 404。
5. `POST /api/links/:code/tags` body `{name}`：name trim 后非空且 ≤64 字符否则 400；Tag 按名 find-or-create；attach；**幂等**（重复打同一标签 → 200、不重复插入）；响应 200 `{tags:[…]}`；link 未找到 → 404。

结构（最低要求）：`src/lib.rs` 暴露 `pub async fn build_app(db_path: &str) -> …`（内部完成 sync 建表、建 state、返回可 serve 的 axum Router；签名形状供 e2e 直接调用）；`src/main.rs` 读 env `PORT`（默认 8080）/ `SHORTLINK_DB`（默认 `shortlink.db`）后 serve。文件划分自由，每文件 <500 行（项目规则）；**Cargo.lock 与 .gitignore（`target/`、`*.db*`）必须交付**。

依赖（建议最小集；多引进任何一个都要在摩擦日志记注）：`bee_rust = { version = "1.2.3", default-features = false, features = ["orm-sqlite", "router", "cache", "logs"] }`（orm-sqlite 不在 full 里，须显式——README feature 表口径）、`tokio`（full）、`axum = "0.8"`、`serde_json`；dev-dependencies：`reqwest`（0.12，default-features=false，features `["json","rustls-tls"]`）。不启用 orm-chrono（created_at 用文档 i64 配对）。

### 60.3 交付物与验收标准

交付物：可运行的 `examples/shortlink/`（含上述文件）+ `DX-FRICTIONS.md` 摩擦日志 + 自带的可脚本化验收流程（`tests/e2e.rs`）。完工简报（文件清单 / 门禁命令与结果 / 摩擦条目摘要 / 注意事项）发 architect；**摩擦日志全文副本同时发 main**（lead 看全量）。

**功能验收（A1–A10，e2e 必须逐条黑盒覆盖）**：
- A1 建库：空目录启动（`build_app(tmp_db)` 或 `cargo run`），`GET /api/links` → 200 `[]`。
- A2 创建：`POST /api/links {"url":"https://example.com/a","code":"abc123"}` → 201，含 id（>0）/code/url；created_at 为 unix 秒整数。
- A3 校验：url 为 `javascript:alert(1)` / `//evil.com` / `notaurl` → 400；code 为 `AB!` → 400。
- A4 唯一：重复 code → 409；**软删后的 code 再建 → 仍 409**（判重含软删）。
- A5 重定向：`GET /abc123` → 302 且 `Location: https://example.com/a`；再次 GET → 302（缓存路径）；未知 code → 404。
- A6 点击：两次 302 后 `GET /api/links` 中该行 `clicks == 2`。
- A7 打标签：`POST /api/links/abc123/tags {"name":"rust"}` → 200 `{"tags":["rust"]}`；重复同请求 → 200 且 tags 仍 `["rust"]`（幂等）；再打第二个标签 → tags 两条。
- A8 列表带关系：`GET /api/links` 元素含 `tags`（与 A7 一致）与 `clicks`（与 A6 一致）。
- A9 软删：`DELETE /api/links/abc123` → 204；**立即** `GET /abc123` → 404（缓存失效钉）；`GET /api/links` 不含该行；库行仍在（e2e 内 `Link::query().with_deleted().count` == 1）。
- A10 端到端脚本：`cargo test` 跑 `tests/e2e.rs` rc=0——自起 `127.0.0.1:0` 临时端口 + 临时库文件，覆盖 A1–A9，不依赖外部服务。

**工程门槛**（在 `examples/shortlink/` 内执行）：`cargo build` rc=0；`cargo fmt --check` rc=0；`cargo clippy --all-targets -- -D warnings` rc=0；`cargo test` rc=0。

**tester 独立验收（M 项，不采信实现者 e2e 的绿）**：
- M1 重启持久化：`SHORTLINK_DB=<tmp> PORT=<p> cargo run` 建数据 → 杀进程 → 同 DB 重启 → `GET /:code` 仍 302、`/api/links` 数据在。
- M2 独立探测：按 A1–A9 逐条自写请求复验（至少含：A4 软删判重、A9 缓存失效即时 404、A7 幂等、A3 校验、大写 `HTTPS://` 前缀接受）。

**摩擦日志格式**（`DX-FRICTIONS.md`，放 `examples/shortlink/` 内；开头记：日期、bee_rust 1.2.1、`rustc --version`、平台）：

```markdown
## F-<n> <一句话标题>
- 类别: 文档缺失 | API 设计 | 报错质量
- 场景: 做哪个端点/哪一步时
- 最小复现: 命令 / ≤10 行代码 / 报错原文
- 解卡方式: 文档内解决 | 试错 | 翻源码（救急，注明文件） | 绕道（说明绕道写法）
- 影响: 粗估耗时 / 是否卡死
- 改进建议: 对 docs 的补充点 / API 设计改动 / 报错文案
```

要求：只记真实摩擦，禁止凑数；每条必须可复现（reviewer 逐条验）；发生过翻源码就必须有对应条目，且条目内先记录文档路径的尝试（顺序约束，见 §60.1）。

### 60.4 分工与排程

- **实现**：全新 agent（零内部上下文），只给 §60.1–60.3 简报（可直接转贴）+ 白名单；完工把简报（文件清单 / 门禁命令与结果 / 摩擦条目摘要 / 注意事项）发 architect。
- **tester**：行为验收（60.3 全项，含 M1/M2）。
- **reviewer**：摩擦日志分诊（逐条验真、去重、判定 文档缺陷 | API 候选 | 预期学习成本 | wontfix，并给归口）+ 轻审代码（越界改动、死代码、安全边界（open redirect/校验）、e2e 断言质量、ponytail 注释合理性）。
- **路由**：findings → architect 裁决；docs 类聚合成 lead 触发的文档终刷清单；API 类进下轮候选。**本批不改 `crates/**` 源码**——挖到的缺陷只记录、不回灌。**摩擦日志副本要求（lead）**：完工简报发 architect 时，摩擦日志全文副本同时发 main（lead 看全量，见 §60.3）。

### 60.5 非目标

认证/会话/分页/前端/Docker/CI 接入/压测/性能；不改 `examples/hello`；不做 code 回收站、自定义短码规则引擎、统计报表。

### 60.6 进度（batch-8，2026-10-06）

- **app-dev 完工简报**（已收，摩擦副本要求已传达）：examples/shortlink/ 9 文件（Cargo.toml 18 / Cargo.lock 258 包 / .gitignore / src {lib 40, models 38, handlers 288, main 13} / tests/e2e.rs 228 / DX-FRICTIONS.md 94）；门禁四连 rc=0（build / fmt --check / clippy --all-targets -D warnings / test）；e2e 6/6（A1–A9 全覆——A5+A9 合证缓存失效，A6 clicks==2 证缓存值携 id）；真二进制冒烟 302 通过；**摩擦 6 条、翻源码 0 次**：F-1 derive 宏展开 `bee_orm::` 路径致 E0433（试错解卡 `use bee_rust::bee_orm;`）；F-2 handler 注入 Pool/Cache 文档零覆盖（rustdoc with_state 一句解卡）；F-3 docs.rs 缺口（derive(Model) 空页 / pool::sqlite 404〔构建未开后端 feature〕/ router 22.73% / query() 不可见）；F-4 路径模板 `{code}` vs `:code` 与 ns 空前缀未文档化；F-5 `insert() -> u64` 未文档化（实测=行数非主键；auto pk/auto_now_add 致每创建路径二次 SELECT）；F-6 仓库 exclude glob 无效（非框架）。
- **架构师核验（第一手）**：文件树/行数与简报一致（wc）；`git status` 仅 `M` plan 文件 + `?? examples/shortlink/`（目录外零改动成立）；Cargo.lock 258 包、`bee_rust 1.2.1 @ registry`、零 `path+` 行（crates.io 实拉成立）；models/handlers/lib 与 §60.2 形状一致（sync 序 Tag→Link→Click；缓存值 {id,url} 命中不查库；DELETE 清缓存容忍 NotFound；幂等打标签经 related 判重且 tags 按 id 升序）。
- **F-6 根因（探针矩阵）**：cargo 1.99 的 `workspace.exclude` 按字面路径匹配、**不支持 glob**——`examples/*` 自始为 no-op；字面路径与 members 两式均奏效。**已裁定（lead，2026-10-06）**：采纳卫生修复、**不加 members**（理由同意架构师：crates.io 版与 path 版 bee_rust 同 lock 并存会破坏"验收已发布产品"边界，且 CI `--workspace` 被拖累）——根 `exclude` 改为字面条目 `["examples/shortlink"]` + 一行注释（说明 cargo 的 exclude 为字面匹配、glob 无效，防未来再踩；由 lead 在 batch-8 提交时一并落，架构师不动根文件）；`examples/hello` 保持 members 不动；shortlink 包内空 `[workspace]` 表**保留**（belt-and-braces：不依赖根 exclude 也能自证独立）。
- **路由**：tester（行为验收 A1–A10 + M1 重启持久化 + M2 独立探测 + 可选冷 CARGO_HOME 拉取）与 reviewer（摩擦 6 条逐条验真分诊 + 轻审）已发令；shortlink 写入冻结，解冻令由架构师发。
- **冻结基线（app-dev 2026-10-06 发布，架构师 `sha256sum` 复核 9/9 逐字节一致）**：`Cargo.toml` bfbbb9f3fb334e8276dfbc2977ba3daa7d9e12e77a1b0382081e893f77fb5713 ｜ `Cargo.lock` b52aa7efe660b1ee634fdca5865c2f2a6f587dad65b8c6d580ba38e812c4452c ｜ `.gitignore` ebe6989e42d0d47b71fbf7c13707daa83c5b7212362ce1d2d2293093b4f90785 ｜ `src/lib.rs` 60de603fc5d7e82218e2ac1e803075ea7c31842398496c741a665994817d6e61 ｜ `src/main.rs` 0ee7f38cbec626b5f5cb35a4064f4a22cd2ba423c77132c5069cf23606d239e0 ｜ `src/models.rs` 49db8d260dc27a90abdb540709c468680ceed838078830f6b44dc477de41fec2 ｜ `src/handlers.rs` 61b91eca2b7b57c68531d3f6836a980046d0491c904bb946302db6c92749c8d3 ｜ `tests/e2e.rs` 863bc5f9eea923d32529aa9f68f3617b6c14d49eb2fce6a006252286c714bcc8 ｜ `DX-FRICTIONS.md` d8852f8b6a7a667930ed94c201896070a43a18258d7daf483f31172318d64a92（target/ 未清理不影响；验收期内任何字节变动=破坏冻结，需报告）。
- **tester 行为验收完成（2026-10-06）：全绿、0 缺陷。** 工程门槛四连 rc=0（build 41.56s / fmt --check / clippy --all-targets -D warnings / test）；M2 独立黑盒 = 自写 python3 脚本（真二进制自 spawn/kill、临时端口 + 临时库、断言自持）**28/28 连跑两遍**（首遍 27/28 系 tester 自身断言口径不齐〔M1 腿 GET 白记一次点击〕、自纠，非产品缺陷；两次原始日志留档）；M1 真重启持久化（同库 kill/restart：302 与 Location 逐字节在、clicks=1+1=2〔重启后记账仍落库〕、软删行仍 404）；边界全钉（`javascript:` / `//evil.com` / `notaurl` / `AB!`→400；软删后同 code 重建→409 证 with_deleted 判重；A9 缓存失效钉 + python3 sqlite3 直读库文件证 deleted=1 行实存；附加：大写 `HTTPS://`→201、标签打不存在 link→404、空白 name→400、重复 DELETE→404）。**冷拉验收：`CARGO_HOME=$(mktemp -d) cargo build` rc=0（3m10s，163 个 crate 真下载、含 `Compiling bee_rust v1.2.1`）——crates.io 1.2.1 完整性与冷缓存真拉证据成立。** 结构只读复核：9 文件齐、0 path 依赖、根 Cargo.toml/Cargo.lock 零触碰、全 <500 行、e2e 自起 127.0.0.1:0 + temp_dir 库、摩擦日志格式合规（F-1..F-6、翻源码 0 次；验真归 reviewer）。**冻结完整性（架构师跑后独立复算）：9/9 sha256 MATCH**（tester 侧接力补充：跑后自拍 9/9 与冻结清单程序化 diff 空 + mtime 窗口证据〔9 文件 mtime 全 ≤15:12、其门槛四连约 15:13 起、探针日志 15:18〕——验收窗口内包内零字节变动）；前后照双人闭环 = before 照（架构师冻结时刻核验）+ after 照（tester 跑后自拍 + 架构师复算，两遍独立）；`git status` 仍仅 plan `M` + `examples/shortlink/` `??`。临时物已清（无残留进程/DB）。**reviewer 分诊与轻审已收回（2026-10-06，见下）。**
- **reviewer 摩擦分诊收口（6/6 逐条实测验真；基座=/tmp/dxrepro 最小 crate〔1.2.1 crates.io 实拉〕+ docs.rs curl 直取）**：F-1 文档缺陷（主）+下轮候选（宏路径健壮性，可选）；F-2 文档缺陷（**出处订正**：日志谓"README 快速开始"，实为 `docs/crates-readme.md:30`——现 README 快速开始无 Rust 代码块）；F-3 文档缺陷+仓库项（4 项全中：derive.Model 页一句占位 / `pool::sqlite` 404 根因=`bee_orm/Cargo.toml` **无 `[package.metadata.docs.rs]`**〔架构师复核确无〕/ bee_router 22.73% 原文 / Model trait 恰 22 方法、`query` 由 derive 生成故 0 命中）；F-4 文档缺陷（实捕 panic「Path segments must not start with `:`」、ns=纯前缀拼接、空前缀合法）；F-5 **两处日志描述订正**（① "u64 未文档化"不准——trait rustdoc 有「returns affected rows.」〔model.rs:283〕，缺口降级为 api.md 未写+未提示 auto-pk 回查；② auto_now_add 非 DB 侧——ORM 注入 `Value::Int(unix_now())`〔model.rs:433〕，反证 create() 回填可行）+**API 候选（主）**+文档缺陷（次）；F-6 非框架（已裁定）。去重：6 条=6 独立项（F-1(b) 并入 F-3 派生页小节；F-2/F-3 同源 docs.rs 成因不同不并）。**DX-FRICTIONS.md 为冻结件不修订——F-5 订正只落本段+终刷清单。**
- **裁定（架构师）**：① F-5 入**下轮候选**（非破坏：`insert` 返 PK 或 `create(&self)->Result<Self>`；rusqlite `last_insert_rowid` 既有）——归档不排程。② F-3 docs.rs metadata=**仓库项随下版**（`[package.metadata.docs.rs]`，首选 `all-features = true`、docs.rs 构建失败退 `features = ["sqlite"]`）——不占轮次。③ E-1/E-2 + F 系列 docs 部分全部并入下方终刷清单（13 镜像）。④ 轻审两条 LOW（`oops()` 500 回显内部错误文本；URL 仅前缀校验致 `https://`/控制字符过 201、其后 302 无 Location）与 create_link TOCTOU 注释缺失——**均不解冻**：示例级/可选加固，冻结件不动、账记此处；shortlink 升格模板时再议。
- **验真额外收获（白名单文档面）**：E-1 `docs/api.md:34-35` 路由注册示例 `ns.get("/users").post("/users")` **缺 handler 参、不可编译**（13 镜像同款；架构师复核原文在）；E-2 README「在项目中使用」`git = …` 与 crates-readme `cargo add bee_rust` 口径不一。
- **batch-8 文档终刷清单（lead 触发，13 镜像全改）**：a. api.md ORM 段补"仅用 bee_rust 时 `use bee_rust::bee_orm;`"（F-1a）；b. api.md 增带 state 的 handler 最小示例 + Controller 挂载说明（F-2）；c. api.md:34-35 示例补 handler 参数（E-1）；d. api.md 注明 path 为 axum 0.8 `{name}` 语法、`ns` 纯前缀拼接、空前缀合法（F-4）；e. api.md insert 小节：返受影响行数 + auto-pk/auto_now_add 二次 SELECT 模式 + auto_now_add 口径修正（F-5）；f. derive(Model) 属性表 + Model trait 页注明派生方法（F-3，与 a 合并小节）；g. README 改 `cargo add bee_rust`（E-2）。**仓库项**：h. bee_orm docs.rs metadata（随下版）；i. bee_router 源码 rustdoc 补注释（→下轮候选）；j. 根 `Cargo.toml` exclude 字面化（lead 随本批提交落）。**下轮候选（API）**：k. F-5 insert→PK / create()；l. F-1b 宏路径健壮性（proc-macro-crate，可选）。
- **轻审结论（reviewer，零改动）**：越界零（"验收已发布产品"边界保住：无 path、lock registry）；结构合规（全 ≤288 行、无死代码）；安全钉全在（open redirect 校验、4xx 无泄漏、302 Location 注入不可达——CRLF 经 axum HeaderValue 类型层丢弃实证）；e2e 非空转（A9=真缓存失效证）；ponytail 注释诚实（N+1、find-or-create 竞态属实）。**batch-8 至此收口**：tester 全绿 0 缺陷 + reviewer 分诊完成 + 轻审零 finding（两条 LOW 裁定不解冻）；唯一遗留 = lead 落根 exclude 修正 + 提交。
- **终极裁定 Y（2026-10-06，lead，"争议到此为止"；A 及此前一切口径均 superseded）**：**维持冻结、不修**——① 撤令后"已在飞别浪费"前提消失；② 存档 diff 完整，且**批九做 F-5（insert 返 PK/create()）时 handlers.rs 反正要动**——两条 LOW 修复与 F-5 合并为一次自然触碰，比单开一轮更省。落地：窄修已由架构师 `git checkout` 还原至 HEAD（diff 存档 `/tmp/shortlink_handlers_low_fix.diff`〔43 行，digest `b8e69d2f…`〕，批九直接 `git apply` 复用），复验 9/9 sha 回基线、整仓 status 仅 plan 增量（lead 下次提交一并落）。**11f3f51 = batch-8 唯一提交、无污染**（committed 9/9 blobs == 原冻结基线逐字节，含 handlers.rs=61b91eca…；根 exclude 字面化 + 注释在；规格 §60 增量 122 行）。create_link TOCTOU 注释归档"升格模板时必须修"。**口径交叉全史（存档）：批准修 → 架构师④不解冻 → 接受反裁定 → 终裁 A → 终极裁定 Y。**
- **batch-9 候选归档（本轮不做）**：F-5 API（`insert` 返 PK 或 `create() -> Self`）**与两条 LOW 修复（`oops()` 泛化 + URL 收紧，diff 存档复用）合并同做**（handlers.rs 一次触碰）；F-1 宏侧修法（架构师下轮权衡三案：宏内探测 / `#[bee(crate=…)]` 属性 / 纯文档修；proc-macro-crate=新依赖）；F-3 docs.rs metadata（各 bee_* crate 加 `[package.metadata.docs.rs] features`）。
- **batch-9 候选归档（本轮不做）**：F-5 API（`insert` 返 PK 或 `create() -> Self`）；F-1 宏侧修法（架构师下轮权衡三案：宏内探测 / `#[bee(crate=…)]` 属性 / 纯文档修；proc-macro-crate=新依赖）；F-3 docs.rs metadata（各 bee_* crate 加 `[package.metadata.docs.rs] features`）。
- **文档终刷范围（lead 触发）**：E-1/E-2 + F-1..F-4 文档部分 ×13；F-5 文档子项（原 a–g 清单 e。）随批九 API 裁定一并处理（避免先写后改）。
- **reviewer 基线自证 + 归档（2026-10-06）**：live 9/9 sha256 与 §60.6 清单逐字一致、mtime 全 ≤15:12（验收窗口前）、git status 仅 plan M + shortlink ??——其分诊全程零字节触碰；其侧无 pending。

---

## 61. Batch-9：未作项收口（F-5 / F-1 / F-3 + LOW 重放；2026-10-06，lead 发令）

**发令背景**：用户令"完成所有未作项"；全新起点，与 batch-8 交叉史无关。batch-8 已冻结于 `11f3f51` + `683ca50`；其两条 LOW 修复（存档 `/tmp/shortlink_handlers_low_fix.diff`，digest `b8e69d2f…`，tester 33/33×2 为其既有验证基线）在本批随 F-5 一次触碰重放。

### 61.1 F-5（API，非破坏）——架构师裁决：新增 `create(&self) -> Result<Self>`

- 非破坏硬约束：`insert()` 签名与语义原样（返回受影响行数）；手写 Model impl 不因新方法被破坏——**以 trait 默认方法提供**（默认实现不可行 → 停报架构师，不得动 trait 既有面）。
- 语义：插入后回读并返回完整实例（主键 + ORM/DB 注入值回填，如 auto pk、auto_now_add 的 unix_now），单行；观测量等价于 insert + 按 pk select（允许后端内部优化）。命名与收参风格对齐既有 Model 方法；若与既有方法名冲突 → 停报架构师改选，勿就地改名。
- 三后端 PK 回取（实现自由度归 coder-orm，观测量一致）：sqlite `last_insert_rowid`；pg `INSERT ... RETURNING`（或 insert+回查）；mysql `last_insert_id`。
- 裁决理由（否决单独公开 `insert_returning_pk`）：create() 直消 F-5 的 DX 痛点（插入后拿 id/要手写二次 SELECT）；其内部必经 PK 回取机制=该形态超集——公开面只留一枚更高级形态（YAGNI；裸 PK 需求出现再加，非破坏）。
- docs 随批：rustdoc 由 coder-orm 就地写；用户面 api.md insert/create 小节归文档终刷（§61.7）。

### 61.2 F-1（宏侧）——架构师裁决：`#[bee(crate = "…")]` 显式属性

- 形态：serde 式 LitStr 值（`crate` 为关键字，parse_any/Token 处理）；值如 `"bee_rust::bee_orm"`；expand 以该路径前缀替换硬编码 `bee_orm::`；**无属性时展开逐字节不变**。
- 裁决理由：零新依赖、trybuild 全可测、显式可预测。**否选**：宏内探测（proc-macro-crate=新依赖；用户只依赖 bee_rust 时 `bee_orm` 非其直接依赖、探测需 fallback 组合、树内不可测）；纯文档修（摩擦保留——仅作属性缺席时的文档备选）。
- api.md 补丁无论选择均写（属性用法 + `use bee_rust::bee_orm;` 备选）→ 归文档终刷。

### 61.3 F-3（docs.rs metadata）——架构师裁决：`[package.metadata.docs.rs] all-features = true`

- 范围：至少 bee_orm、bee_kv、bee_cache、bee_rust；其余 coder-orm 按"默认 feature 构建会藏公开面"口径枚举。
- 裁决理由：全 feature 自动覆盖、零维护漂移；docs.rs 构建能力足够（bundled sqlite / rustls 编译重但可行）。**退路**（仅首发 docs.rs 构建失败时启用）：显式 `features = ["sqlite","postgres","mysql","chrono","rust_decimal"]`。
- derive(Model) 属性表 doc 补在 bee_orm_macro 宏源码 rustdoc（coder-macro 同批）。

### 61.4 shortlink 触碰——②③ 均并入发布后窗口（终态；单写入者 = coder-orm）

- **终态（2026-10-06，lead「最后一条」，以磁盘为准；唯一口径、不再翻转）**：shortlink 零触碰、回基线 `61b91eca…`（9/9、status 净）——**②（LOW 重放）与 ③（create() 改造）均并入发布后窗口**；批九 = F-5 + F-1 + F-3。
- 口径史（存档；全部中间态 superseded）：修订一「②-now」（架构师）与 lead 直裁「②③ 合并挂起」交叉 → coder-orm 按修订一 apply 至 `64f4070d…` 并**跑完四门禁全绿**（冷构建 11m22s、fmt/clippy rc=0、e2e 6/6；双钉逐字相符）→ 架构师还原令 + 催办令 → coder-orm 还原回基线（17:17 实拍）→ lead「② 保留」（R2）→ 架构师 re-apply 令（**已执行**，第二次 apply 落盘 `64f4070d…`）→ **lead 终态「磁盘为准，②③ 均挂起」（R3；R2 作废）→ 架构师撤销 re-apply 令 + 第二次还原令（回基线）**。教训：同一事项 5 次口径翻转、apply→revert→apply→revert 两轮空转（零字节损失：存档 + sha 钉全程在位）——收敛手段=「磁盘为准 + 单一终态」。
- ③ 不可编译根因：shortlink 钉 registry `bee_rust = "1.2.3"`、含 F-5 版本未发布；path/[patch] 越界否决。
- 发布后窗口（1.2.2 后，届时顺序）：① 核存档 == `b8e69d2f…`（源=§61.9 嵌入；字节副本 /tmp、`target/`、`docs/superpowers/patches/`，去留 lead 提交时定）；② `git apply`（对基线）；③ 双钉 `64f4070d…` + 其余 8 == §60.6；④ create 三处改造（:122/:204/:266 定案）→ `cargo update -p bee_rust`（registry 口径）→ 新 9 文件 sha；⑤ 四门禁 + tester 复验（33/33 复用 + 前后双拍）。既有证据一并可用：tester 33/33×2（batch-8）+ coder-orm 本批四门绿（冷构建）。
- **话题级冻结（2026-10-06，lead，唯一有效口径、压过一切）**：shortlink 当前字节状态（`61b91eca…` 或 `64f4070d…`）**均不再重要**——批九提交用显式路径（crates/bee_orm + crates/bee_orm_macro + 相关 manifest），**shortlink 不入本提交**；一切 shortlink 动作/指令/回执即刻全停（在途动作不管）；**②③ 收尾话题仅由 lead 在 1.2.2 发布后明确重开**，此前任何来源的 shortlink 指令一律作废。本条为本话题终态。

### 61.5 验收

- 常规四门禁；受影响 crate clippy 矩阵（F-1 后 bare/全 feature 双跑）。
- tester 真库门：F-5 三后端 PK 语义各一钉（sqlite/pg/mysql，bee_orm 侧）+ 既有套件 / clippy 矩阵（bare/全 feature 双跑）；**跑前/跑后磁盘 sha 双拍**。shortlink：零触碰；33/33 复跑随 ②③ 挂起至发布后窗口（§61.4 终态）。
- reviewer 收口：F-5 签名/重命名审查、F-1 属性/探测面审查。

### 61.6 纪律（batch-8 两条硬教训，硬约束）

1. 收口期状态变更只由单一写入者执行（本批提交归 lead；中间态维持者=唯一角色）。
2. 评审/验收的冻结基线只用磁盘 sha、不用消息链断言。

### 61.7 文档终刷（a–g 不变，lead 在批九代码冻结后触发）

E-1（路由示例补 handler 参 + 真编译 ×13）、E-2、F-1..F-5 文档部分（含 F-5 create/insert 口径）、docs.rs 生效说明。

### 61.8 发令与回执

coder-orm（61.1 + 61.3 + 61.4）/ coder-macro（61.2 + derive doc）→ tester（61.5）→ reviewer（61.5）→ lead 提交。回执格式="收到指令集 1..N"（列步骤号，防交叉）。

---

### 61.9 存档：shortlink LOW 修复 diff（字节嵌入，2026-10-06）

- 用途：消除 /tmp 易失（batch-8 教训；diff 复用/复核不再依赖 /tmp）。原文件 `/tmp/shortlink_handlers_low_fix.diff`，sha256 `b8e69d2ffa3217440f671d1da31c85be4c69b8e6b4e46d4ee0408c15ddd5a4fe`，43 行 / 1719 字节。
- 下方 fenced block 内容为逐字节嵌入（提取 sha256 已验证 == 上述值）。

```diff
diff --git a/examples/shortlink/src/handlers.rs b/examples/shortlink/src/handlers.rs
index 95208c9..83c5585 100644
--- a/examples/shortlink/src/handlers.rs
+++ b/examples/shortlink/src/handlers.rs
@@ -34,7 +34,9 @@ fn bad_request(msg: impl Into<String>) -> HttpError {
 }
 
 fn oops(e: impl std::fmt::Display) -> HttpError {
-    HttpError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
+    // Internal details go to the log; the client only gets a generic message.
+    eprintln!("shortlink: internal error: {e}");
+    HttpError(StatusCode::INTERNAL_SERVER_ERROR, "internal server error".into())
 }
 
 fn cache_key(code: &str) -> String {
@@ -46,6 +48,14 @@ fn is_http_url(url: &str) -> bool {
     lower.starts_with("http://") || lower.starts_with("https://")
 }
 
+/// The host part — after `://`, up to the first `/`, `?`, `#` or the end — must be non-empty.
+fn has_host(url: &str) -> bool {
+    let Some(after_scheme) = url.find("://").map(|i| &url[i + 3..]) else {
+        return false;
+    };
+    !after_scheme.split(['/', '?', '#']).next().unwrap_or("").is_empty()
+}
+
 fn is_valid_code(code: &str) -> bool {
     (4..=32).contains(&code.len())
         && code.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
@@ -91,7 +101,11 @@ pub async fn create_link(
     let Some(url) = body.get("url").and_then(|v| v.as_str()) else {
         return Err(bad_request("url is required"));
     };
-    if url.len() > 2048 || !is_http_url(url) {
+    if url.len() > 2048
+        || !is_http_url(url)
+        || url.bytes().any(|b| b < 0x20 || b == 0x7F)
+        || !has_host(url)
+    {
         return Err(bad_request("url must be an absolute http(s) URL of at most 2048 chars"));
     }
 
```
