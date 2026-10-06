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
